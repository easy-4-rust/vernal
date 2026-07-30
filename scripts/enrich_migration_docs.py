#!/usr/bin/env python3
"""为当前迁移四件套补充可审计的模块级详细内容。

脚本只补充未达到 ``audit_migration_docs`` 详细度门禁的当前文档，并使用固定
标记保证重复执行不会重复追加。对象事实来自同一审计器，不手写完成数量。
"""

from __future__ import annotations

import importlib.util
import sys
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AUDIT_PATH = ROOT / "scripts" / "audit_migration_docs.py"
SPEC = importlib.util.spec_from_file_location("migration_audit_for_enrichment", AUDIT_PATH)
assert SPEC is not None and SPEC.loader is not None
AUDIT = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUDIT
SPEC.loader.exec_module(AUDIT)

PROFILES = {
    "vernal-aop": ["统一 Advice 合同", "MethodInterceptor 调用链", "Advisor 与 Pointcut", "代理创建与自动代理", "before/after/error 顺序", "aspect-rs 精确复用边界", "JVM 代理平台边界"],
    "vernal-aspects": ["事务切面", "缓存切面", "异步切面", "可配置对象切面", "注解启用桥接", "编译期织入边界"],
    "vernal-beans": ["Bean 定义与注册", "BeanFactory 获取链", "实例化与依赖注入", "PostProcessor 顺序", "初始化与销毁", "类型转换与属性访问", "Scope 与循环依赖"],
    "vernal-context": ["ApplicationContext 生命周期", "事件发布与监听", "资源和消息解析", "注解配置处理", "组件扫描与索引", "环境和属性源", "BeanFactory 协作边界"],
    "vernal-context-support": ["缓存集成", "调度与 Quartz", "邮件抽象", "模板与 UI 支持", "依赖组件适配", "资源生命周期", "跨模块配置"],
    "vernal-expression": ["表达式解析", "AST 求值", "类型转换", "属性与方法访问", "运算符语义", "求值上下文", "错误位置和诊断"],
    "vernal-messaging": ["Message 与 Header", "Channel 与发送接收", "消息转换", "Handler 调用链", "目标解析和路由", "异步与 Reactive 流", "协议适配"],
    "vernal-tx": ["事务定义和属性", "TransactionManager", "TransactionStatus", "同步回调顺序", "传播与隔离", "提交回滚和保存点", "事务拦截器", "DAO 异常体系"],
    "vernal-core": ["资源定位与关闭", "类型转换", "元数据访问", "I/O 与编码", "并发上下文", "错误因果链"],
    "vernal-core-test": ["测试缓冲与流", "反射断言", "资源夹具", "并发协调", "失败诊断", "跨平台确定性"],
    "vernal-async": ["任务提交与句柄", "同步/异步执行器", "超时取消与拒绝", "装饰器顺序", "上下文传播", "虚拟线程边界"],
    "vernal-cache": ["缓存与管理器", "读写淘汰清空", "键与条件", "并发加载", "注解拦截", "依赖复用边界"],
    "vernal-context-indexer": ["候选组件索引", "stereotype 元数据", "编译期扫描", "索引格式", "合并与稳定排序", "运行时加载"],
    "vernal-instrument": ["JVM 转换专属性", "Rust 编译期替代", "启动钩子", "动态库边界", "可观测性接入", "PLATFORM_NA 证据"],
    "vernal-jdbc": ["连接生命周期", "执行模板", "参数绑定", "结果映射", "批处理", "异常翻译", "事务同步", "LOB 与元数据"],
    "vernal-log": ["延迟消息", "复合级别委派", "结构化字段", "tracing 桥接", "格式限制", "错误链记录"],
    "vernal-orm": ["会话生命周期", "异常转换", "事务桥接", "实体回调", "查询映射", "依赖隔离"],
    "vernal-oxm": ["编解组合同", "输入输出源", "命名空间编码", "XXE 安全", "对象图错误", "流式关闭"],
    "vernal-r2dbc": ["连接工厂", "DatabaseClient", "异步绑定", "行流背压取消", "批处理", "异常翻译", "事务上下文"],
    "vernal-rbatis": ["executor 适配", "事务状态", "拦截器顺序", "错误翻译", "分页", "缓存桥接", "连接释放"],
    "vernal-rbdc": ["驱动注册", "连接生命周期", "连接池", "Statement 绑定", "结果行流", "保存点", "错误翻译"],
    "vernal-test": ["TestContext", "引导器", "监听器顺序", "上下文缓存", "Mock HTTP", "测试事务", "异步超时", "失败诊断"],
    "vernal-web": ["HTTP 抽象", "媒体协商", "URI 编码", "multipart", "客户端拦截", "消息编解码", "错误映射", "运行时边界"],
    "vernal-web-support": ["HandlerMapping 合同", "HandlerAdapter 合同", "参数与返回扩展点", "过滤器短路", "资源模板", "运行时适配", "错误响应", "契约测试"],
    "vernal-webflux": ["异步分发", "映射与适配", "参数与返回值", "Codec", "背压取消", "过滤器顺序", "异常响应", "WebClient"],
    "vernal-webmvc": ["请求分发", "映射与适配", "参数解析", "返回值视图", "拦截器顺序", "异常链", "绑定验证", "静态资源"],
    "vernal-websocket": ["Upgrade 握手", "会话生命周期", "帧语义", "背压取消", "子协议", "STOMP", "关闭状态", "SockJS 边界"],
}


