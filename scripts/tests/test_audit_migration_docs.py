"""迁移文档审计器单元测试。"""

from __future__ import annotations

import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "audit_migration_docs.py"
SPEC = importlib.util.spec_from_file_location("audit_migration_docs", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)


class NamingTests(unittest.TestCase):
    """验证 Java→Rust 命名与目录规则。"""

    def test_acronyms_convert_to_snake_case(self) -> None:
        self.assertEqual(AUDIT.camel_to_snake("URIEditor"), "uri_editor")
        self.assertEqual(AUDIT.camel_to_snake("AotProcessor"), "aot_processor")
        self.assertEqual(AUDIT.camel_to_snake("XMLReader"), "xml_reader")

    def test_retain_last_two_package_segments(self) -> None:
        source = Path("factory/xml/support/FooBar.java")
        self.assertEqual(
            AUDIT.expected_rust_path(source, 2),
            Path("xml/support/foo_bar.rs"),
        )

    def test_retain_single_segment_when_only_one_exists(self) -> None:
        source = Path("propertyeditors/PatternEditor.java")
        self.assertEqual(
            AUDIT.expected_rust_path(source, 2),
            Path("propertyeditors/pattern_editor.rs"),
        )

    def test_root_object_stays_at_root(self) -> None:
        self.assertEqual(
            AUDIT.expected_rust_path(Path("BeansException.java"), 2),
            Path("beans_exception.rs"),
        )


