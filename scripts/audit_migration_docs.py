#!/usr/bin/env python3
"""审计 Spring Java 对象到 Vernal Rust 文件的迁移事实。

本脚本只依赖 Python 标准库。默认读取 ``docs/migration-manifest.toml``，
按固定的“保留末两层包目录”规则生成报告，并校验全部迁移文档都有权威级别标记。
"""

from __future__ import annotations

import argparse
import dataclasses
import re
import subprocess
import sys
import tomllib
from collections import Counter, defaultdict
from pathlib import Path
from typing import Iterable


COMPLETE_STATUSES = {"IMPLEMENTED", "DEPENDENCY_REUSED", "PLATFORM_NA"}
INCOMPLETE_STATUSES = {"MISSING", "MISPLACED", "STUB", "PARTIAL", "UNVERIFIED"}
ALL_STATUSES = COMPLETE_STATUSES | INCOMPLETE_STATUSES
STUB_PATTERNS = (
    re.compile(r"\btodo!\s*\("),
    re.compile(r"\bunimplemented!\s*\("),
    re.compile(r"\bTODO\b"),
    re.compile(r"\bSTUB\b", re.IGNORECASE),
    re.compile(r"占位(?!符)"),  # "占位符" 是合法术语,不视为 stub 标记
    re.compile(r"实际实现需要"),
    re.compile(r"执行销毁逻辑"),
)
PUBLIC_TYPE_PATTERN = re.compile(
    r"(?m)^pub(?:\([^)]*\))?\s+(?:unsafe\s+)?"
    r"(?:struct|enum|trait|type)\s+([A-Z][A-Za-z0-9_]*)\b"
)
DOC_MARKER_PATTERN = re.compile(
    r"<!--\s*migration-doc:\s*authority=(authoritative|support|historical)"
    r"\s+canonical=([^\s]+)\s*-->"
)
REQUIRED_CURRENT_DOCUMENTS = (
    "对象名称一致性检查.md",
    "对象级对照表.md",
    "语义迁移对照表.md",
    "迁移路线图.md",
)
OBJECT_TABLE_APPENDIX_START = "<!-- historical-design-appendix-start -->"
OBJECT_TABLE_APPENDIX_END = "<!-- historical-design-appendix-end -->"
CURRENT_CONTRACT_START = "<!-- current-migration-contract-start -->"
CURRENT_CONTRACT_END = "<!-- current-migration-contract-end -->"


@dataclasses.dataclass(frozen=True)
class ModuleConfig:
    """单个 Spring→Vernal 模块的扫描配置。"""

    name: str
    source_root: Path
    source_package: str
    source_commit: str
    source_prefixes: tuple[Path, ...]
    target_root: Path
    target_optional: bool
    docs_dir: Path
    retain_segments: int
    object_table: Path | None
    dependencies: tuple[dict[str, str], ...]
    dispositions: tuple[dict[str, str], ...]


@dataclasses.dataclass(frozen=True)
class ObjectRecord:
    """一个 Java 对象的当前迁移判定。"""

    fqn: str
    source_path: str
    expected_path: str
    actual_path: str
    status: str
    evidence: str


def camel_to_snake(name: str) -> str:
    """将 Java PascalCase 类型名转换为 Rust snake_case 文件名。"""

    first = re.sub(r"([A-Z]+)([A-Z][a-z])", r"\1_\2", name)
    second = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", first)
    return second.replace("$", "_").lower()


def expected_rust_path(java_relative_path: Path, retain_segments: int = 2) -> Path:
    """计算 Java 源对象对应的 Rust 相对路径。"""

    package_parts = java_relative_path.parent.parts
    kept = package_parts[-retain_segments:] if package_parts else ()
    rust_name = f"{camel_to_snake(java_relative_path.stem)}.rs"
    return Path(*kept, rust_name) if kept else Path(rust_name)


def has_stub_marker(text: str) -> bool:
    """判断 Rust 文件是否含明确的 stub/占位标记。"""

    return any(pattern.search(text) for pattern in STUB_PATTERNS)


