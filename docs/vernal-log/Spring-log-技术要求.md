<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-log 技术要求
> 迁移文档治理：本文级别为 **authoritative**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。


> 来源：`spring-core/.../core/log` 的 5 个业务对象；规则见
> [迁移验收规范](../迁移验收规范.md)。

当前 `crates/vernal-log` 只有 `LogFactory`，而 Spring 本范围没有该顶层对象；它是 Vernal
初始化辅助，且方法仍含 TODO，不能抵消任何来源对象。目标按末两层保留：

| Java | 目标 |
|---|---|
| `core/log/LogAccessor.java` | `core/log/log_accessor.rs` |
| `core/log/LogMessage.java` | `core/log/log_message.rs` |

后端可依赖 `tracing`，但必须为每个 Spring 对象记录精确符号和 wrapper 语义，不能用
“tracing 已有日志”直接标 `DEPENDENCY_REUSED`。

```mermaid
flowchart LR
    L["lazy LogMessage"] --> A["LogAccessor level check"]
    A --> D["CompositeLog / delegate"]
    D --> T["tracing backend"]
```

---

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->

## 原详细文档（完整保留）

> 以下正文完整恢复自 Vernal 提交 `dd20300d16a09200bd8a379ff14db1e2da99b67c`。其中历史对象数量、完成状态、
> 路径算法和依赖替代结论如与本文顶部或自动对象台账冲突，以顶部当前结论和
> `docs/migration-audit/` 为准；其 API、设计背景、阶段拆解和测试说明继续保留。

# vernal-log 技术要求（对标 spring-jcl）

> **版本**：v1.0（2026-07-28）
> **定位**：vernal-log crate 技术交接文档，对标 Spring Framework 7.0.8 spring-jcl（Java Commons Logging）。
> **现状**：2 文件 / 33 行骨架，edition 2024 / rustc 1.88。
> **引用约定**：crate 选型依据见《Spring 组件替换约定》8.8 节（日志 — 异步调度邻近）。

---

## 一、总览

### 1.1 定位与边界

vernal-log 是 Vernal Framework 的 **日志门面 + 初始化入口**，对标 spring-jcl 模块，
底层基于 `tracing 0.1.41` + `tracing-subscriber`，提供框架级日志抽象。

| 维度 | spring-jcl（语义参考） | vernal-log（实现） | 差异说明 |
|:---|:---|:---|:---|
| 门面 | `org.apache.commons.logging.Log` | `tracing` macro 族 | tracing 已统一为 facade |
| 工厂 | `LogFactory.getLog(...)` | `LogFactory::get(name)` | 1:1 对应 |
| 后端 | Log4j / JUL / Logback | **tracing-subscriber** | tracing 生态 |
| 级别 | TRACE/DEBUG/INFO/WARN/ERROR/FATAL | `tracing::Level`（TRACE/DEBUG/INFO/WARN/ERROR） | 6→5 |
| 结构化日志 | ❌ 不支持 | ✅ tracing 原生支持 | 升级 |
| 异步日志 | ❌ 不支持 | ✅ 通过 `tracing-subscriber` + `tracing-appender::non_blocking` | 升级 |
| Span 跟踪 | ❌ 不支持 | ✅ tracing 的核心特性 | 升级 |
| MDC | ✅ ThreadLocal | ✅ `tracing::Span::current()` | 1:1 语义 |
| 桥接其他门面 | SLF4J / Log4j2 / Logback | ❌ 不桥接（直接用 tracing） | — |

### 1.2 架构分层

```
┌─────────────────────────────────────────────────────────┐
│  用户代码：tracing::info!("user logged in")              │
│  → tracing macro 族 → Span 上下文传递                   │
├─────────────────────────────────────────────────────────┤
│  门面层：LogFactory + Logger trait                       │
│  → 抽象日志接口（适配 Spring Log 语义）                  │
├─────────────────────────────────────────────────────────┤
│  桥接层：TracingLogger（实现 Logger）                    │
│  → 把 trace/debug/info/warn/error 映射到 tracing        │
├─────────────────────────────────────────────────────────┤
│  订阅器层：tracing-subscriber                            │
│  → EnvFilter / fmt / JSON / 非阻塞写入                  │
├─────────────────────────────────────────────────────────┤
│  输出层：stdout / file / OTLP / syslog                   │
│  → tracing-appender / opentelemetry-otlp                │
└─────────────────────────────────────────────────────────┘
```

