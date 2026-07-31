<!-- migration-doc: authority=historical canonical=语义迁移对照表.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [语义迁移对照表.md](语义迁移对照表.md) 为准。

# spring-core + tx_di → vernal-core 功能语义迁移对照表

> 版本：v1.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core + tx_di **dev**
> 现状基线：`vernal-framework/crates/vernal-core`（7 个模块、约 1100 行 Rust 源码）
> 实现底盘：**thiserror + Arc<dyn Error> + 标准库 From 派生**（不引入 anyhow 兼容性约束）

迁移原则：**功能语义对齐，实现方式 Rust 化**。

状态图例：✅ 已迁移并有测试 / 🔶 语义等价但形态不同 / ⬜ 未迁移（路线图）/ 🚫 不迁移 / 🆕 vernal-core 新增

---

## 一、错误体系（**vernal-core 核心改造点**）

### 1.1 错误枚举与三种形态

| Java 概念 | Spring 语义 | tx_di 语义 | vernal-core 实现 | 状态 |
|---|---|---|---|---|
| 嵌套运行时异常 | `NestedRuntimeException` + 子类异常 | `AppError::Internal(anyhow::Error)` | `VernalError::Infrastructure(SharedError)` | 🆕 |
| 业务错误码（zero-alloc） | `ErrorCoded.getErrorCode() -> String` + 子类常量 | `AppError::ErrCode { domain, code, message }` | `VernalError::Business { domain, code, message }` | 🆕 |
| 业务错误 + 动态上下文 | 子类内部 `getMessage()` 拼接 | `AppError::WithContext { context: String }` | `VernalError::WithContext { context: String }` + `WithContextEntries { context: ErrorContext }` | 🆕 |
| `err.appendContext()` | — | `AppError::with_context(code, "...")` | `VernalError::with_context()` + `with_context_entries()` | 🆕 |
| 错误身份比较 | `instanceof` + `equals` | `PartialEq` 只比较 domain + code | `VernalError::PartialEq`（**只比较 domain + code**，忽略 context） | 🆕 |
| 错误分类（4 种） | `ErrorCoded` 子类层级 | — | `ErrorKind::{Business, Infrastructure, Validation, Internal}` | 🆕 |

### 1.2 错误访问器对应

| tx_di `AppError` 方法 | Spring 对应 | vernal-core `VernalError` 方法 | 状态 |
|---|---|---|---|
| `domain() -> &str` | `getMessage()` 中的前缀 | `domain() -> Option<&'static str>`（基础设施返回 `None`） | 🆕 |
| `code() -> i32` | 异常子类常量 | `code() -> Option<i32>` | 🆕 |
| `message() -> &str` | `getMessage()` | `message() -> &'static str`（基础设施返回 `"internal error"`） | 🆕 |
| `context() -> Option<&str>` | — | （与 Spring 对齐：基础版用 `Option<&str>`，结构化版用 `&ErrorContext`） | 🆕 |
| `internal() -> Option<&anyhow::Error>` | — | （用 `std::error::Error::source()` 代替；保留 `is_infrastructure()` 快速判定） | 🆕 |
| `is_internal() -> bool` | — | `is_infrastructure() -> bool` | 🆕 |
| `is_same_kind(&Other) -> bool` | `equals` 仅识别同类 | `ErrorCode::is_same_kind()` 与 `VernalError::PartialEq` 协同 | 🆕 |
| `err_code() -> AppErrCode` | — | `ErrorCode::into_vernal_error()` | 🆕 |
| `full_message() -> String` | — | `Display` + `ErrorReport::display()` | 🆕 |
| `Display` → `"[DOMAIN:CODE] MESSAGE"` | Spring 默认 `getMessage()` 形式 | `Write!` 同样格式 `[{domain}:{code}] {message}` | 🆕 |
| `From<anyhow::Error>` | — | （不引入 anyhow；改用 `From<BoxError>` + `From<SharedError>`） | 🔶 |
| `From<&str>` / `From<String>` | `IllegalArgumentException(msg)` | （不直接转换到 `VernalError`；让业务用 `VernalError::business()` 自己造） | 🔶 |

### 1.3 错误码 trait（`CodeMsg` / `ErrorCode`）