def production_rust_files(target_root: Path) -> list[Path]:
    """返回除 lib.rs/mod.rs 外的生产 Rust 对象文件。"""

    return sorted(
        path
        for path in target_root.rglob("*.rs")
        if path.name not in {"lib.rs", "mod.rs"}
    )


def java_object_files(source_root: Path) -> list[Path]:
    """返回 Java 业务对象文件，不计 package-info.java。"""

    return sorted(
        path
        for path in source_root.rglob("*.java")
        if path.name != "package-info.java"
    )


def read_manifest(repo_root: Path, manifest_path: Path) -> tuple[dict, list[ModuleConfig]]:
    """读取 TOML 清单并解析为模块配置。"""

    data = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    retain_default = int(data.get("retain_segments", 2))
    modules: list[ModuleConfig] = []
    for raw in data.get("module", []):
        object_table = raw.get("object_table")
        modules.append(
            ModuleConfig(
                name=raw["name"],
                source_root=(repo_root / raw["source_root"]).resolve(),
                source_package=raw["source_package"],
                source_commit=raw["source_commit"],
                source_prefixes=tuple(
                    Path(prefix) for prefix in raw.get("source_prefixes", [])
                ),
                target_root=(repo_root / raw["target_root"]).resolve(),
                target_optional=bool(raw.get("target_optional", False)),
                docs_dir=(repo_root / raw["docs_dir"]).resolve(),
                retain_segments=int(raw.get("retain_segments", retain_default)),
                object_table=(repo_root / object_table).resolve() if object_table else None,
                dependencies=tuple(raw.get("dependencies", [])),
                dispositions=tuple(raw.get("dispositions", [])),
            )
        )
    return data, modules


def _test_sources(target_root: Path) -> list[tuple[str, str]]:
    """一次性收集目标 crate 的集成测试和单元测试片段及其路径。"""

    crate_root = target_root.parent
    sources: list[tuple[str, str]] = []
    tests_root = crate_root / "tests"
    if tests_root.is_dir():
        for path in tests_root.rglob("*.rs"):
            sources.append(
                (
                    path.relative_to(crate_root).as_posix(),
                    path.read_text(encoding="utf-8", errors="ignore"),
                )
            )
    for path in target_root.rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="ignore")
        marker = text.find("#[cfg(test)]")
        if marker >= 0:
            sources.append(
                (
                    f"{path.relative_to(crate_root).as_posix()}#[cfg(test)]",
                    text[marker:],
                )
            )
    return sources


def _test_evidence(
    test_sources: list[tuple[str, str]],
    rust_stem: str,
    java_type: str,
) -> str | None:
    """返回引用目标文件名或 Java 类型名的第一份测试证据路径。"""

    for path, text in test_sources:
        if rust_stem in text or java_type in text:
            return path
    return None