### 1.3 关键决策

| 项 | 决策 | 理由 |
|:---|:---|:---|
| 门面库 | **tracing 0.1.41** | 已统一为 Rust 生态事实标准；原生支持 Span / 结构化日志 / 异步友好 |
| 订阅器 | **tracing-subscriber** | 官方订阅器；支持 fmt / EnvFilter / JSON |
| 异步写入 | `tracing-appender::non_blocking` | 不阻塞主线程 |
| 抽象 Logger | `Logger` trait | 对齐 Spring `Log`，让用户能用 `LogFactory::get` |
| Init 控制权 | `LogFactory::init*` 静态方法 | 与 Spring `LogFactory.getLog` 1:1 对应 |
| FATAL | 🚫 移除 | tracing 仅有 ERROR 顶位；如需 FATAL，使用 ERROR + custom level |
| Spring Log 桥接 | 兼容层（`LogAdapter`） | 让 Spring 风格 `log.info(msg)` 调用映射到 tracing |
| 远程日志 | 不内置 | 留给 OTLP / Loki 适配（`opentelemetry-otlp`） |

### 1.4 当前骨架文件

| 文件 | 行数 | 内容 |
|:---|:---|:---|
| `lib.rs` | 8 | 模块声明 + re-export（`LogFactory`） |
| `log_factory.rs` | 20 | `LogFactory` struct + `init()` / `init_with_level()`（空实现） |

### 1.5 与其他 crate 的边界

| crate | 关系 | 说明 |
|:---|:---|:---|
| `vernal-core` | 依赖 | 基础类型（**当前未实际引用**，预留） |
| `vernal-aop` | 上层使用 | AOP 切面日志 |
| `vernal-aspects` | 上层使用 | 切面日志 |
| `vernal-context` | 同级装配 | 启动时调用 `LogFactory::init()` |
| `vernal-web` | 下游使用 | HTTP 请求/响应日志 |
| `tracing` 0.1.41 | 上游 | 门面 macro 族 |
| `tracing-subscriber` | 上游 | 订阅器（fmt / EnvFilter） |
| `tracing-appender` | 上游 | 异步非阻塞写入 |

---

## 二、核心 Trait 体系

### 2.1 Logger —— 日志门面契约

**目标**：对标 Spring `org.apache.commons.logging.Log`。

#### Spring API（Java）

```java
public interface Log {
    boolean isDebugEnabled();
    boolean isErrorEnabled();
    boolean isFatalEnabled();
    boolean isInfoEnabled();
    boolean isTraceEnabled();
    boolean isWarnEnabled();

    void trace(Object message);
    void trace(Object message, Throwable t);
    void trace(String format, Object... args);

    void debug(Object message);
    void debug(Object message, Throwable t);
    void debug(String format, Object... args);

    void info(Object message);
    void info(Object message, Throwable t);
    void info(String format, Object... args);

    void warn(Object message);
    void warn(Object message, Throwable t);
    void warn(String format, Object... args);

    void error(Object message);
    void error(Object message, Throwable t);
    void error(String format, Object... args);

    void fatal(Object message);
    void fatal(Object message, Throwable t);
    void fatal(String format, Object... args);
}
```

#### Rust trait（待补齐）