def escape(value: object) -> str:
    """转义 Markdown 表格单元格。"""

    return str(value).replace("|", "\\|").replace("\n", " ")


def object_name(record: object) -> str:
    """取得对象简单名称。"""

    return record.fqn.rsplit(".", 1)[-1]


def object_group(record: object) -> str:
    """按预期 Rust 目录聚合对象。"""

    parent = Path(record.expected_path).parent.as_posix()
    return "crate 根目录" if parent == "." else parent


def synthetic_records(name: str) -> list[object]:
    """为没有 Spring 源码根的文档型模块建立显式规划对象。"""

    definitions = {
        "vernal-log": [
            ("org.springframework.core.log.CompositeLog", "core/log/composite_log.rs"),
            ("org.springframework.core.log.LogAccessor", "core/log/log_accessor.rs"),
            ("org.springframework.core.log.LogDelegateFactory", "core/log/log_delegate_factory.rs"),
            ("org.springframework.core.log.LogFormatUtils", "core/log/log_format_utils.rs"),
            ("org.springframework.core.log.LogMessage", "core/log/log_message.rs"),
        ],
        "vernal-web-support": [
            ("SharedHandlerMapping", "mapping/shared_handler_mapping.rs"),
            ("SharedHandlerAdapter", "adapter/shared_handler_adapter.rs"),
            ("ArgumentResolverContract", "method/argument_resolver_contract.rs"),
            ("ReturnValueHandlerContract", "method/return_value_handler_contract.rs"),
            ("RuntimeFilterAdapter", "filter/runtime_filter_adapter.rs"),
            ("ResponseCommitGuard", "response/response_commit_guard.rs"),
            ("StaticResourceSupport", "resource/static_resource_support.rs"),
            ("TemplateRenderSupport", "view/template_render_support.rs"),
            ("RuntimeErrorMapper", "error/runtime_error_mapper.rs"),
            ("WebRuntimeTestContract", "testing/web_runtime_test_contract.rs"),
        ],
        "vernal-rbatis": [
            ("RbatisExecutorAdapter", "executor/rbatis_executor_adapter.rs"),
            ("RbatisTransactionManager", "transaction/rbatis_transaction_manager.rs"),
            ("RbatisTransactionStatus", "transaction/rbatis_transaction_status.rs"),
            ("RbatisInterceptorAdapter", "intercept/rbatis_interceptor_adapter.rs"),
            ("RbatisErrorTranslator", "support/rbatis_error_translator.rs"),
            ("RbatisPageAdapter", "page/rbatis_page_adapter.rs"),
            ("RbatisCacheAdapter", "cache/rbatis_cache_adapter.rs"),
            ("RbatisConnectionFactory", "connection/rbatis_connection_factory.rs"),
        ],
        "vernal-rbdc": [
            ("RbdcDriverAdapter", "driver/rbdc_driver_adapter.rs"),
            ("RbdcConnectionFactory", "connection/rbdc_connection_factory.rs"),
            ("RbdcConnectionAdapter", "connection/rbdc_connection_adapter.rs"),
            ("RbdcPoolAdapter", "pool/rbdc_pool_adapter.rs"),
            ("RbdcStatementAdapter", "statement/rbdc_statement_adapter.rs"),
            ("RbdcResultAdapter", "result/rbdc_result_adapter.rs"),
            ("RbdcErrorTranslator", "support/rbdc_error_translator.rs"),
        ],
    }
    status = "MISSING" if name == "vernal-log" else "UNVERIFIED"
    return [
        AUDIT.ObjectRecord(
            fqn=fqn,
            source_path="规划对象（非自动 Spring 源对象）",
            expected_path=path,
            actual_path="—",
            status=status,
            evidence="尚无满足严格验收规范的本地实现与测试证据",
        )
        for fqn, path in definitions[name]
    ]