def audit_module(repo_root: Path, module: ModuleConfig) -> list[ObjectRecord]:
    """扫描单个模块并返回逐对象判定。"""

    if not module.source_root.is_dir():
        raise FileNotFoundError(f"{module.name}: source root not found: {module.source_root}")
    if not module.target_root.is_dir() and not module.target_optional:
        raise FileNotFoundError(f"{module.name}: target root not found: {module.target_root}")

    rust_files = (
        production_rust_files(module.target_root)
        if module.target_root.is_dir()
        else []
    )
    test_sources = (
        _test_sources(module.target_root)
        if module.target_root.is_dir()
        else []
    )
    by_stem: dict[str, list[Path]] = defaultdict(list)
    for rust_file in rust_files:
        by_stem[rust_file.stem].append(rust_file)

    disposition_by_fqn: dict[str, dict[str, str]] = {}
    for disposition in module.dispositions:
        fqn = disposition["java_fqn"]
        if fqn in disposition_by_fqn:
            raise ValueError(f"{module.name}: duplicate disposition for {fqn}")
        status = disposition["status"]
        if status not in {"DEPENDENCY_REUSED", "PLATFORM_NA"}:
            raise ValueError(f"{module.name}: invalid disposition status {status}")
        if status == "DEPENDENCY_REUSED":
            dependency = next(
                (
                    item
                    for item in module.dependencies
                    if item["crate"] == disposition.get("crate")
                ),
                None,
            )
            if dependency is None:
                raise ValueError(f"{module.name}: unknown reused crate for {fqn}")
            symbol = disposition.get("symbol", "")
            if not symbol or symbol not in dependency.get("symbols", ""):
                raise ValueError(f"{module.name}: unregistered dependency symbol for {fqn}")
            test_path = repo_root / disposition.get("integration_test", "")
            if not disposition.get("integration_test") or not test_path.is_file():
                raise ValueError(f"{module.name}: missing dependency integration test for {fqn}")
        if status == "PLATFORM_NA" and not disposition.get("platform_evidence"):
            raise ValueError(f"{module.name}: missing platform evidence for {fqn}")
        disposition_by_fqn[fqn] = disposition

    records: list[ObjectRecord] = []
    for java_file in java_object_files(module.source_root):
        java_relative = java_file.relative_to(module.source_root)
        mapping_relative = java_relative
        for prefix in module.source_prefixes:
            prefix_parts = prefix.parts
            if java_relative.parts[: len(prefix_parts)] == prefix_parts:
                mapping_relative = Path(*java_relative.parts[len(prefix_parts) :])
                break
        expected = expected_rust_path(mapping_relative, module.retain_segments)
        candidates = by_stem.get(expected.stem, [])
        fqn_suffix = ".".join(java_relative.with_suffix("").parts)
        fqn = ".".join(
            part for part in (module.source_package, fqn_suffix) if part
        )
        disposition = disposition_by_fqn.get(fqn)
        if disposition is not None:
            status = disposition["status"]
            if status == "DEPENDENCY_REUSED":
                actual = f'{disposition["crate"]}::{disposition["symbol"]}'
                evidence = (
                    f'依赖精确符号；集成测试 `{disposition["integration_test"]}`'
                )
            else:
                actual = "—"
                evidence = disposition["platform_evidence"]
            records.append(
                ObjectRecord(
                    fqn=fqn,
                    source_path=java_relative.as_posix(),
                    expected_path=expected.as_posix(),
                    actual_path=actual,
                    status=status,
                    evidence=evidence,
                )
            )
            continue

        if not candidates:
            records.append(
                ObjectRecord(
                    fqn=fqn,
                    source_path=java_relative.as_posix(),
                    expected_path=expected.as_posix(),
                    actual_path="—",
                    status="MISSING",
                    evidence="未找到同名 Rust 对象文件",
                )
            )
            continue

        exact = next(
            (
                candidate
                for candidate in candidates
                if candidate.relative_to(module.target_root) == expected
            ),
            None,
        )
        if exact is None:
            actual = ", ".join(
                candidate.relative_to(module.target_root).as_posix()
                for candidate in candidates
            )
            records.append(
                ObjectRecord(
                    fqn=fqn,
                    source_path=java_relative.as_posix(),
                    expected_path=expected.as_posix(),
                    actual_path=actual,
                    status="MISPLACED",
                    evidence="文件名存在，但未位于保留末两层包目录计算出的路径",
                )
            )
            continue

        text = exact.read_text(encoding="utf-8", errors="ignore")
        relative_exact = exact.relative_to(module.target_root).as_posix()
        if has_stub_marker(text):
            status = "STUB"
            evidence = "存在 todo!/unimplemented!/TODO/占位或空业务逻辑标记"
        else:
            has_source_doc = "对应 Java" in text and java_file.stem in text
            public_types = PUBLIC_TYPE_PATTERN.findall(text)
            has_main_type = java_file.stem in public_types
            test_evidence = _test_evidence(
                test_sources,
                exact.stem,
                java_file.stem,
            )
            if has_source_doc and has_main_type and test_evidence:
                status = "IMPLEMENTED"
                evidence = (
                    "预期路径、公开主类型、中文 Java 来源注释均存在；"
                    f"测试证据 `{test_evidence}`"
                )
            else:
                status = "UNVERIFIED"
                missing = []
                if not has_source_doc:
                    missing.append("中文 Java 来源注释")
                if not has_main_type:
                    missing.append("同名公开主类型")
                if not test_evidence:
                    missing.append("测试引用")
                evidence = "缺少" + "、".join(missing)
        records.append(
            ObjectRecord(
                fqn=fqn,
                source_path=java_relative.as_posix(),
                expected_path=expected.as_posix(),
                actual_path=relative_exact,
                status=status,
                evidence=evidence,
            )
        )
    return records