```rust
use std::fmt;
use std::sync::Arc;
use tracing::{Event, Metadata, Subscriber};
use tracing::span::{Attributes, Id, Record};

/// 日志门面 trait — 对标 Spring `Log`。
pub trait Logger: Send + Sync {
    /// 是否启用对应级别
    fn is_enabled(&self, level: LogLevel) -> bool;

    /// 输出日志
    fn log(&self, level: LogLevel, message: &str);
    fn log_with_error(&self, level: LogLevel, message: &str, error: &(dyn std::error::Error + Send + Sync));

    // 便捷方法（默认委托到 log/log_with_error）
    fn trace(&self, message: &str) { self.log(LogLevel::Trace, message); }
    fn debug(&self, message: &str) { self.log(LogLevel::Debug, message); }
    fn info(&self, message: &str)  { self.log(LogLevel::Info, message); }
    fn warn(&self, message: &str)  { self.log(LogLevel::Warn, message); }
    fn error(&self, message: &str) { self.log(LogLevel::Error, message); }
    // fatal → error（tracing 无 FATAL）
    fn fatal(&self, message: &str) { self.log(LogLevel::Error, message); }
}

/// 日志级别 — 对齐 Spring + tracing。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_tracing_level(self) -> tracing::Level {
        match self {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info  => tracing::Level::INFO,
            LogLevel::Warn  => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}
```

#### 约束表

| 约束项 | 要求 | 说明 |
|:---|:---|:---|
| `Send + Sync` | 必须 | 跨 Tokio task 共享 |
| `is_enabled` | 必须 | 用于条件日志，避免构造开销 |
| `log` / `log_with_error` | 必须 | 核心方法 |
| `trace/debug/info/warn/error` | 默认实现 | 调用 `log()` |
| `fatal` | 默认映射到 `Error` | tracing 无 FATAL |

---

### 2.2 LogFactory —— 日志工厂

**现状**：已定义 struct + 两个空方法。

```rust
pub struct LogFactory;

impl LogFactory {
    pub fn init() {
        // TODO: 集成 tracing_subscriber
    }

    pub fn init_with_level(level: &str) {
        // TODO: 集成 tracing_subscriber::fmt::init()
    }
}
```

#### 待补齐

```rust
impl LogFactory {
    /// 默认初始化（读取 RUST_LOG 环境变量）
    pub fn init() -> Result<(), LogError> {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init()
            .map_err(|e| LogError::AlreadyInitialized(e.to_string()))
    }

    /// 指定默认级别初始化
    pub fn init_with_level(level: &str) -> Result<(), LogError> {
        let filter = EnvFilter::try_new(level)
            .map_err(|e| LogError::InvalidFilter(e.to_string()))?;
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .try_init()
            .map_err(|e| LogError::AlreadyInitialized(e.to_string()))
    }

    /// 完整配置初始化
    pub fn init_with_config(config: LogConfig) -> Result<(), LogError>;

    /// 获取命名 Logger
    pub fn get(name: &str) -> Arc<dyn Logger> {
        Arc::new(TracingLogger::new(name.to_string()))
    }
}
```

#### Spring 对照

| Spring 方法 | vernal-log 方法 | 状态 |
|:---|:---|:---|
| `LogFactory.getLog(String)` | `LogFactory::get(name)` | ⏳ |
| `LogFactory.getLog(Class)` | `LogFactory::get(name)` | ⏳ |
| `LogFactory.releaseAll()` | ⏳ | — |
| `LogFactory.getInstance()` | （单例模式，直接调用 `get`） | ✅ |

---

### 2.3 TracingLogger —— tracing 实现

**目标**：实现 `Logger` trait，桥接到 tracing。