def status_table(records: list[object]) -> list[str]:
    """生成严格状态汇总表。"""

    counts = Counter(record.status for record in records)
    lines = ["| 状态 | 数量 | 计入完成 |", "|---|---:|---|"]
    for status in sorted(AUDIT.ALL_STATUSES | set(counts)):
        complete = "是" if status in AUDIT.COMPLETE_STATUSES else "否"
        lines.append(f"| `{status}` | {counts[status]} | {complete} |")
    return lines


def current_contract(
    name: str,
    filename: str,
    records: list[object],
    concerns: list[str],
    config: object | None,
) -> str:
    """生成当前四件套各自的模块化新规范执行块。"""

    counts = Counter(record.status for record in records)
    source = (
        f"`{config.source_package}` @ `{config.source_commit}`"
        if config is not None
        else "文档型/依赖适配模块；以显式规划对象和固定依赖证据为边界"
    )
    target = (
        config.target_root.relative_to(ROOT).as_posix()
        if config is not None
        and config.target_root.is_relative_to(ROOT)
        else f"docs/{name} 中声明的目标 crate/适配层"
    )
    lines = [
        "## 当前迁移规范执行口径", "",
        "| 规范项 | 本模块当前要求 |", "|---|---|",
        f"| 模块 | `{name}` |",
        f"| 来源基线 | {source} |",
        f"| 当前对象边界 | {len(records)} 个对象；不把 `package-info.java`、合并对象或模糊依赖豁免计入完成 |",
        f"| 目标根 | `{target}` |",
        "| 目录和文件 | 去掉组织及模块根包，保留末两层；目录/文件 snake_case，一个源对象一个 `.rs` 文件 |",
        "| 类型和方法 | 类型 PascalCase；方法和参数 snake_case，名称与 Java 语义一一对应 |",
        "| 模块文件 | `lib.rs`/`mod.rs` 只含模块文档、声明和显式重导出 |",
        "| 完成口径 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |",
        "| 红线 | 禁止 stub、兼容文件转引充数、生产 wildcard import、无证据的依赖替代 |",
        "| 注释和测试 | 中文来源注释；正常、失败、边界、顺序、生命周期和并发/取消测试 |",
        "", "### 当前状态快照", "",
        "| 状态 | 数量 |", "|---|---:|",
    ]
    for status in sorted(AUDIT.ALL_STATUSES | set(counts)):
        lines.append(f"| `{status}` | {counts[status]} |")

    lines += ["", "### 本文档执行责任", ""]
    if filename == "对象名称一致性检查.md":
        lines += [
            "- 必须逐对象核对 Java 名称、snake_case 文件名、末两层目录、当前路径和公开主类型。",
            "- 同名但目录不符一律标为 `MISPLACED`；文件存在但注释或测试不足标为 `UNVERIFIED`。",
            "- 不能用“已合并”“已改名”或兼容重导出消除应有对象文件。",
        ]
    elif filename == "对象级对照表.md":
        lines += [
            "- 必须覆盖全部对象的 FQN、源路径、预期路径、当前路径、状态和可定位证据。",
            "- 顶部当前事实优先于历史设计附录；附录中的旧统计和旧完成标记不参与验收。",
            "- 依赖复用必须记录 crate、固定提交、精确符号和本地集成测试。",
        ]
    elif filename == "语义迁移对照表.md":
        lines += [
            f"- 本模块必须覆盖：{'、'.join(concerns)}。",
            "- 必须记录入口、协作对象、回调顺序、错误传播、资源释放以及异步取消/背压语义。",
            "- 接口相似不能代替行为证据；每个关键语义域都要有正常、失败和边界测试。",
        ]
    else:
        lines += [
            "- 实施顺序固定为 `MISPLACED` → `MISSING` → `STUB/PARTIAL` → `UNVERIFIED`。",
            f"- 当前优先债务：MISPLACED={counts['MISPLACED']}、MISSING={counts['MISSING']}、"
            f"STUB={counts['STUB']}、PARTIAL={counts['PARTIAL']}、UNVERIFIED={counts['UNVERIFIED']}。",
            "- 每批必须绑定对象清单、目录调整、语义测试、审计命令和回退条件。",
        ]
    return "\n".join(lines)