def validate_rust_red_lines(module: ModuleConfig) -> list[str]:
    """收集 mod/lib 与生产 wildcard import 红线。

    这些事实会进入报告并使相应模块保持未完成；本轮交付不修改业务源码，
    因此既存债务本身不作为文档生成失败条件。
    """

    violations: list[str] = []
    for path in module.target_root.rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="ignore")
        relative = path.relative_to(module.target_root)
        if path.name in {"lib.rs", "mod.rs"} and PUBLIC_TYPE_PATTERN.search(text):
            violations.append(f"类型定义位于 `{relative}`")
        if path.name not in {"lib.rs", "mod.rs"}:
            public_types = PUBLIC_TYPE_PATTERN.findall(text)
            non_builder_types = [
                type_name for type_name in public_types if not type_name.endswith("Builder")
            ]
            if len(non_builder_types) > 1:
                violations.append(
                    f"单文件多个公开对象位于 `{relative}`："
                    + "、".join(f"`{name}`" for name in non_builder_types)
                )
        production_text = text.split("#[cfg(test)]", maxsplit=1)[0]
        if re.search(r"(?m)^\s*use\s+[^;]+::\*\s*;", production_text):
            violations.append(f"生产代码 wildcard import 位于 `{relative}`")
    return violations


def module_report(
    module: ModuleConfig,
    records: list[ObjectRecord],
    source_commit: str,
    *,
    canonical: str = "../迁移验收规范.md",
) -> str:
    """生成确定性的 Markdown 审计报告。"""

    counts = Counter(record.status for record in records)
    duplicate_fqns = [
        fqn for fqn, count in Counter(record.fqn for record in records).items() if count > 1
    ]
    if duplicate_fqns:
        raise ValueError("duplicate ledger rows: " + ", ".join(sorted(duplicate_fqns)))
    completed = sum(counts[status] for status in COMPLETE_STATUSES)
    lines = [
        f"<!-- migration-doc: authority=authoritative canonical={canonical} -->",
        f"# {module.name} 迁移事实审计",
        "",
        "> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。",
        f"> Spring 基线提交：`{source_commit}`；路径规则：保留末 `{module.retain_segments}` 层包目录。",
        "",
        CURRENT_CONTRACT_START,
        "## 当前迁移规范执行口径",
        "",
        "| 规范项 | 本模块强制要求 |",
        "|---|---|",
        f"| 来源基线 | `{source_commit}` |",
        f"| Java 对象边界 | {len(records)} 个 class/interface/enum/record；`package-info.java` 不计入 |",
        f"| 目录算法 | 去掉组织和模块根包，保留末 {module.retain_segments} 层包目录 |",
        "| 文件边界 | 一个 Java 对象对应一个 snake_case `.rs` 文件；内部类/Builder 可随主对象 |",
        "| 模块文件 | `lib.rs`/`mod.rs` 只允许模块文档、声明和显式重导出 |",
        "| 完成状态 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |",
        "| 未完成状态 | `MISSING`、`MISPLACED`、`STUB`、`PARTIAL`、`UNVERIFIED` |",
        "| 注释与测试 | 中文 Java 来源注释；正常、失败、边界和生命周期语义测试 |",
        "",
        "本文件顶部事实区始终按当前源码重新生成；下方历史设计附录不得覆盖这里的对象数量、路径、状态或证据。",
        CURRENT_CONTRACT_END,
        "",
        "## 汇总",
        "",
        "| 指标 | 数量 |",
        "|---|---:|",
        f"| Java 业务对象 | {len(records)} |",
        f"| 已处理（严格三类） | {completed} |",
    ]
    for status in sorted(ALL_STATUSES):
        lines.append(f"| `{status}` | {counts[status]} |")
    violations = validate_rust_red_lines(module)
    lines.extend(
        [
            "",
            "## 结构红线",
            "",
        ]
    )
    if violations:
        lines.extend(
            [
                "> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。",
                "",
                *(f"- {violation}" for violation in violations),
            ]
        )
    else:
        lines.append("- 未发现 `lib.rs`/`mod.rs` 类型定义或生产 wildcard import。")

    if module.dependencies:
        lines.extend(
            [
                "",
                "## 依赖复用边界",
                "",
                "| Crate | 固定提交 | Cargo 证据 | 精确符号 |",
                "|---|---|---|---|",
            ]
        )
        for dependency in module.dependencies:
            symbols = dependency.get("symbols", "未登记；不得判为 DEPENDENCY_REUSED")
            lines.append(
                "| `{}` | `{}` | `{}` | {} |".format(
                    dependency["crate"],
                    dependency["source_commit"],
                    dependency["manifest"],
                    symbols.replace("|", "\\|"),
                )
            )

    lines.extend(
        [
            "",
            "## 逐对象台账",
            "",
            "| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |",
            "|---|---|---|---|---|---|",
        ]
    )
    for record in records:
        lines.append(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | {} |".format(
                record.fqn,
                record.source_path,
                record.expected_path,
                record.actual_path,
                record.status,
                record.evidence.replace("|", "\\|"),
            )
        )
    lines.append("")
    return "\n".join(lines)