| tx_di `CodeMsg` trait | Spring 对应 | vernal-core `ErrorCode` trait | 状态 |
|---|---|---|---|
| `fn err_code(self) -> AppErrCode` | `ErrorCoded.getErrorCode()` | `fn into_vernal_error(self) -> VernalError`（消费 `self`，与 `Domain`/`code()`/`message()` 三个钩子配套） | 🆕 |
| 派生宏 | `@interface` + JavaDoc 文档 | `#[derive(ErrorCode)]`（vernal-macros 提供，对标 `#[derive(thiserror::Error)]`） | 🆕 |
| `#[err("DOMAIN")]` 域标注 | 子类命名 `XxxException` + Spring 内置约定 | enum 顶层 `#[err("domain")]` 属性 | 🆕 |
| `#[err(code, "message")]` 变体标注 | 子类 `private static final long serialVersionUID` | enum 变体 `#[err(-1, "组件未找到")]` | 🆕 |

### 1.4 错误上下文（`ErrorContext`）

| tx_di 语义 | Spring 对应 | vernal-core 实现 | 状态 |
|---|---|---|---|
| `AppError::WithContext.context: String`（单一段字符串） | `NestedRuntimeException.getMessage()` 拼接 | `VernalError::WithContext.context: String` | 🆕 |
| — | — | `VernalError::WithContextEntries.context: ErrorContext`（多键值对，键是 `&'static str`，值是 `String`） | 🆕 |
| — | — | `ErrorContext::new().with(key, value).with(...)` 链式 API | 🆕 |
| — | `nestedMessageStack` | `ErrorContext::is_empty() / len() / entries()` | 🆕 |

### 1.5 脱敏诊断报告（`ErrorReport`）

| Spring 概念 | tx_di 概念 | vernal-core `ErrorReport` | 状态 |
|---|---|---|---|
| `System.err.println(getMessage())` | — | `Display` 实现 | 🆕 |
| 不暴露 context 中的敏感数据 | （`Internal` 用 `format!("{e}")` 直接打印全链路） | `ErrorReport::from_error(&err)`：**仅暴露** entry 数量，**不暴露** 具体键值 | 🆕 |
| `ErrorReport::is_infrastructure()` | — | 等价 API | 🆕 |
| `ErrorReport::kind() -> ErrorKind` | — | 等价 API（`Business / Infrastructure`） | 🆕 |
| `ErrorReport::to_json()`（Spring 6.1 `ErrorResponseException`） | — | 待新增：返回 `serde_json::Value`（含 `domain/code/message/context_entries/kind`，不含 context 值） | ⬜ |

### 1.6 错误域常量（`ErrorDomain`）

| tx_di `#[err("DI")]` | Spring 默认 namespace | vernal-core `ErrorDomain` 常量 | 状态 |
|---|---|---|---|
| `"DI"` | `org.springframework.beans.factory` | `ErrorDomain::IOC` | 🆕 |
| `"AOP"` | `org.springframework.aop` | `ErrorDomain::AOP` | 🆕 |
| `"CTX"` | `org.springframework.context` | `ErrorDomain::CONTEXT` | 🆕 |
| `"WEB"` | `org.springframework.web` | `ErrorDomain::WEB` | 🆕 |
| `"HTTP"` | `org.springframework.http` | `ErrorDomain::HTTP` | 🆕 |
| `"TOWER"` | — | `ErrorDomain::TOWER` | 🆕 |
| `"DISC"` | `org.springframework.context.annotation.ClassPathBeanDefinitionScanner` | `ErrorDomain::DISCOVERY` | 🆕 |
| `"MAC"` | — | `ErrorDomain::MACROS` | 🆕 |
| `"CORE"` | `org.springframework.core` | `ErrorDomain::CORE` | 🆕 |
| `"BRIDGE"` | — | `ErrorDomain::BRIDGE` | 🆕 |

### 1.7 From 转换矩阵