def grouped(records: list[object]) -> dict[str, list[object]]:
    """按目标目录稳定分组。"""

    result: dict[str, list[object]] = defaultdict(list)
    for record in records:
        result[object_group(record)].append(record)
    return dict(sorted(result.items()))


def naming_detail(name: str, records: list[object], _: list[str]) -> str:
    """生成逐对象名称和路径核验。"""

    lines = [
        f"## {name} 命名审计范围", "",
        "本检查以对象边界和预期 Rust 路径为唯一结构基准；类型同名但目录错误仍是 `MISPLACED`。",
        "",
        "| 维度 | 强制规则 | 失败判定 |", "|---|---|---|",
        "| 文件 | PascalCase 转 snake_case | `MISSING`/`MISPLACED` |",
        "| 目录 | 只保留源包末两层 | `MISPLACED` |",
        "| 主类型 | 一个文件对应一个源对象 | `UNVERIFIED` |",
        "| 方法参数 | lowerCamelCase 转 snake_case，语义不缩写 | `PARTIAL` |",
        "| 注释 | 中文用途、Java FQN 和公开方法来源 | `UNVERIFIED` |",
        "| 依赖 | crate、提交、符号、集成测试缺一不可 | 未完成 |",
        "", "## 状态汇总", "", *status_table(records),
        "", "## 目标目录分布", "",
        "| 目标目录 | 对象数 | 代表对象 | 约束 |", "|---|---:|---|---|",
    ]
    for directory, items in grouped(records).items():
        samples = "、".join(f"`{object_name(item)}`" for item in items[:5])
        lines.append(f"| `{escape(directory)}` | {len(items)} | {samples} | `mod.rs` 仅声明和重导出 |")
    lines += [
        "", "## 逐对象命名判定", "",
        "| 对象 | 预期 Rust 文件 | 当前文件/依赖 | 判定 | 证据 |",
        "|---|---|---|---|---|",
    ]
    for record in records:
        if record.status in {"DEPENDENCY_REUSED", "PLATFORM_NA"}:
            verdict = record.status
        elif record.actual_path == "—":
            verdict = "MISSING"
        elif record.actual_path == record.expected_path:
            verdict = "PATH_MATCH"
        else:
            verdict = "MISPLACED"
        lines.append(
            f"| `{escape(record.fqn)}` | `{escape(record.expected_path)}` | "
            f"`{escape(record.actual_path)}` | `{verdict}` | {escape(record.evidence)} |"
        )
    lines += [
        "", "## 修复与复核顺序", "",
        "1. 先处理 `MISPLACED`，同步目录模块声明和所有引用。",
        "2. 再补 `MISSING`；文件必须承载真实逻辑，不能转引兼容文件充数。",
        "3. 对 `UNVERIFIED` 补公开主类型、中文来源注释和测试证据。",
        "4. 依赖复用逐项核对精确符号，功能相似不能豁免对象。",
        "5. 重跑审计；当前路径与预期路径不同即保持未完成。",
        "", "## 命名验收清单", "",
        "- [ ] 目录、文件、方法和参数均为 snake_case。",
        "- [ ] 类型为 PascalCase，且语义名称与源对象一致。",
        "- [ ] 每个业务文件只有一个源对象。",
        "- [ ] `lib.rs`/`mod.rs` 不定义业务类型。",
        "- [ ] 生产代码无 wildcard import。",
        "- [ ] 公开对象和方法具有中文来源注释。",
        "- [ ] 内部类或 Builder 仅随主对象同文件。",
        "- [ ] 台账无重复、模糊合并或无证据豁免。",
    ]
    return "\n".join(lines)