def compose_object_table(report: str, current_text: str = "") -> str:
    """将生成事实区与当前对象表中的历史设计附录合成为唯一权威文档。

    事实区每次从源码重算；附录只保存旧文档中仍有参考价值的分组、背景和决策，
    不得参与状态汇总。写报告时保留附录，避免重新生成时丢失人工迁移上下文。
    """

    has_start = OBJECT_TABLE_APPENDIX_START in current_text
    has_end = OBJECT_TABLE_APPENDIX_END in current_text
    if has_start != has_end:
        raise ValueError("object table historical appendix markers are incomplete")
    if not has_start:
        return report
    appendix = current_text.split(OBJECT_TABLE_APPENDIX_START, 1)[1]
    appendix = appendix.split(OBJECT_TABLE_APPENDIX_END, 1)[0].strip()
    return (
        report.rstrip()
        + "\n\n"
        + OBJECT_TABLE_APPENDIX_START
        + "\n"
        + appendix
        + "\n"
        + OBJECT_TABLE_APPENDIX_END
        + "\n"
    )


def authority_for_doc(
    relative: Path,
    audited_modules: frozenset[str] = frozenset(),
) -> tuple[str, str]:
    """为现有 Markdown 计算权威级别和规范链接。"""

    path = relative.as_posix()
    name = relative.name
    if len(relative.parts) >= 4 and relative.parts[1] == "history":
        return "historical", "../../对象级对照表.md"
    if path == "迁移验收规范.md" or path.startswith("migration-audit/"):
        return "authoritative", "../迁移验收规范.md" if "/" in path else "迁移验收规范.md"
    if "aspect-rs-参考/" in path:
        return "historical", "../../迁移验收规范.md"
    if path == "vernal-web-support/README.md":
        return "historical", "Spring-web-support-技术要求.md"
    current_names = {
        "对象名称一致性检查.md",
        "对象级对照表.md",
        "语义迁移对照表.md",
        "迁移路线图.md",
    }
    if len(relative.parts) == 2 and relative.parts[0].startswith("vernal-"):
        if name in current_names or (
            name.startswith("Spring-") and name.endswith("-技术要求.md")
        ):
            if name == "对象级对照表.md" and relative.parts[0] in audited_modules:
                return "authoritative", f"../migration-audit/{relative.parts[0]}.md"
            return "authoritative", "../迁移验收规范.md"
        legacy_kind = next(
            (
                current_name
                for current_name in current_names
                if current_name.removesuffix(".md") in name
            ),
            None,
        )
        if legacy_kind:
            return "historical", legacy_kind
    return "support", "迁移验收规范.md" if "/" not in path else "../迁移验收规范.md"