```rust
pub struct TracingLogger {
    target: String,
}

impl TracingLogger {
    pub fn new(target: impl Into<String>) -> Self {
        Self { target: target.into() }
    }
}

impl Logger for TracingLogger {
    fn is_enabled(&self, level: LogLevel) -> bool {
        use tracing::level_filters::LevelFilter;
        let target_level = tracing::level_filters::STATIC_MAX_LEVEL_FALLBACK;  // 简化
        level.as_tracing_level() <= target_level
    }

    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Trace => tracing::trace!(target: %self.target, "{}", message),
            LogLevel::Debug => tracing::debug!(target: %self.target, "{}", message),
            LogLevel::Info  => tracing::info!(target: %self.target, "{}", message),
            LogLevel::Warn  => tracing::warn!(target: %self.target, "{}", message),
            LogLevel::Error => tracing::error!(target: %self.target, "{}", message),
        }
    }

    fn log_with_error(
        &self,
        level: LogLevel,
        message: &str,
        error: &(dyn std::error::Error + Send + Sync),
    ) {
        match level {
            LogLevel::Trace => tracing::trace!(target: %self.target, error = %error, "{}", message),
            LogLevel::Debug => tracing::debug!(target: %self.target, error = %error, "{}", message),
            LogLevel::Info  => tracing::info!(target: %self.target, error = %error, "{}", message),
            LogLevel::Warn  => tracing::warn!(target: %self.target, error = %error, "{}", message),
            LogLevel::Error => tracing::error!(target: %self.target, error = %error, "{}", message),
        }
    }
}
```

---

### 2.4 LogConfig —— 高级配置

**目标**：让 `LogFactory::init_with_config` 接受结构化配置。

```rust
pub struct LogConfig {
    pub default_level: LogLevel,
    pub filter: Option<String>,                  // RUST_LOG 风格
    pub format: LogFormat,
    pub writer: LogWriter,
    pub span_events: Vec<tracing::span::SpanEvent>,
    pub with_ansi: bool,
}

#[derive(Debug, Clone)]
pub enum LogFormat {
    Full,           // 时间戳 + level + target + message
    Compact,        // 简化版
    Pretty,         // 多行版（开发用）
    Json,           // 结构化 JSON（生产用）
}

#[derive(Debug, Clone)]
pub enum LogWriter {
    Stdout,
    Stderr,
    File { path: PathBuf, non_blocking: bool, rotate: Option<RotateConfig> },
    Otlp { endpoint: String },
}
```

---

## 三、订阅器与 Appender

### 3.1 EnvFilter —— 级别控制

```rust
use tracing_subscriber::EnvFilter;

// 1. 从 RUST_LOG 读取
let filter = EnvFilter::try_from_default_env()?;

// 2. 指定默认级别
let filter = EnvFilter::new("info,vernal_cache=debug,sqlx=warn");

// 3. 显式覆盖
let filter = EnvFilter::from_default_env()
    .add_directive("vernal_core=trace".parse().unwrap());
```

### 3.2 fmt Subscriber —— 格式化输出

```rust
use tracing_subscriber::fmt;

tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_target(true)        // 显示 module path
    .with_thread_ids(false)
    .with_thread_names(false)
    .with_file(false)
    .with_line_number(false)
    .with_ansi(true)
    .compact()
    .init();
```

### 3.3 JSON Subscriber —— 结构化输出

```rust
use tracing_subscriber::fmt::format::FmtSpan;

tracing_subscriber::fmt()
    .json()
    .with_current_span(true)
    .with_span_list(false)
    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    .init();
```

### 3.4 Non-blocking Writer —— 异步写入

```rust
use tracing_appender::non_blocking;

let (non_blocking, guard) = non_blocking(std::io::stdout());
// guard 必须持有到进程结束，否则日志丢失
```

### 3.5 日志轮转

```rust
use tracing_appender::rolling::{daily, Rotation};

let file_appender = daily("/var/log/vernal", "vernal.log");
```

---

## 四、MDC / Span 集成

### 4.1 Spring MDC 对照

| Spring MDC | vernal-log 实现 |
|:---|:---|
| `MDC.put("traceId", id)` | `tracing::Span::current().record("traceId", &id)` |
| `MDC.get("traceId")` | 通过 `tracing` subscriber 提取 |
| `MDC.clear()` | `tracing::Span::exit()` 或 `.in_scope` 闭包结束 |

### 4.2 HTTP 请求示例

```rust
// vernal-web（伪代码）
#[tracing::instrument(skip(req), fields(method = %req.method, uri = %req.uri))]
pub async fn handle_request(req: Request) -> Response {
    let trace_id = generate_trace_id();
    tracing::Span::current().record("trace_id", &trace_id.as_str());
    process_request(req).await
}
```

### 4.3 用户上下文传递