def semantic_detail(name: str, records: list[object], concerns: list[str]) -> str:
    """生成模块语义域、对象簇、调用链和测试矩阵。"""

    lines = [
        f"## {name} 语义边界", "",
        "语义等价以可观察行为为准，但不能借“功能相似”消除源对象边界。",
        "",
        "| 语义域 | Rust 目标表达 | 必须保留 | 验收证据 |", "|---|---|---|---|",
    ]
    for concern in concerns:
        lines.append(
            f"| {concern} | trait/struct + `Result`/显式上下文；流式边界使用 `Stream` | "
            "正常、边界、错误、顺序与资源释放 | 单元测试和跨对象集成测试 |"
        )
    lines += [
        "", "## 对象簇职责", "",
        "| 目标目录 | 数量 | 代表对象 | 当前状态 |", "|---|---:|---|---|",
    ]
    for directory, items in grouped(records).items():
        samples = "、".join(f"`{object_name(item)}`" for item in items[:5])
        states = "、".join(f"{key}:{value}" for key, value in sorted(Counter(item.status for item in items).items()))
        lines.append(f"| `{escape(directory)}` | {len(items)} | {samples} | {states} |")
    labels = concerns[:5]
    lines += [
        "", "## 目标调用链", "", "```mermaid", "flowchart LR",
        f'    A["{labels[0]}"] --> B["{labels[1]}"]',
        f'    B --> C["{labels[2]}"]',
        f'    C --> D["{labels[3]}"]',
        f'    D --> E["{labels[4]}"]',
        '    E --> F["结果/错误/释放"]',
        '    F -. 失败证据 .-> G["台账状态降级"]',
        "```", "",
        "入口、适配、执行、回调和清理都必须可观察；只验证最终返回值不足以证明顺序、取消或释放正确。",
        "", "## Java 到 Rust 技术语义", "",
        "| Java 机制 | Rust 映射 | 验证重点 |", "|---|---|---|",
        "| nullable | `Option<T>` | `None` 与空值不得混同 |",
        "| checked exception | `thiserror` + `Result<T, E>` | 分类、cause 和上下文 |",
        "| synchronized/并发容器 | `Arc<RwLock<_>>`/`DashMap` | 竞争、可见性和死锁 |",
        "| CompletableFuture | `Future`/`JoinHandle` | 完成、失败、取消和超时 |",
        "| Reactor Mono | `async fn -> Result<T, E>` | 单值、空结果和错误 |",
        "| Reactor Flux | `Stream<Item=Result<T,E>>` | 背压、顺序、取消和终止 |",
        "| ServiceLoader | `inventory` + registry | 自动注册和确定性 |",
        "| JVM/字节码 | 有证据才可 `PLATFORM_NA` | 困难不等于不适用 |",
        "", "## 语义测试矩阵", "",
        "| 语义域 | 正常场景 | 失败/边界 | 观察点 |", "|---|---|---|---|",
    ]
    for concern in concerns:
        lines.append(
            f"| {concern} | 最小有效输入完成全链路 | 空输入、重复、下游错误或取消 | "
            "返回值、错误类型、调用顺序、状态和释放 |"
        )
    lines += [
        "", "## 语义完成清单", "",
        "- [ ] 对象职责和协作边界明确。",
        "- [ ] Javadoc 前置、后置和异常语义译为中文注释。",
        "- [ ] 关键链路有正常、失败和边界测试。",
        "- [ ] 异步能力覆盖完成、错误、取消、超时与背压。",
        "- [ ] 资源型对象覆盖获取、复用和释放。",
        "- [ ] 依赖复用有固定提交、精确符号和集成测试。",
        "- [ ] JVM 专属能力有不可迁移证据。",
        "- [ ] 状态严格使用验收规范定义。",
    ]
    return "\n".join(lines)