def validate_current_document_detail(
    path: Path,
    *,
    audited_object_table: bool,
) -> list[str]:
    """校验当前四件套不是只有标题和摘要的空壳文档。

    自动生成的对象表以“汇总、红线、逐对象台账”以及对象行完整性为详细依据；
    其余文档必须同时具备足够正文、多个二级章节和可核验的表格或任务项。
    """

    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    nonblank = [line for line in lines if line.strip()]
    h2_count = sum(1 for line in lines if re.match(r"^##\s+\S", line))
    table_rows = sum(
        1
        for line in lines
        if line.lstrip().startswith("|")
        and not re.match(r"^\s*\|[\s:|-]+\|\s*$", line)
    )
    action_items = sum(
        1
        for line in lines
        if re.match(r"^\s*(?:[-*]\s+(?:\[[ xX]\]\s+)?|\d+\.\s+)\S", line)
    )

    errors: list[str] = []
    relative = path.as_posix()
    if path.name == "对象级对照表.md" and audited_object_table:
        if h2_count < 3:
            errors.append(f"{relative}: object ledger requires at least 3 detailed sections")
        if table_rows < 5:
            errors.append(f"{relative}: object ledger has insufficient auditable rows")
        return errors

    if len(nonblank) < 45:
        errors.append(
            f"{relative}: current document is a short summary "
            f"({len(nonblank)} nonblank lines; requires at least 45)"
        )
    if h2_count < 3:
        errors.append(
            f"{relative}: current document requires at least 3 level-2 detail sections"
        )
    if table_rows < 5 and action_items < 8:
        errors.append(
            f"{relative}: current document lacks a detailed evidence table or task matrix"
        )
    return errors


def validate_document_sets(docs_root: Path, manifest: dict) -> list[str]:
    """校验每个 vernal 模块目录都有且只声明一套当前五件套。"""

    errors: list[str] = []
    configured_items = {
        Path(item["docs_dir"]).name: (key, item)
        for key in ("module", "documentation_only")
        for item in manifest.get(key, [])
    }
    configured = set(configured_items)
    actual = {
        path.name
        for path in docs_root.glob("vernal-*")
        if path.is_dir()
    }
    for name in sorted(actual - configured):
        errors.append(f"unregistered documentation module: docs/{name}")
    for name in sorted(configured - actual):
        errors.append(f"missing documentation directory: docs/{name}")

    for name in sorted(configured & actual):
        module_dir = docs_root / name
        conflicting_history = sorted(module_dir.glob("*-历史详细版.md"))
        for path in conflicting_history:
            current_name = path.name.replace("-历史详细版.md", ".md")
            errors.append(
                f"docs/{name}/{path.name}: historical detail must be merged into "
                f"{current_name} as an appendix instead of remaining a duplicate document"
            )
        duplicate_archives = [
            path
            for path in module_dir.glob("history/**/*.md")
            if path.name
            in {
                "对象级对照表.md",
                "aspect-rs-对象迁移旧蓝本.md",
                "多源对象迁移旧蓝本.md",
            }
        ]
        for path in sorted(duplicate_archives):
            errors.append(
                f"docs/{name}/{path.relative_to(module_dir)}: historical object ledger "
                "must be merged into 对象级对照表.md as a historical design appendix"
            )
        technical = list(module_dir.glob("Spring-*-技术要求.md"))
        if len(technical) != 1:
            errors.append(
                f"docs/{name}: expected exactly one Spring-*-技术要求.md, "
                f"found {len(technical)}"
            )
        else:
            head = "\n".join(
                technical[0].read_text(encoding="utf-8").splitlines()[:8]
            )
            marker = DOC_MARKER_PATTERN.search(head)
            if marker is None or marker.group(1) != "authoritative":
                errors.append(
                    f"docs/{name}/{technical[0].name}: "
                    "current document is not authoritative"
                )
        for filename in REQUIRED_CURRENT_DOCUMENTS:
            path = module_dir / filename
            if not path.is_file():
                errors.append(f"docs/{name}: missing current document {filename}")
                continue
            head = "\n".join(path.read_text(encoding="utf-8").splitlines()[:8])
            marker = DOC_MARKER_PATTERN.search(head)
            if marker is None or marker.group(1) != "authoritative":
                errors.append(f"docs/{name}/{filename}: current document is not authoritative")
                continue
            config_kind, _ = configured_items[name]
            document_text = path.read_text(encoding="utf-8")
            if (
                CURRENT_CONTRACT_START not in document_text
                or CURRENT_CONTRACT_END not in document_text
            ):
                errors.append(
                    f"docs/{name}/{filename}: missing current module migration contract"
                )
            errors.extend(
                validate_current_document_detail(
                    path,
                    audited_object_table=(
                        filename == "对象级对照表.md" and config_kind == "module"
                    ),
                )
            )
    return errors