```rust
// 用户登录后
let user_span = tracing::info_span!("user_context", user_id = %user.id, role = %user.role);
let _enter = user_span.enter();
// 后续所有日志都带有 user_id 字段
```

---

## 五、与 tracing 生态集成

### 5.1 现有 vernal 集成现状

| crate | 使用 tracing |
|:---|:---|
| `vernal-core` | ✅ 已有 |
| `vernal-expression` | ✅ 已有 |
| `vernal-context` | ⏳ 待集成 |
| `vernal-cache` | ⏳ 待集成 |
| `vernal-tx` | ⏳ 待集成 |
| `vernal-async` | ⏳ 待集成 |
| `vernal-log` | ✅ 本 crate 提供门面 |

### 5.2 跨 crate 调用链

```rust
// vernal-context 启动时
LogFactory::init_with_config(LogConfig {
    default_level: LogLevel::Info,
    filter: Some("vernal=debug,sqlx=warn".to_string()),
    format: LogFormat::Json,
    writer: LogWriter::Stdout,
    ..Default::default()
})?;
```

### 5.3 适配外部日志门面

如需调用使用 log facade 的第三方 crate，使用 `tracing-log`：

```toml
[dependencies]
tracing-log = "0.2"  # 把 log 宏转发到 tracing
```

---

## 六、测试与验证

### 6.1 单元测试（待补齐）

| 测试项 | 目标 |
|:---|:---|
| `log_factory_init_with_level` | `init_with_level("debug")` 设置 DEBUG |
| `log_factory_init_with_invalid_level` | 非法级别返回 `InvalidFilter` |
| `log_factory_init_twice` | 第二次 init 返回 `AlreadyInitialized` |
| `tracing_logger_is_enabled` | 验证各级别启用判断 |
| `tracing_logger_log_emits_event` | 调用 `log` 后能通过 custom subscriber 捕获 event |
| `log_level_ord` | TRACE < DEBUG < INFO < WARN < ERROR |

### 6.2 集成测试（待补齐）

```rust
// crates/vernal-log/tests/integration.rs（待创建）
use tracing_subscriber::fmt::TestWriter;

#[test]
fn test_log_emission() {
    // 1. 用 TestWriter 替换 stdout
    // 2. init_with_config(...)
    // 3. 调用 logger.info("test")
    // 4. 验证 writer 收到 "test"
}
```

### 6.3 性能基准（待补齐）

| 基准 | 目标 |
|:---|:---|
| `bench_is_enabled_off` | 日志禁用时 `is_enabled` < 5ns |
| `bench_info_log` | info 级别日志调用 < 100ns |
| `bench_json_format` | JSON 格式化 < 5µs/行 |
| `bench_non_blocking_throughput` | non_blocking writer > 100k log/s |

### 6.4 编译期验证

- `cargo check -p vernal-log --all-features` 必须通过。
- `cargo clippy -p vernal-log --all-features -- -D warnings` 必须 0 警告。
- `cargo test -p vernal-log` 必须 100% 通过。

---

## 附录 A：与 Spring JCL 的完整 API 对照

| Spring 接口 | vernal-log 类型 | 状态 |
|:---|:---|:---|
| `org.apache.commons.logging.Log` | `Logger` trait | ⏳ |
| `org.apache.commons.logging.LogFactory` | `LogFactory` struct | ✅（骨架） |
| `Log.isTraceEnabled()` | `Logger::is_enabled(LogLevel::Trace)` | ⏳ |
| `Log.trace(msg)` | `Logger::trace(msg)` | ⏳ |
| `Log.debug(msg)` | `Logger::debug(msg)` | ⏳ |
| `Log.info(msg)` | `Logger::info(msg)` | ⏳ |
| `Log.warn(msg)` | `Logger::warn(msg)` | ⏳ |
| `Log.error(msg)` | `Logger::error(msg)` | ⏳ |
| `Log.fatal(msg)` | `Logger::fatal(msg)`（→ ERROR） | ⏳ |
| `Log.trace(msg, t)` | `Logger::log_with_error(LogLevel::Trace, msg, err)` | ⏳ |
| `LogFactory.getLog(String)` | `LogFactory::get(name)` | ⏳ |
| `LogFactory.releaseAll()` | ⏳ | — |