def roadmap_detail(name: str, records: list[object], concerns: list[str]) -> str:
    """生成按状态、目录和对象批次展开的实施路线图。"""

    lines = [
        f"## {name} 当前基线", "", *status_table(records), "",
        f"对象总数 **{len(records)}**；只有 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成。",
        "", "## 阶段与退出条件", "",
        "| 阶段 | 工作 | 退出条件 | 失败处理 |", "|---|---|---|---|",
        "| P0 台账冻结 | 固定提交、对象和路径 | 数量可复现且无重复 | 回到源码扫描 |",
        "| P1 结构校正 | 优先处理 `MISPLACED` | 无目录错位 | 回退单对象移动 |",
        "| P2 对象补齐 | 建立一对象一文件真实实现 | 类型和来源注释齐全 | 保持未完成 |",
        "| P3 语义实现 | 生命周期和错误模型 | 单对象语义测试通过 | 降级 `PARTIAL` |",
        "| P4 集成 | 跨对象、依赖和并发链 | 正常与失败集成测试 | 隔离适配层 |",
        "| P5 固化 | 审计、测试、Clippy、文档 | 证据绑定提交和日期 | 漂移即降级 |",
        "", "## 目录批次", "",
        "| 批次 | 目标目录 | 数量 | 当前状态 | 交付证据 |", "|---|---|---:|---|---|",
    ]
    for index, (directory, items) in enumerate(grouped(records).items(), 1):
        states = "、".join(f"{key}:{value}" for key, value in sorted(Counter(item.status for item in items).items()))
        lines.append(f"| D{index:02d} | `{escape(directory)}` | {len(items)} | {states} | 模块声明、对象测试、审计报告 |")
    order = {"MISPLACED": 0, "STUB": 1, "PARTIAL": 2, "MISSING": 3, "UNVERIFIED": 4}
    items = sorted(records, key=lambda item: (order.get(item.status, 9), item.expected_path, item.fqn))
    lines += [
        "", "## 对象批次", "",
        "| 批次 | 对象范围 | 当前状态 | 完成证据 |", "|---|---|---|---|",
    ]
    for offset in range(0, len(items), 20):
        chunk = items[offset : offset + 20]
        names = "、".join(f"`{object_name(item)}`" for item in chunk)
        states = "、".join(sorted({item.status for item in chunk}))
        lines.append(f"| O{offset // 20 + 1:02d} | {names} | {states} | 路径、注释、单测和集成证据 |")
    lines += ["", "## 语义能力顺序", ""]
    for index, concern in enumerate(concerns, 1):
        lines.append(
            f"{index}. **{concern}**：冻结接口和错误类型，完成正常链路，再补失败、取消、并发与释放测试。"
        )
    lines += [
        "", "## 每批强制门禁", "",
        f"- [ ] `python3 scripts/audit_migration_docs.py --module {name} --check` 通过。",
        "- [ ] 目录和 snake_case 文件名与对象表一致。",
        "- [ ] 每个文件只有一个源对象且包含真实逻辑。",
        "- [ ] 无 todo、unimplemented、占位字段或空业务分支。",
        "- [ ] 对象和公开方法具有中文来源注释。",
        "- [ ] 正常、失败、边界、并发/取消和释放测试可定位。",
        "- [ ] 生产代码无 wildcard import。",
        "- [ ] `lib.rs`/`mod.rs` 只声明和重导出。",
        "- [ ] 依赖复用精确到提交、符号和集成测试。",
        "- [ ] 统计从当前工作树生成，不手写完成率。",
        "", "## 最终验收命令", "", "```bash",
        "PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests",
        f"PYTHONDONTWRITEBYTECODE=1 python3 scripts/audit_migration_docs.py --module {name} --check",
        "PYTHONDONTWRITEBYTECODE=1 python3 scripts/audit_migration_docs.py --all --check",
        f"cargo test -p {name}",
        f"cargo clippy -p {name} --all-targets -- -D warnings",
        "```", "",
        "> 若目标 crate 不存在，Cargo 命令必须记录为“目标 crate 不存在”；不得伪造通过。",
    ]
    return "\n".join(lines)