| 源类型 | Spring 行为 | tx_di 行为 | vernal-core `From` 实现 | 状态 |
|---|---|---|---|---|
| `Box<dyn Error + Send + Sync>` | `new IllegalStateException(cause)` | `AppError::Internal(anyhow!(e))` | `VernalError::Infrastructure(Arc::new(e))` | 🆕 |
| `Arc<dyn Error + Send + Sync>` | — | — | `VernalError::Infrastructure(arc)` | 🆕 |
| `std::io::Error` | `NestedIOException` | `AppError::Internal(e.into())` | `VernalError::Infrastructure(Arc::new(io))` | 🆕 |
| `serde_json::Error` | `NestedRuntimeException` | `From<serde_json::Error>` | （**不直接 impl From**，避免 vernal-core 反向依赖 serde_json；让 `vernal-context` 转换） | 🔶 |
| `toml::de::Error` | `NestedRuntimeException` | `From<toml::de::Error>` | （同上） | 🔶 |
| `anyhow::Error` | — | `From<anyhow::Error>` | （**故意不引入 anyhow**；vernal 沿用 `BoxError/SharedError` 让业务自行选择 anyhow） | 🚫 |

---

## 二、Ordered / init_sort 排序

| Spring `Ordered` | tx_di `init_sort() -> i32` | vernal-core `INIT_SORT_*` | 状态 |
|---|---|---|---|
| `HIGHEST_PRECEDENCE = Integer.MIN_VALUE` | `i32::MIN`（日志组件最先） | `INIT_SORT_INFRASTRUCTURE = i32::MIN + 1` | ✅ |
| （业务默认） | `10000` | `INIT_SORT_BUSINESS = 0` | ✅ |
| （应用层最后） | `i32::MAX`（Web 服务器最后） | `INIT_SORT_APPLICATION = i32::MAX - 1` | ✅ |
| `LOWEST_PRECEDENCE = Integer.MAX_VALUE` | `i32::MAX` | `INIT_SORT_DEFAULT = i32::MAX` | ✅ |
| `PriorityOrdered` 标记接口 | （无显式概念） | （用常量代替 trait：值最小者优先；无需运行时类型标记） | 🔶 |
| `getOrder()` 动态取值 | `Component::init_sort()` 静态函数 | （**未实现**，Spring 风格放在业务代码中：`#[component(init_sort = N)]` 提供静态值；如果用户需要动态值，需要扩展 trait） | ⬜ |
| `OrderComparator` 比较两个 Ordered | `topo_sort` 用 `init_sort` 打破平局 | vernal-context 的 `startup_coordinator` 排序时使用 `INIT_SORT_*` 常量 | 🔶 |

---

## 三、Lifecycle 生命周期阶段

| Spring `Lifecycle` 概念 | tx_di 5 阶段 | vernal-core `LifecyclePhase` 现状 | vernal-context `LifecyclePhase` 现状 | 状态 |
|---|---|---|---|---|
| `isRunning()` 查询是否在运行 | （无 enum，仅 bool 状态） | `LifecyclePhase::is_active()` 判定 Initialized/Started | `LifecyclePhase::Initialize/Start/Stop`（3 变体） | 🔶 |
| `void start()` 启动 | `Component::async_run()` 启动后台长任务 | `LifecyclePhase::Starting/Started` | `LifecyclePhase::Start` | ✅ |
| `void stop()` 停止 | `Component::shutdown()` | `LifecyclePhase::Stopping/Stopped` | `LifecyclePhase::Stop` | ✅ |
| `SmartLifecycle.getPhase(): Integer` | `init_sort()` | （未对应） | — | ⬜ |
| `Lifecycle.start()` before `async_init()`? | tx_di 顺序：`init → async_init → async_run → shutdown`（**没有 start**） | `Initializing → Initialized → Starting → Started`（4 阶段细分） | `Initialize → Start → Stop`（3 阶段简化） | 🔶 |
| `LifecycleProcessor` 协调所有组件的 start/stop | `App::shutdown()` + `App::comp_run()` | （vernal-core 只提供 `LifecyclePhase`；实际协调留给 `vernal-context::application_startup_coordinator`） | 同上 | 🚫 |

> **重命名建议**：vernal-core 中现有 `LifecyclePhase`（8 变体）应当重命名为 `AppLifecyclePhase`，避免与 `vernal-context::LifecyclePhase` 重名（详见《对象名称一致性检查.md》）。

---

## 四、StopWatch 与类型转换

### 4.1 StopWatch