## 附录 B：依赖清单

| 依赖 | 版本 | 用途 |
|:---|:---|:---|
| `vernal-core` | path = `../vernal-core` | 基础类型（**当前未引用**，预留） |
| `tracing` | 0.1.41（**待集成**） | 日志门面 macro 族 |
| `tracing-subscriber` | workspace（**待集成**） | 订阅器（fmt / EnvFilter / JSON） |
| `tracing-appender` | workspace（**待集成**） | 非阻塞写入 + 日志轮转 |
| `tracing-log` | 0.2（**待集成**） | log → tracing 转发 |
| `thiserror` | workspace（**待集成**） | `LogError` derive |

## 附录 C：迁移路线

### C.1 P0（v0.1，骨架完成）

- [x] `LogFactory` struct
- [x] `LogFactory::init()` / `init_with_level()`（空实现）
- [ ] 集成 `tracing-subscriber`
- [ ] `LogFactory::init()` 实际初始化
- [ ] `LogLevel` enum
- [ ] `Logger` trait
- [ ] `TracingLogger` 实现

### C.2 P1（v0.2，结构化）

- [ ] `LogConfig` struct
- [ ] JSON / Compact / Pretty 三种 format
- [ ] `LogWriter` 枚举（Stdout / File / Otlp）
- [ ] 非阻塞写入（`non_blocking` + guard）

### C.3 P2（v0.3，高级）

- [ ] 日志轮转（tracing-appender rolling）
- [ ] Span 集成（`#[tracing::instrument]` 文档化）
- [ ] 与 vernal-aspects 集成（切面日志自动织入）
- [ ] 与 vernal-web 集成（HTTP 请求日志中间件）

### C.4 P3（v0.4，远程）

- [ ] OTLP 导出（`opentelemetry-otlp`）
- [ ] 进程内 log buffer（崩溃前 dump）
- [ ] 与 `tracing-actix-web` / `tracing-tokio` 对齐

### C.5 P4（v1.0，发布）

- [ ] 完整文档 + 示例
- [ ] `cargo doc` 公开 API
- [ ] 100% 测试覆盖

## 附录 D：向后兼容与弃用策略

- 本 crate 处于 v0.x 阶段，**允许 breaking change**。
- `Logger` trait 新增方法必须提供默认实现。
- `LogLevel` 是 enum，新增变体是 minor change。
- `LogFactory::init` 重复调用返回错误而非 panic（避免影响测试）。
- tracing 升级（如 0.2）需经 vernal-architecture RFC 评审。

## 附录 E：使用示例

### E.1 基础用法

```rust
// 在 main 中初始化
fn main() {
    vernal_log::LogFactory::init_with_level("info,vernal_cache=debug").unwrap();

    // 方式 1：直接用 tracing macro（推荐）
    tracing::info!("应用启动");

    // 方式 2：用 LogFactory 拿 Logger
    let logger = vernal_log::LogFactory::get("com.example.MyService");
    logger.info("用户登录成功");
    logger.error_with(&"数据库连接失败", &err);
}
```

### E.2 高级配置

```rust
use vernal_log::{LogConfig, LogFormat, LogWriter, LogLevel};

LogFactory::init_with_config(LogConfig {
    default_level: LogLevel::Info,
    filter: Some("vernal=debug,sqlx=warn,tower=info".to_string()),
    format: LogFormat::Json,
    writer: LogWriter::Stdout,
    with_ansi: false,
    span_events: vec![],
})?;
```

### E.3 Span 上下文

```rust
#[tracing::instrument(skip(self), fields(user_id = %user_id))]
pub async fn process_user(&self, user_id: u64) -> Result<(), BoxError> {
    tracing::info!("开始处理用户");
    self.db.query_user(user_id).await?;
    tracing::info!("处理完成");
    Ok(())
}
```