def object_detail(name: str, records: list[object], concerns: list[str]) -> str:
    """生成文档型模块的详细规划对象台账。"""

    lines = [
        f"## {name} 对象边界", "",
        "本模块没有独立 Spring 源码扫描根；下表是显式本地对象或对象族，`PLANNED`/`UNVERIFIED` 不计入完成。",
        "", "## 对象职责与证据", "",
        "| 对象 | 预期路径 | 职责 | 状态 | 必需证据 |", "|---|---|---|---|---|",
    ]
    for index, record in enumerate(records):
        lines.append(
            f"| `{object_name(record)}` | `{escape(record.expected_path)}` | "
            f"{concerns[index % len(concerns)]} | `{record.status}` | 正常、失败、顺序与释放测试 |"
        )
    lines += [
        "", "## 状态和依赖边界", "",
        "| 判定 | 允许条件 | 当前规则 |", "|---|---|---|",
        "| `IMPLEMENTED` | 文件、逻辑、中文注释和测试齐全 | 未审计不得声明 |",
        "| `DEPENDENCY_REUSED` | crate/提交/符号/集成测试齐全 | 仅生态相似仍未完成 |",
        "| `PLATFORM_NA` | JVM 或字节码不可迁移证据 | 实现困难不构成证据 |",
        "| `PARTIAL` | 有真实逻辑但语义不全 | 必须登记缺口 |",
        "| `PLANNED`/`UNVERIFIED` | 设计存在但证据不足 | 不计入完成 |",
        "", "## 对象协作链", "", "```mermaid", "flowchart LR",
        '    A["入口合同"] --> B["适配对象"]',
        '    B --> C["依赖或运行时"]',
        '    C --> D["结果/错误翻译"]',
        '    D --> E["资源释放与观测"]',
        '    E -. 验收证据 .-> F["台账状态"]',
        "```", "", "## 集成测试矩阵", "",
        "| 语义域 | 正常场景 | 失败场景 | 证据 |", "|---|---|---|---|",
    ]
    for concern in concerns:
        lines.append(
            f"| {concern} | 经适配层完成真实调用 | 依赖错误、取消或释放失败 | "
            "类型化错误、顺序、状态和测试路径 |"
        )
    lines += [
        "", "## 对象完成清单", "",
        "- [ ] 一个对象一个 snake_case 文件。",
        "- [ ] 主类型 PascalCase，方法参数 snake_case。",
        "- [ ] `mod.rs` 只声明并重导出。",
        "- [ ] 对象及公开方法有中文来源/设计注释。",
        "- [ ] 依赖符号和版本固定可追溯。",
        "- [ ] 适配层保留原始错误 cause。",
        "- [ ] 资源、事务、取消和释放有测试。",
        "- [ ] 对象表与路线图使用同一对象集合。",
    ]
    return "\n".join(lines)