| Spring `StopWatch` | tx_di（无对应） | vernal-core `StopWatch` | 状态 |
|---|---|---|---|
| `new StopWatch(String id)` | — | `StopWatch::new(name: impl Into<String>)` | ✅ |
| `start(String taskName)` / `stop()` | — | `start(name)` / `stop()`（同名 API） | ✅ |
| `getTotalTimeNanos()` | — | `total_elapsed() -> Duration` | ✅ |
| `getTaskCount()` | — | `task_count() -> usize` | ✅ |
| `prettyPrint()` 输出 | — | `pretty_print() -> String` | ✅ |
| `shortSummary()` | — | （**未实现**，添加 `short_summary() -> String` 等价） | ⬜ |
| `prettyPrint(TimeUnit)` Spring 6.1 新增 | — | （**未实现**，添加 `pretty_print_with_unit(unit) -> String`） | ⬜ |
| `start(long)` / `start(String, long)` | — | （**未实现**，添加 `start_with_ticks(name, ticks)`） | ⬜ |

### 4.2 ConversionService

| Spring `ConversionService` 行为 | tx_di（无对应） | vernal-core `ConversionService` | 状态 |
|---|---|---|---|
| `boolean canConvert(sourceType, targetType)` | — | （**未实现**，补充 `can_convert<T: Convertible>() -> bool`） | ⬜ |
| `<T> T convert(Object source, Class<T> targetType)` | — | `ConversionService::convert<T: Convertible>(value: &str) -> Result<T, ConversionError>` | ✅ |
| 运行时注册 Converter | — | （**未实现**，trait 静态分派替代；用户用过程宏或 trait impl 拓展） | 🚫 |
| Generics 转换：`List<String>` ↔ `List<Integer>` | — | （**未实现**，配置属性绑定只需标量转换） | 🚫 |
| 自定义 `Formatter` / `Printer` / `Parser` | — | （**未实现**，由 `Convertible` trait 替代） | 🚫 |

### 4.3 内置 Convertible 类型

| Spring Converter | tx_di | vernal-core `impl Convertible for X` | 状态 |
|---|---|---|---|
| `StringToBooleanConverter` | — | `impl Convertible for bool`（接受 true/1/yes/on/false/0/no/off） | ✅ |
| `StringToNumberConverterFactory` | — | 13 个数字类型 `impl_convertible_for_number!` 宏（i8~i128、isize、u8~u128、usize、f32、f64） | ✅ |
| `StringToEnumConverterFactory` | — | `convert_enum<T: FromStr>()` 函数（对任何 enum + FromStr 都能复用） | ✅ |
| （无对应） | — | `impl Convertible for String`（恒等函数） | 🆕 |
| （无对应） | — | `impl<T: Convertible> Convertible for Option<T>`（空字符串 → None，非空 → Some(T)） | 🆕 |
| `StringToCharsetConverter` | — | （**未实现**，运行时不强制） | 🚫 |
| `StringToUUIDConverter` | — | （**故意不实现**：vernal-core 已经定义 `ObjectId`，UUID 用 `uuid` crate 由业务层引入） | 🚫 |
| `StringToDateConverter` 等 | — | （**未实现**，日期/时间由 `chrono`/`time` crate 提供，不污染 vernal-core） | 🚫 |
| `Duration` ↔ String | — | （**未实现**，但配置属性绑定需要；用 `FromStr` blanket impl 走 `convert_enum::<Duration>` 即可） | ⬜ |

---

## 五、tx_di 特有：inner_init / async_init 阶段

| tx_di 阶段 | vernal 替代方案 | 状态 |
|---|---|---|
| `Component::inner_init(&mut self, store: &Store)` （构造后立即同步，**可以访问 store**） | vernal-core 不提供 `inner_init`；vernal-beans `Component::build(deps, store)` 自带 store 引用，可以在这一步完成所有同步初始化 | 🔶 |
| `Component::init(app: &Arc<App>)` （App 阶段同步） | vernal-context `Lifecycle::initialize()`（async，签名上对齐） | 🔶 |
| `Component::async_init(app) -> BoxFuture` | `Lifecycle::initialize() -> LifecycleFuture<'_>`（仅 async） | 🔶 |
| `Component::async_run(app, token)`（后台 task） | `Lifecycle::start(token)`（async），由 `vernal-context::application_startup_coordinator` 包装为 tokio::spawn | 🔶 |
| `Component::shutdown(&self)`（同步） | `Lifecycle::stop()`（async） | 🔶 |
| `ComponentMeta.has_async_run` 跳过未覆写 | `Lifecycle::start` 同样有默认空实现，调度器跳过未实现的组件 | ✅ |

---