def classify_documents(
    docs_root: Path,
    spring_commit: str,
    aspect_commit: str,
    audited_modules: frozenset[str],
) -> int:
    """给所有 Markdown 增加或更新文档治理标记。"""

    changed = 0
    for path in sorted(docs_root.rglob("*.md")):
        if path.parent.name == "migration-audit":
            continue
        relative = path.relative_to(docs_root)
        if (
            relative.name == "对象级对照表.md"
            and relative.parts[0] in audited_modules
        ):
            # 该文件与 migration-audit 报告同源生成，不能插入额外治理横幅。
            continue
        authority, canonical = authority_for_doc(relative, audited_modules)
        marker = (
            f"<!-- migration-doc: authority={authority} canonical={canonical} -->"
        )
        if authority == "historical":
            baseline = aspect_commit if "aspect-rs-参考/" in relative.as_posix() else spring_commit
            notice = (
                f"> 迁移文档治理：本文级别为 **historical**，历史基线提交 `{baseline}`。"
                f"正文不得作为当前验收结论；以 [{canonical}]({canonical}) 为准。"
            )
        else:
            notice = (
                f"> 迁移文档治理：本文级别为 **{authority}**。正文中的历史统计或完成标记"
                f"不得单独作为验收结论；以 [{canonical}]({canonical}) 和自动审计报告为准。"
            )
        text = path.read_text(encoding="utf-8")
        match = DOC_MARKER_PATTERN.search("\n".join(text.splitlines()[:6]))
        if match:
            lines = text.splitlines()
            for index, line in enumerate(lines[:6]):
                if DOC_MARKER_PATTERN.search(line):
                    lines[index] = marker
                    if index + 2 < len(lines) and lines[index + 2].startswith(
                        "> 迁移文档治理："
                    ):
                        lines[index + 2] = notice
                    else:
                        lines[index + 2:index + 2] = [notice, ""]
                    text = "\n".join(lines) + ("\n" if text.endswith("\n") else "")
                    break
        else:
            text = f"{marker}\n\n{notice}\n\n{text}"
        path.write_text(text, encoding="utf-8")
        changed += 1
    return changed


def validate_doc_markers(docs_root: Path) -> list[str]:
    """校验全部 Markdown 都有有效的治理标记。"""

    errors: list[str] = []
    for path in sorted(docs_root.rglob("*.md")):
        head = "\n".join(path.read_text(encoding="utf-8").splitlines()[:8])
        match = DOC_MARKER_PATTERN.search(head)
        if not match:
            errors.append(f"missing migration-doc marker: {path.relative_to(docs_root)}")
            continue
        canonical = match.group(2)
        canonical_path = (path.parent / canonical).resolve()
        if not canonical_path.exists():
            errors.append(
                f"broken canonical link: {path.relative_to(docs_root)} -> {canonical}"
            )
    return errors