def replace_detail(path: Path, detail: str) -> None:
    """幂等替换当前详细内容块。"""

    start = "<!-- detailed-current-start -->"
    end = "<!-- detailed-current-end -->"
    text = path.read_text(encoding="utf-8").rstrip() + "\n"
    payload = f"{start}\n{detail.rstrip()}\n{end}\n"
    if start in text and end in text:
        before, remainder = text.split(start, 1)
        _, after = remainder.split(end, 1)
        text = before + payload + after.lstrip("\n")
    else:
        text += "\n" + payload
    path.write_text(text, encoding="utf-8")


def replace_contract(path: Path, contract: str) -> bool:
    """在当前文档顶部幂等写入模块化新规范执行块。"""

    start = AUDIT.CURRENT_CONTRACT_START
    end = AUDIT.CURRENT_CONTRACT_END
    text = path.read_text(encoding="utf-8")
    payload = f"{start}\n{contract.rstrip()}\n{end}"
    if start in text and end in text:
        before, remainder = text.split(start, 1)
        _, after = remainder.split(end, 1)
        updated = before.rstrip() + "\n\n" + payload + "\n\n" + after.lstrip()
    else:
        lines = text.splitlines()
        title_index = next(
            (index for index, line in enumerate(lines) if line.startswith("# ")),
            None,
        )
        if title_index is None:
            raise ValueError(f"current migration document has no title: {path}")
        insert_at = title_index + 1
        while insert_at < len(lines) and (
            not lines[insert_at].strip()
            or lines[insert_at].startswith("> 迁移文档治理：")
        ):
            insert_at += 1
        lines[insert_at:insert_at] = ["", payload, ""]
        updated = "\n".join(lines) + "\n"
    if updated == text:
        return False
    path.write_text(updated, encoding="utf-8")
    return True


def main() -> int:
    """补齐所有不满足详细度门禁的当前四件套。"""

    _, configs = AUDIT.read_manifest(ROOT, ROOT / "docs" / "migration-manifest.toml")
    config_by_dir = {config.docs_dir.name: config for config in configs}
    records_by_dir = {
        name: AUDIT.audit_module(ROOT, config)
        for name, config in config_by_dir.items()
    }
    for name in PROFILES:
        if name not in records_by_dir:
            records_by_dir[name] = synthetic_records(name)

    changed: list[Path] = []
    contracted: list[Path] = []
    renderers = {
        "对象名称一致性检查.md": naming_detail,
        "语义迁移对照表.md": semantic_detail,
        "迁移路线图.md": roadmap_detail,
    }
    for name, concerns in PROFILES.items():
        module_dir = ROOT / "docs" / name
        records = records_by_dir[name]
        config = config_by_dir.get(name)
        contract_files = list(renderers)
        if config is None:
            contract_files.append("对象级对照表.md")
        for filename in contract_files:
            path = module_dir / filename
            if replace_contract(
                path,
                current_contract(name, filename, records, concerns, config),
            ):
                contracted.append(path)
        for filename, renderer in renderers.items():
            path = module_dir / filename
            if AUDIT.validate_current_document_detail(path, audited_object_table=False):
                replace_detail(path, renderer(name, records, concerns))
                changed.append(path)
        object_table = module_dir / "对象级对照表.md"
        if name not in config_by_dir and AUDIT.validate_current_document_detail(
            object_table,
            audited_object_table=False,
        ):
            replace_detail(object_table, object_detail(name, records, concerns))
            changed.append(object_table)

    print(f"updated module contracts: {len(contracted)}")
    print(f"enriched migration documents: {len(changed)}")
    for path in contracted:
        print(f"contract {path.relative_to(ROOT)}")
    for path in changed:
        print(path.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