## 六、Rust 生态集成（基于 crates.io 调研的最终方案）

vernal-core 必须保持**默认零外部依赖**（除 `linkme` 在 workspace level 提供）。本节列出经过实际调研的集成策略。

### 6.1 选型矩阵（canonical picks）

| 关注点 | 推荐 crate | 版本 | 许可证 | 整合方式 |
|---|---|---|---|---|
| 错误类型派生 | **`thiserror`** | `2.0.19` | MIT/Apache-2.0 | 文档推荐使用（业务 crate 自行 `#[derive(thiserror::Error)]`），vernal-core 不反向依赖 |
| 通用错误捕获（escape hatch） | **`anyhow`** | `1.0.104` | MIT/Apache-2.0 | **不在 vernal-core 依赖**，**但** vernal-core 的 `BoxError` / `SharedError` 与 anyhow 互转在 `vernal-bridge` 提供 |
| 深度错误链 + 上下文 | **`error-stack`** | `0.8.0` | MIT/Apache-2.0 | 可选（与 `thiserror` 互斥，`vernal-context` 集成） |
| checked 错误选项 | **`snafu`** | `0.9.2` (MSRV 1.81) | MIT/Apache-2.0 | 不采用（MSRV 高，且需要 selector structs，不符合 Rust 习惯） |
| UUID v4 / v7 | **`uuid`** | `1.24.0` | MIT/Apache-2.0 | feature-gated `id/uuid_id.rs`（feature = `"uuid"`） |
| ULID | **`ulid`** | `2.0.1` | MIT | 同上（feature = `"ulid"`） |
| NanoId | **`nanoid`** | `0.5.0` | MIT | 同上（feature = `"nanoid"`，用于短 ID） |
| Snowflake | **`snowflake`** | `1.3.0` | MIT/Apache-2.0 | 同上（feature = `"snowflake"`，Twitter 风格 64-bit ID） |
| 高精度 / WASM 时间 | **`web-time`** | `1.1.0` | MIT/Apache-2.0 | **替换**`std::time::Instant`，`StopWatch::new` 改为 `web_time::Instant::now()` |
| 跨大小写转换 | **`convert_case`** | `0.11.0` | MIT | feature-gated `convert/case_converter.rs`（feature = `"case"`，enum case-insensitive 解析） |
| 插入序 Map | **`indexmap`** | `2.14.0` | MIT/Apache-2.0 | **不在 vernal-core**（仅 `vernal-beans` 等上层使用） |
| 全局类型注册（与 linkme 并列） | **`inventory`** | `0.3.24` | MIT/Apache-2.0 | 可选替代 linkme（仅在需要 dynamic lib 插件时引入） |
| async fn in dyn traits | **`async-trait`** | `0.1.91` | MIT/Apache-2.0 | **不引入**（Rust 1.75+ 已有原生支持；vernal-core 不持有 dyn trait） |
| 日志 / 链路追踪 | **`tracing`** | `0.1.44` | MIT | **不引入**（归 `vernal-log`） |

### 6.2 显式排除的 crate

| 排除 | 原因 |
|---|---|
| `priority-queue` 2.7.0 | **LGPL-3.0 OR MPL-2.0 copyleft**，污染 Vernal 的 MIT 协议 |
| `instant` 0.1.13 | **已停止维护**（README 推荐 fork 或用 web-time） |
| `snowflake-rs` 0.1.1 | **自 2018 起废弃**，依赖过时的 `time` 0.1 |
| `conv` / `conv2` / `easy-cast` / `num_convert` | 都是数值转换 traits，**抽象错位**（Spring ConversionService 是 String → T 转换，不一样） |
| `qbench` | crates.io 上找不到具体包；改用 `web-time::Instant` + 手动聚合 |

### 6.3 集成原则

1. **vernal-core 保持零外部依赖**（默认 `cargo build`）：所有上述 crate 均作为 **可选 feature flag** 提供。
2. **新 feature flag 命名**：经过 CRUD 测试的统一前缀：
   - `feature = "uuid"` / `"ulid"` / `"nanoid"` / `"snowflake"`（ID）
   - `feature = "case"`（enum 大小写无关）
   - `feature = "serde"`（ID 序列化）
   - `feature = "chrono"` / `"time"`（日期 + Convertible）