def git_head(path: Path) -> str:
    """读取 Git HEAD；失败时返回 unknown。"""

    try:
        return subprocess.check_output(
            ["git", "-C", str(path), "rev-parse", "HEAD"],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def selected_modules(modules: Iterable[ModuleConfig], names: list[str]) -> list[ModuleConfig]:
    """按 CLI 参数选择模块。"""

    module_list = list(modules)
    if not names:
        return module_list
    by_name = {module.name: module for module in module_list}
    missing = [name for name in names if name not in by_name]
    if missing:
        raise ValueError(f"unknown module(s): {', '.join(missing)}")
    return [by_name[name] for name in names]


def main(argv: list[str] | None = None) -> int:
    """命令行入口。"""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", default="docs/migration-manifest.toml")
    parser.add_argument("--module", action="append", default=[])
    parser.add_argument("--all", action="store_true", help="审计清单中的全部模块")
    parser.add_argument("--check", action="store_true", help="检查报告与文档标记")
    parser.add_argument("--write-reports", action="store_true", help="写入生成报告")
    parser.add_argument("--classify-docs", action="store_true", help="更新全部文档治理标记")
    args = parser.parse_args(argv)

    repo_root = Path(__file__).resolve().parents[1]
    manifest_path = (repo_root / args.manifest).resolve()
    manifest, modules = read_manifest(repo_root, manifest_path)
    chosen = selected_modules(modules, args.module)
    if not args.all and not args.module:
        parser.error("provide --module NAME or --all")

    if args.classify_docs:
        changed = classify_documents(
            repo_root / "docs",
            manifest["spring_commit"],
            manifest["aspect_rs_commit"],
            frozenset(module.name for module in modules),
        )
        print(f"classified {changed} Markdown documents")

    report_dir = repo_root / "docs" / "migration-audit"
    if args.write_reports:
        report_dir.mkdir(parents=True, exist_ok=True)

    errors: list[str] = []
    for module in chosen:
        records = audit_module(repo_root, module)
        report = module_report(module, records, module.source_commit)
        report_path = report_dir / f"{module.name}.md"
        if args.write_reports:
            report_path.write_text(report, encoding="utf-8")
            if module.object_table is not None:
                generated_object_table = module_report(
                    module,
                    records,
                    module.source_commit,
                    canonical=f"../migration-audit/{module.name}.md",
                )
                current_text = (
                    module.object_table.read_text(encoding="utf-8")
                    if module.object_table.exists()
                    else ""
                )
                object_table = compose_object_table(
                    generated_object_table,
                    current_text,
                )
                module.object_table.write_text(object_table, encoding="utf-8")
        elif args.check:
            if not report_path.exists():
                errors.append(f"missing generated report: {report_path.relative_to(repo_root)}")
            elif report_path.read_text(encoding="utf-8") != report:
                errors.append(f"stale generated report: {report_path.relative_to(repo_root)}")
            if module.object_table is not None:
                generated_object_table = module_report(
                    module,
                    records,
                    module.source_commit,
                    canonical=f"../migration-audit/{module.name}.md",
                )
                if not module.object_table.exists():
                    errors.append(
                        "missing generated object table: "
                        f"{module.object_table.relative_to(repo_root)}"
                    )
                else:
                    current_text = module.object_table.read_text(encoding="utf-8")
                    try:
                        expected_object_table = compose_object_table(
                            generated_object_table,
                            current_text,
                        )
                    except ValueError as error:
                        errors.append(
                            f"invalid object table appendix: "
                            f"{module.object_table.relative_to(repo_root)}: {error}"
                        )
                    else:
                        if current_text != expected_object_table:
                            errors.append(
                                "stale generated object table: "
                                f"{module.object_table.relative_to(repo_root)}"
                            )
        else:
            print(report)

    if args.check:
        spring_root = (repo_root / "../spring-framework").resolve()
        aspect_root = (repo_root / "../aspect-rs").resolve()
        if git_head(spring_root) != manifest["spring_commit"]:
            errors.append("spring-framework HEAD does not match migration manifest")
        if git_head(aspect_root) != manifest["aspect_rs_commit"]:
            errors.append("aspect-rs HEAD does not match migration manifest")
        errors.extend(validate_doc_markers(repo_root / "docs"))
        errors.extend(validate_document_sets(repo_root / "docs", manifest))

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    if args.check:
        print(f"migration audit passed: modules={len(chosen)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