class SourceInspectionTests(unittest.TestCase):
    """验证 stub、mod.rs 和状态识别所需的基础逻辑。"""

    def test_stub_markers(self) -> None:
        self.assertTrue(AUDIT.has_stub_marker("fn run() { todo!() }"))
        self.assertTrue(AUDIT.has_stub_marker("// 实际实现需要清理缓存"))
        self.assertFalse(AUDIT.has_stub_marker("fn run() { perform_cleanup(); }"))

    def test_production_files_exclude_module_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "lib.rs").write_text("pub mod sample;\n", encoding="utf-8")
            (root / "mod.rs").write_text("pub mod sample;\n", encoding="utf-8")
            (root / "sample.rs").write_text("pub struct Sample;\n", encoding="utf-8")
            self.assertEqual(
                [path.name for path in AUDIT.production_rust_files(root)],
                ["sample.rs"],
            )

    def test_java_inventory_excludes_package_info(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Foo.java").write_text("class Foo {}", encoding="utf-8")
            (root / "package-info.java").write_text("/** docs */", encoding="utf-8")
            self.assertEqual(
                [path.name for path in AUDIT.java_object_files(root)],
                ["Foo.java"],
            )

    def test_public_type_pattern_detects_mod_rs_violation(self) -> None:
        self.assertIsNotNone(AUDIT.PUBLIC_TYPE_PATTERN.search("pub struct Wrong;\n"))
        self.assertIsNone(AUDIT.PUBLIC_TYPE_PATTERN.search("struct PrivateHelper;\n"))
        self.assertIsNone(AUDIT.PUBLIC_TYPE_PATTERN.search("pub mod correct;\n"))


class StatusTests(unittest.TestCase):
    """验证完成状态的严格三类口径。"""

    def test_only_three_statuses_are_complete(self) -> None:
        self.assertEqual(
            AUDIT.COMPLETE_STATUSES,
            {"IMPLEMENTED", "DEPENDENCY_REUSED", "PLATFORM_NA"},
        )
        self.assertTrue(
            {"MISSING", "MISPLACED", "STUB", "PARTIAL", "UNVERIFIED"}
            <= AUDIT.INCOMPLETE_STATUSES
        )

    def test_dependency_reuse_requires_registered_symbol_and_test(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "java"
            target = root / "crate" / "src"
            test_file = root / "crate" / "tests" / "reuse.rs"
            source.mkdir()
            target.mkdir(parents=True)
            test_file.parent.mkdir()
            (source / "Advice.java").write_text("interface Advice {}", encoding="utf-8")
            test_file.write_text("#[test] fn reuse() {}", encoding="utf-8")
            module = AUDIT.ModuleConfig(
                name="sample",
                source_root=source,
                source_package="example",
                source_commit="abc",
                source_prefixes=(),
                target_root=target,
                target_optional=False,
                docs_dir=root,
                retain_segments=2,
                object_table=None,
                dependencies=(
                    {
                        "crate": "aspect-core",
                        "symbols": "aspect_core::Aspect",
                        "source_commit": "abc",
                        "manifest": "Cargo.toml",
                    },
                ),
                dispositions=(
                    {
                        "java_fqn": "example.Advice",
                        "status": "DEPENDENCY_REUSED",
                        "crate": "aspect-core",
                        "symbol": "aspect_core::Aspect",
                        "integration_test": str(test_file.relative_to(root)),
                    },
                ),
            )
            records = AUDIT.audit_module(root, module)
            self.assertEqual(records[0].status, "DEPENDENCY_REUSED")

    def test_platform_na_requires_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "java"
            target = root / "crate" / "src"
            source.mkdir()
            target.mkdir(parents=True)
            (source / "JvmProxy.java").write_text("class JvmProxy {}", encoding="utf-8")
            module = AUDIT.ModuleConfig(
                name="sample",
                source_root=source,
                source_package="example",
                source_commit="abc",
                source_prefixes=(),
                target_root=target,
                target_optional=False,
                docs_dir=root,
                retain_segments=2,
                object_table=None,
                dependencies=(),
                dispositions=(
                    {
                        "java_fqn": "example.JvmProxy",
                        "status": "PLATFORM_NA",
                        "platform_evidence": "依赖 JVM 字节码生成。",
                    },
                ),
            )
            records = AUDIT.audit_module(root, module)
            self.assertEqual(records[0].status, "PLATFORM_NA")

    def test_missing_optional_target_yields_missing_records(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "java"
            source.mkdir()
            (source / "Foo.java").write_text("class Foo {}", encoding="utf-8")
            module = AUDIT.ModuleConfig(
                name="sample",
                source_root=source,
                source_package="example",
                source_commit="abc",
                source_prefixes=(),
                target_root=root / "missing-crate" / "src",
                target_optional=True,
                docs_dir=root,
                retain_segments=2,
                object_table=None,
                dependencies=(),
                dispositions=(),
            )
            records = AUDIT.audit_module(root, module)
            self.assertEqual(records[0].status, "MISSING")

    def test_duplicate_ledger_rows_are_rejected(self) -> None:
        module = AUDIT.ModuleConfig(
            name="sample",
            source_root=Path("."),
            source_package="example",
            source_commit="abc",
            source_prefixes=(),
            target_root=Path("."),
            target_optional=False,
            docs_dir=Path("."),
            retain_segments=2,
            object_table=None,
            dependencies=(),
            dispositions=(),
        )
        record = AUDIT.ObjectRecord(
            fqn="example.Foo",
            source_path="Foo.java",
            expected_path="foo.rs",
            actual_path="foo.rs",
            status="IMPLEMENTED",
            evidence="test",
        )
        with self.assertRaisesRegex(ValueError, "duplicate ledger rows"):
            AUDIT.module_report(module, [record, record], "abc")

    def test_generated_object_table_preserves_historical_design_appendix(self) -> None:
        report = "# 当前事实\n"
        current = (
            "# 旧事实\n\n"
            f"{AUDIT.OBJECT_TABLE_APPENDIX_START}\n"
            "## 历史设计附录\n\n旧包分组说明。\n"
            f"{AUDIT.OBJECT_TABLE_APPENDIX_END}\n"
        )
        merged = AUDIT.compose_object_table(report, current)
        self.assertTrue(merged.startswith(report))
        self.assertIn("旧包分组说明", merged)
        self.assertNotIn("# 旧事实", merged)

    def test_incomplete_historical_design_appendix_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "markers are incomplete"):
            AUDIT.compose_object_table(
                "# 当前事实\n",
                AUDIT.OBJECT_TABLE_APPENDIX_START + "\n未闭合\n",
            )


class DocumentationSetTests(unittest.TestCase):
    """验证每个模块必须具备唯一的当前五件套。"""

    def test_current_and_legacy_documents_have_distinct_authority(self) -> None:
        audited = frozenset({"vernal-sample"})
        self.assertEqual(
            AUDIT.authority_for_doc(
                Path("vernal-sample/对象级对照表.md"),
                audited,
            ),
            ("authoritative", "../migration-audit/vernal-sample.md"),
        )
        self.assertEqual(
            AUDIT.authority_for_doc(
                Path("vernal-sample/spring到vernal对象级对照表.md"),
                audited,
            ),
            ("historical", "对象级对照表.md"),
        )
        self.assertEqual(
            AUDIT.authority_for_doc(
                Path("vernal-sample/history/2026-07-30-dd20300d/对象级对照表.md"),
                audited,
            ),
            ("historical", "../../对象级对照表.md"),
        )

    def test_document_set_reports_missing_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            module_dir = docs / "vernal-sample"
            module_dir.mkdir()
            (module_dir / "Spring-sample-技术要求.md").write_text(
                "<!-- migration-doc: authority=authoritative "
                "canonical=../迁移验收规范.md -->\n",
                encoding="utf-8",
            )
            manifest = {
                "module": [{"docs_dir": "docs/vernal-sample"}],
                "documentation_only": [],
            }
            errors = AUDIT.validate_document_sets(docs, manifest)
            self.assertEqual(len(errors), 4)
            self.assertTrue(all("missing current document" in error for error in errors))

    def test_short_summary_documents_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            module_dir = docs / "vernal-sample"
            module_dir.mkdir()
            marker = (
                "<!-- migration-doc: authority=authoritative "
                "canonical=../迁移验收规范.md -->\n"
            )
            (module_dir / "Spring-sample-技术要求.md").write_text(
                marker,
                encoding="utf-8",
            )
            for filename in AUDIT.REQUIRED_CURRENT_DOCUMENTS:
                (module_dir / filename).write_text(
                    marker + f"# {filename}\n\n只有一句摘要。\n",
                    encoding="utf-8",
                )
            manifest = {
                "module": [{"docs_dir": "docs/vernal-sample"}],
                "documentation_only": [],
            }
            errors = AUDIT.validate_document_sets(docs, manifest)
            detail_errors = [
                error
                for error in errors
                if "short summary" in error
                or "detailed sections" in error
                or "auditable rows" in error
            ]
            self.assertGreaterEqual(len(detail_errors), 4)

    def test_root_historical_detail_conflicts_with_authoritative_set(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            module_dir = docs / "vernal-sample"
            module_dir.mkdir()
            marker = (
                "<!-- migration-doc: authority=authoritative "
                "canonical=../迁移验收规范.md -->\n"
            )
            (module_dir / "Spring-sample-技术要求.md").write_text(
                marker,
                encoding="utf-8",
            )
            for filename in AUDIT.REQUIRED_CURRENT_DOCUMENTS:
                (module_dir / filename).write_text(
                    marker + f"# {filename}\n",
                    encoding="utf-8",
                )
            (module_dir / "对象级对照表-历史详细版.md").write_text(
                "# 历史表\n",
                encoding="utf-8",
            )
            manifest = {
                "module": [{"docs_dir": "docs/vernal-sample"}],
                "documentation_only": [],
            }
            errors = AUDIT.validate_document_sets(docs, manifest)
            self.assertTrue(
                any("must be merged into 对象级对照表.md" in error for error in errors)
            )

    def test_archived_object_ledger_must_be_merged_into_current_table(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            module_dir = docs / "vernal-sample"
            archive = module_dir / "history" / "baseline"
            archive.mkdir(parents=True)
            marker = (
                "<!-- migration-doc: authority=authoritative "
                "canonical=../迁移验收规范.md -->\n"
            )
            (module_dir / "Spring-sample-技术要求.md").write_text(
                marker,
                encoding="utf-8",
            )
            for filename in AUDIT.REQUIRED_CURRENT_DOCUMENTS:
                (module_dir / filename).write_text(
                    marker + f"# {filename}\n",
                    encoding="utf-8",
                )
            (archive / "对象级对照表.md").write_text(
                "# 历史对象表\n",
                encoding="utf-8",
            )
            manifest = {
                "module": [{"docs_dir": "docs/vernal-sample"}],
                "documentation_only": [],
            }
            errors = AUDIT.validate_document_sets(docs, manifest)
            self.assertTrue(
                any("must be merged into 对象级对照表.md" in error for error in errors)
            )

    def test_detailed_document_contract_accepts_evidence_matrix(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "语义迁移对照表.md"
            body = [
                "# 语义迁移对照表",
                "## 语义边界",
                "## 调用链",
                "## 测试矩阵",
                "| 场景 | 证据 |",
                "|---|---|",
            ]
            body.extend(f"| 场景 {index} | 证据 {index} |" for index in range(8))
            body.extend(f"- 详细说明 {index}" for index in range(40))
            path.write_text("\n".join(body) + "\n", encoding="utf-8")
            self.assertEqual(
                AUDIT.validate_current_document_detail(
                    path,
                    audited_object_table=False,
                ),
                [],
            )

    def test_current_four_documents_require_module_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            module_dir = docs / "vernal-sample"
            module_dir.mkdir()
            marker = (
                "<!-- migration-doc: authority=authoritative "
                "canonical=../迁移验收规范.md -->\n"
            )
            (module_dir / "Spring-sample-技术要求.md").write_text(
                marker,
                encoding="utf-8",
            )
            detailed = (
                marker
                + "# 当前文档\n"
                + "## 范围\n## 证据\n## 验收\n"
                + "| 场景 | 证据 |\n|---|---|\n"
                + "".join(f"| 场景 {index} | 证据 {index} |\\n" for index in range(8))
                + "".join(f"- 说明 {index}\\n" for index in range(45))
            )
            for filename in AUDIT.REQUIRED_CURRENT_DOCUMENTS:
                (module_dir / filename).write_text(detailed, encoding="utf-8")
            manifest = {
                "module": [{"docs_dir": "docs/vernal-sample"}],
                "documentation_only": [],
            }
            errors = AUDIT.validate_document_sets(docs, manifest)
            contract_errors = [
                error for error in errors if "missing current module migration contract" in error
            ]
            self.assertEqual(len(contract_errors), 4)

    def test_classification_does_not_mutate_generated_object_table(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            module_dir = docs / "vernal-sample"
            module_dir.mkdir()
            table = module_dir / "对象级对照表.md"
            generated = (
                "<!-- migration-doc: authority=authoritative "
                "canonical=../migration-audit/vernal-sample.md -->\n"
                "# generated\n"
            )
            table.write_text(generated, encoding="utf-8")
            AUDIT.classify_documents(
                docs,
                "spring",
                "aspect",
                frozenset({"vernal-sample"}),
            )
            self.assertEqual(table.read_text(encoding="utf-8"), generated)


if __name__ == "__main__":
    unittest.main()