3. **`VernalError::Infrastructure` 选用 `Arc<dyn Error>`**：与 Spring `NestedRuntimeException` 语义一致，**不**把 anyhow 拉进 vernal-core 的依赖图。
4. **`BoxError` / `SharedError` 与 anyhow 互转**：在 `vernal-bridge` crate 提供 `impl From<anyhow::Error> for BoxError`、`From<BoxError> for anyhow::Error`，确保业务 crate 即便已用 anyhow 也能 0 成本接入 vernal-core。
5. **`web_time::Instant` 替换 `std::time::Instant`**：让 vernal-core **目标**支持 `wasm32-unknown-unknown`（虽然 vernal 暂时不发布 wasm 版）。`web-time` 在 native 是 `std::time::Instant` 的 alias，零额外开销。

### 6.4 与 tx_di 的依赖差异

| tx_di 依赖 | vernal-core 依赖 | 差异说明 |
|---|---|---|
| `anyhow = "1"` | **不引入**（推荐业务 crate 用 `thiserror`，`vernal-bridge` 提供 `From<anyhow>` 互转） | vernal 用 `Arc<dyn Error>` 作为 `VernalError::Infrastructure` 的内部载体，避免与 `thiserror` 互斥 |
| `dashmap = "5"` | **不引入** | vernal-core 不持有并发 Map，那是 `vernal-beans` 的责任 |
| `linkme = "0.3"` | **不引入**（workspace-level 已提供） | 留给 `vernal-discovery` |
| `tracing = "0.1"` | **不引入** | 留待 `vernal-log`；vernal-core 自身不打印日志 |
| `tokio = "1"` | **不引入** | vernal-core 完全同步；`VernalError` 不需要异步语义 |
| `tokio-util = "0.7"` | **不引入** | 由 `vernal-context` 引入并 re-export |
| `serde_json` | **不引入**（feature gate "serde" 可选） | 让 `vernal-context` 决定 JSON 错误格式化 |
| `toml` | **不引入** | 同上 |

### 6.5 建议的 `Cargo.toml` 增量变更

```toml
# crates/vernal-core/Cargo.toml 新增部分

[dependencies]
linkme = { workspace = true }                # 已有
thiserror = { version = "2.0", optional = true }
indexmap = { version = "2.14", optional = true, default-features = false }
web-time = { version = "1.1", optional = true, default-features = false }
convert_case = { version = "0.11", optional = true }
uuid = { version = "1.24", optional = true, features = ["v7"] }
ulid = { version = "2.0", optional = true }
nanoid = { version = "0.5", optional = true }
snowflake = { version = "1.3", optional = true }
serde = { version = "1", optional = true }
chrono = { version = "0.4", optional = true, default-features = false, features = ["clock", "serde"] }

[features]
default = []
uuid        = ["dep:uuid"]
ulid        = ["dep:ulid"]
nanoid      = ["dep:nanoid"]
snowflake   = ["dep:snowflake"]
serde       = ["dep:serde"]
chrono      = ["dep:chrono"]
case        = ["dep:convert_case"]
web-time    = ["dep:web-time"]
derive-all  = ["dep:thiserror"]
```

> **最终编译目标**：
> - `cargo build -p vernal-core` 默认零外部依赖通过
> - `cargo build -p vernal-core --features "uuid,serde"` 增加 2 个依赖，启用 `ObjectId` 的 UUID v7 + JSON 序列化
> - 整个 vernal workspace 仅在 `vernal-bridge` / `vernal-context` 等上层按需启用 `chrono` / `thiserror`，**vernal-core 自身不依赖**

---

## 七、规范与命名一致性

| 来源 | 类型命名 | vernal-core 处理 |
|---|---|---|
| Spring `ConversionService` / `Ordered` / `Lifecycle` / `SmartLifecycle` | 全大驼峰 + 动词/名词 | 全部保留大驼峰（`ConversionService`、`Ordered`、`Lifecycle`、`SmartLifecycle`） |
| tx_di `Component` / `Scope` / `Store` / `BuildContext` / `App` | 同上 | 大驼峰，**但 `App` → `ApplicationContext`**（迁到 `vernal-context`） |
| tx_di `AppError` / `AppErrCode` / `AppResult` | `AppXxx` 命名族 | vernal 改为 `VernalError` / `VernalErrorCode` / **`保留** `BoxError`/**`SharedError`` 反而是 vernal 新增** |
| tx_di `CodeMsg` | 短名 | 改为 `ErrorCode`（语义更广，包括 domain/code/message + into_vernal_error） |
| Rust 标准 `Init` / `Run` / `Stop` | 全小写动词 | vernal 用同样的小写动词：`start`、`initialize`、`stop`、`shutdown` |

---

## 八、目标文件分布

迁移完成后，vernal-core 应该新增下列子模块（**目标**）

```
crates/vernal-core/src/
├── lib.rs                              # FRAMEWORK_VERSION 等常量
├── failure.rs                          # BoxError / SharedError（已有）
├── lifecycle_phase.rs                  # 重命名为 app_lifecycle_phase.rs
├── ordered.rs                          # INIT_SORT_* 常量（已有）
├── convert/
│   ├── mod.rs                          # ConversionService + Convertible trait（已有）
│   ├── converter.rs                    # Converter<S, T> trait（已有）
│   ├── boolean_converter.rs            # bool（已有）
│   ├── number_converter.rs             # 13 数字类型（已有）
│   ├── string_converter.rs             # String（已有）
│   ├── option_converter.rs             # Option<T>（已有）
│   ├── enum_converter.rs               # convert_enum<T: FromStr>（已有）
│   ├── datetime_converter.rs           # 🆕 OffsetDateTime / Duration（可选 feature）
│   └── path_converter.rs               # 🆕 PathBuf（可选 feature）
├── error/
│   ├── mod.rs                          # 重新导出（已有）
│   ├── vernal_error.rs                 # 4 变体 enum（已有）
│   ├── error_code.rs                   # ErrorCode trait + is_same_kind（已有）
│   ├── error_kind.rs                   # 4 分类（已有）
│   ├── error_domain.rs                 # 子系统域常量（已有）
│   ├── error_context.rs                # ErrorContext 键值对（已有）
│   ├── error_report.rs                 # 脱敏报告（已有）
│   └── from_io.rs                      # 🆕 From<std::io::Error>（独立文件以便 feature gating）
├── id/
│   ├── mod.rs                          # 重新导出（已有）
│   ├── object_id.rs                    # ObjectId（已有）
│   └── serde_object_id.rs              # 🆕 serde 支持（可选 feature）
└── time/
    ├── mod.rs                          # 重新导出（已有）
    └── stop_watch.rs                   # StopWatch（已有）
```

**新增 .rs 文件数**：4 (`datetime_converter.rs` / `path_converter.rs` / `from_io.rs` / `serde_object_id.rs`)
**修改 .rs 文件数**：3 (重命名 `lifecycle_phase.rs` 为 `app_lifecycle_phase.rs`；`ordered.rs` 增加 3 个 Spring 风格锚点常量；`stop_watch.rs` 增加 short_summary/prettyPrint(unit))

---

## 九、验收清单（迁移完成时所有条目须 ✅）

- [ ] `vernal_core::VernalError` 测试覆盖 4 个变体的 Display/Debug/PartialEq/from_code/from_anyhow（0 anyhow）
- [ ] `vernal_core::error::ErrorCode` trait 的 `is_same_kind()` 测试通过
- [ ] `vernal_core::error::ErrorReport::from_error(&internal_err)` 不泄露源错误链到 Display（防信息泄露）
- [ ] `vernal_core::INIT_SORT_INFRASTRUCTURE < INIT_SORT_BUSINESS < INIT_SORT_APPLICATION < INIT_SORT_DEFAULT` 常量关系保持不变
- [ ] `vernal_core::LifecyclePhase`（**已重命名为 `AppLifecyclePhase`**）的 8 个变体映射 Spring `SmartLifecycle` 8 状态
- [ ] `vernal_core::Convertible` 的 5 个内置实现（bool / 数字 / String / Option / enum）全部通过 round-trip 测试
- [ ] `vernal_core::StopWatch::short_summary()` 输出与 Spring 6.1 `prettyPrint(TimeUnit.NANOSECONDS)` 字段对齐
- [ ] `vernal_core` 在 `cargo build --no-default-features` 下零外部依赖通过
- [ ] `vernal_core::From<std::io::Error> for VernalError` 与 Spring `NestedIOException` 同语义（IO → Infrastructure）
- [ ] 所有公开类型均有中文 `///` rustdoc 注释

