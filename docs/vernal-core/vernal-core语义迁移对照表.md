# spring-core + tx_di → vernal-core 功能语义迁移对照表 v2.0

> 版本：v2.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core（12.8 万行 Java）+ tx_di **dev**（73,479 行 Rust）
> 现状基线：`vernal-framework/crates/vernal-core`（22 个 .rs / 1,417 行 / 7 模块）
> 实现底盘：**Rust 同步语义 + Arc<dyn Error> + 零外部依赖 + feature-gated 可选集成**
>
> **本表配套三份文档**：
> - 《vernal-core 迁移路线图 v2.0》：阶段计划与工作量
> - 《vernal-core 对象级对照表 v2.0》：每个源对象的归属判定
> - 《vernal-core 对象名称一致性检查 v2.0》：命名空间冲突与修复
>
> 迁移原则：**功能语义对齐，实现方式 Rust 化**。Spring classpath 扫描 + 运行时反射 → Rust 过程宏 + 编译期 linkme；Spring 异常体系 → Rust 错误枚举；Spring 注解 → Rust 派生宏 + 过程宏属性。

状态图例：✅ 已迁移并有测试 / 🔶 语义等价但形态不同 / ⬜ 未迁移（路线图） / 🚫 不迁移 / 🆕 vernal-core 新增

---

## 一、错误体系（**vernal-core 核心改造点**）

### 1.1 错误枚举与三种形态

| Java 概念 | Spring 语义 | tx_di 语义 | vernal-core 实现 | 状态 |
|---|---|---|---|---|
| 嵌套运行时异常基类 | `NestedRuntimeException` + 子类异常（如 `BeansException`） | `AppError::Internal(anyhow::Error)` | `VernalError::Infrastructure(SharedError)` | 🆕 |
| 业务错误码（zero-alloc） | `ErrorCoded.getErrorCode() -> String` + 子类常量 | `AppError::ErrCode { domain, code, message }` | `VernalError::Business { domain, code, message }` | 🆕 |
| 业务错误 + 动态上下文 | 子类内部 `getMessage()` 拼接 | `AppError::WithContext { context: String }` | `VernalError::WithContext { context: String }` + `WithContextEntries { context: ErrorContext }` | 🆕 |
| `err.appendContext()` | `NestedRuntimeException.appendCurrentContext()` | `AppError::with_context(code, "...")` | `VernalError::with_context()` + `with_context_entries()` | 🆕 |
| 错误身份比较 | `instanceof` + `equals` | `PartialEq` 只比较 domain + code | `VernalError::PartialEq`（**只比较 domain + code**，忽略 context） | 🆕 |
| 错误分类（4 种） | `ErrorCoded` 子类层级 | — | `ErrorKind::{Business, Infrastructure, Validation, Internal}` | 🆕 |
| IO 异常嵌套 | `NestedIOException` | `From<io::Error>` | `From<io::Error> for VernalError` | 🆕 |
| 错误码字符串 | `ErrorCoded.getErrorCode() -> String` | `AppErrCode.code_msg()` | `ErrorCode::message()`（`&'static str`） | 🆕 |

### 1.2 错误访问器对应

| tx_di `AppError` 方法 | Spring 对应 | vernal-core `VernalError` 方法 | 状态 |
|---|---|---|---|
| `domain() -> &str` | `getMessage()` 中的前缀 | `domain() -> Option<&'static str>`（基础设施返回 `None`） | 🆕 |
| `code() -> i32` | 异常子类常量 | `code() -> Option<i32>` | 🆕 |
| `message() -> &str` | `getMessage()` | `message() -> &'static str`（基础设施返回 `"internal error"`） | 🆕 |
| `context() -> Option<&str>` | `NestedRuntimeException.getMessage()` 拼接 | `WithContext.context`（struct field）；`WithContextEntries.context.entries()` | 🆕 |
| `internal() -> Option<&anyhow::Error>` | — | （用 `std::error::Error::source()` 代替；保留 `is_infrastructure()` 快速判定） | 🆕 |
| `is_internal() -> bool` | — | `is_infrastructure() -> bool` | 🆕 |
| `is_same_kind(&Other) -> bool` | `equals` 仅识别同类 | `ErrorCode::is_same_kind()` 与 `VernalError::PartialEq` 协同 | 🆕 |
| `err_code() -> AppErrCode` | — | `ErrorCode::into_vernal_error()` | 🆕 |
| `full_message() -> String` | `getMessage()` + cause 拼接 | `Display` + `ErrorReport::display()` | 🆕 |
| `Display` → `"[DOMAIN:CODE] MESSAGE"` | Spring 默认 `getMessage()` 形式 | `Write!` 同样格式 `[{domain}:{code}] {message}` | 🆕 |
| `From<anyhow::Error>` | — | （不引入 anyhow；改用 `From<BoxError>` + `From<SharedError>`） | 🔶 |
| `From<&str>` / `From<String>` | `IllegalArgumentException(msg)` | `AppError::from("...")` | `From<String> for VernalError` + `From<&str> for VernalError`（**新增 S2**） | 🆕（待 S2） |
| `From<std::io::Error>` | `NestedIOException(cause)` | `From<io::Error>` | `From<io::Error> for VernalError` | 🆕 |
| `From<serde_json::Error>` / `From<toml::de::Error>` | `NestedRuntimeException` | `From<serde_json::Error>` | （**不直接 impl From**，避免 vernal-core 反向依赖 serde_json；让 `vernal-context` 转换） | 🔶 |

### 1.3 错误码 trait（`CodeMsg` / `ErrorCode`）

| tx_di `CodeMsg` trait | Spring 对应 | vernal-core `ErrorCode` trait | 状态 |
|---|---|---|---|
| `fn err_code(self) -> AppErrCode` | `ErrorCoded.getErrorCode(): String` | `fn into_vernal_error(self) -> VernalError`（消费 `self`，与 `domain()`/`code()`/`message()` 三个钩子配套） | 🆕 |
| 派生宏 | `@interface` + JavaDoc 文档 | `#[derive(ErrorCode)]`（vernal-macros 提供，对标 `#[derive(thiserror::Error)]`） | 🆕 |
| `#[err("DOMAIN")]` 域标注 | 子类命名 `XxxException` + Spring 内置约定 | enum 顶层 `#[err("domain")]` 属性 | 🆕 |
| `#[err(code, "message")]` 变体标注 | 子类 `private static final long serialVersionUID` | enum 变体 `#[err(-1, "组件未找到")]` | 🆕 |
| `is_same_kind` 跨枚举比较 | `instanceof` + equals | `ErrorCode::is_same_kind(&dyn ErrorCode)` | 🆕 |

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
| Spring 6.1 `ErrorResponseException` JSON 序列化 | — | `ErrorReport::to_json()` 返回 `serde_json::Value`（含 `domain/code/message/context_entries/kind`，不含 context 值） | 🬜（待 S10） |
| Spring Boot Actuator `/actuator/errors` | — | `ErrorReport::to_http_status()` 映射到 HTTP 状态码 | 🬜（待 S10 / 移交 vernal-web） |

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
| `"MIG"` | `org.springframework.jdbc.datasource.init` | `ErrorDomain::MIGRATION` | 🆕（待 S3） |
| `"SEC"` | `org.springframework.security` | `ErrorDomain::SECURITY` | 🆕（待 S3） |
| `"MTR"` | `org.springframework.boot.actuate.metrics` | `ErrorDomain::METRICS` | 🆕（待 S3） |

### 1.7 From 转换矩阵

| 源类型 | Spring 行为 | tx_di 行为 | vernal-core `From` 实现 | 状态 |
|---|---|---|---|---|
| `Box<dyn Error + Send + Sync>` | `new IllegalStateException(cause)` | `AppError::Internal(anyhow!(e))` | `VernalError::Infrastructure(Arc::new(e))` | ✅ |
| `Arc<dyn Error + Send + Sync>` | — | — | `VernalError::Infrastructure(arc)` | ✅ |
| `std::io::Error` | `NestedIOException` | `AppError::Internal(e.into())` | `VernalError::Infrastructure(Arc::new(io))` | ✅ |
| `serde_json::Error` | `NestedRuntimeException` | `From<serde_json::Error>` | （**不直接 impl From**，避免 vernal-core 反向依赖 serde_json；让 `vernal-context` 转换） | 🔶 |
| `toml::de::Error` | `NestedRuntimeException` | `From<toml::de::Error>` | （同上） | 🔶 |
| `anyhow::Error` | — | `From<anyhow::Error>` | （**故意不引入 anyhow**；vernal 沿用 `BoxError/SharedError` 让业务自行选择 anyhow；互转在 `vernal-bridge` 提供） | 🚫 |
| `tokio::task::JoinError` | — | — | `#[cfg(feature = "tokio")] impl From<JoinError> for VernalError` | 🆕（待 S2） |
| `String` / `&str` | `IllegalArgumentException(msg)` | `From<String> / From<&str>` | `From<String> / From<&str> for VernalError` | 🆕（待 S2） |

---

## 二、Ordered / init_sort 排序

| Spring `Ordered` | tx_di `init_sort() -> i32` | vernal-core `INIT_SORT_*` | 状态 |
|---|---|---|---|
| `HIGHEST_PRECEDENCE = Integer.MIN_VALUE` | `i32::MIN`（日志组件最先） | `INIT_SORT_INFRASTRUCTURE = i32::MIN + 1` + `HIGHEST_PRECEDENCE` 别名 | ✅ + 🆕（待 S4） |
| `BeanFactory` 初始化（早于所有业务） | — | `INIT_SORT_BEAN_FACTORY = i32::MIN + 2` | 🆕（待 S4） |
| 事件监听器注册 | — | `INIT_SORT_EVENT_LISTENER = -2_000_000_000` | 🆕（待 S4） |
| 消息源初始化 | — | `INIT_SORT_MESSAGE_SOURCE = -1_000_000_000` | 🆕（待 S4） |
| （业务默认） | `10000` | `INIT_SORT_BUSINESS = 0` | ✅ |
| （应用层最后） | `i32::MAX`（Web 服务器最后） | `INIT_SORT_APPLICATION = i32::MAX - 1` | ✅ |
| 异步任务 | — | `INIT_SORT_TASK = i32::MAX - 100` | 🆕（待 S4） |
| `LOWEST_PRECEDENCE = Integer.MAX_VALUE` | `i32::MAX` | `INIT_SORT_DEFAULT = i32::MAX` + `LOWEST_PRECEDENCE` 别名 | ✅ + 🆕（待 S4） |
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
| `SmartLifecycle.isAutoStartup()` | `Component::has_async_run` | （未对应，需要在 `Component` trait 加 `fn is_auto_startup()`） | — | ⬜ |

> **重命名建议**：vernal-core 中现有 `LifecyclePhase`（8 变体）应当重命名为 `AppLifecyclePhase`，避免与 `vernal-context::LifecyclePhase` 重名（详见《vernal-core 对象名称一致性检查 v2.0》S1 阶段）。

### 3.1 生命周期阶段 8 状态映射（SmartLifecycle → AppLifecyclePhase）

| SmartLifecycle 状态 | tx_di 阶段 | vernal-core `AppLifecyclePhase` 变体 | 说明 |
|---|---|---|---|
| （未创建） | （构造前） | `Created` | 刚构造完成，尚未初始化 |
| `isRunning() == false` + 正在 init | `init` | `Initializing` | `inner_init` 或 `initialize` 执行中 |
| `isRunning() == false` + init 完成 | `init` 完成 | `Initialized` | 同步初始化完成，尚未启动 |
| 正在 `start` | `async_init` | `Starting` | `start` 或 `async_run` 执行中 |
| `isRunning() == true` | `async_run` | `Started` | 所有启动钩子完成 |
| 正在 `stop(Runnable)` | `shutdown` | `Stopping` | `stop` 或 `shutdown` 执行中 |
| `isRunning() == false` + stop 完成 | `shutdown` 完成 | `Stopped` | 所有停止钩子完成 |
| 任何阶段失败 | panic | `Failed` | 启动失败 / 运行时 panic |

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
| `shortSummary()` | — | `short_summary() -> String`（**新增 S6**） | ⬜ |
| `prettyPrint(TimeUnit)` Spring 6.1+ 新增 | — | `pretty_print_with_unit(unit) -> String` + `StopWatchUnit` 枚举（**新增 S6**） | ⬜ |
| `start(long)` / `start(String, long)` | — | `start_with_ticks(name, ticks)`（**新增 S6**） | ⬜ |
| `getTaskInfo()` | — | `task_info_array() -> Vec<TaskInfo>`（**新增 S6**） | ⬜ |
| `StopWatchUnit.NANOSECONDS / MICROSECONDS / MILLISECONDS / SECONDS / MINUTES` | — | `StopWatchUnit::{Nano, Micro, Milli, Second, Minute}`（**新增 S6**） | ⬜ |

### 4.2 ConversionService

| Spring `ConversionService` 行为 | tx_di（无对应） | vernal-core `ConversionService` | 状态 |
|---|---|---|---|
| `boolean canConvert(sourceType, targetType)` | — | `can_convert::<T: Convertible>() -> bool`（**新增 S5**） | ⬜ |
| `<T> T convert(Object source, Class<T> targetType)` | — | `ConversionService::convert<T: Convertible>(value: &str) -> Result<T, ConversionError>` | ✅ |
| 运行时注册 Converter | — | （**未实现**，trait 静态分派替代；用户用过程宏或 trait impl 拓展） | 🚫 |
| Generics 转换：`List<String>` ↔ `List<Integer>` | — | （**未实现**，配置属性绑定只需标量转换） | 🚫 |
| 自定义 `Formatter` / `Printer` / `Parser` | — | （**未实现**，由 `Convertible` trait 替代） | 🚫 |

### 4.3 内置 Convertible 类型（v2.0 目标：10 个）

| Spring Converter | tx_di | vernal-core `impl Convertible for X` | 状态 |
|---|---|---|---|
| `StringToBooleanConverter` | — | `impl Convertible for bool`（接受 true/1/yes/on/false/0/no/off） | ✅ |
| `StringToNumberConverterFactory` | — | 13 个数字类型 `impl_convertible_for_number!` 宏（i8~i128、isize、u8~u128、usize、f32、f64） | ✅ |
| `StringToEnumConverterFactory` | — | `convert_enum<T: FromStr>()` 函数（对任何 enum + FromStr 都能复用） | ✅ |
| （无对应） | — | `impl Convertible for String`（恒等函数） | 🆕 |
| （无对应） | — | `impl<T: Convertible> Convertible for Option<T>`（空字符串 → None，非空 → Some(T)） | 🆕 |
| `StringToCharsetConverter` | — | （**未实现**，运行时不强制） | 🚫 |
| `StringToUUIDConverter` | — | `impl Convertible for uuid::Uuid`（feature = `"convert-uuid"`） | ⬜（待 S5） |
| `StringToDateConverter` / `StringToInstantConverter` | — | `impl Convertible for time::OffsetDateTime`（feature = `"convert-time"`） | ⬜（待 S5） |
| `StringToDurationConverter` | — | `impl Convertible for std::time::Duration`（std only，解析 `"30s"/"5m"`） | ⬜（待 S5） |
| `StringToURLConverter` | — | `impl Convertible for url::Url`（feature = `"convert-url"`） | ⬜（待 S5） |
| `StringToPathConverter` | — | `impl Convertible for std::path::PathBuf`（std only） | ⬜（待 S5） |
| `StringToSocketAddress` | — | `impl Convertible for std::net::SocketAddr`（std only） | ⬜（待 S5） |
| `StringToByteArrayConverter` | — | `impl Convertible for bytes::Bytes`（feature = `"convert-bytes"`） | ⬜（待 S5） |
| `NumberToNumberConverterFactory` | — | （**不实现**，原生 `as` cast 即可） | 🚫 |
| `DateToStringConverter` | — | （**不实现**，feature-gated chrono/time 互转） | 🚫 |
| `CollectionToCollectionConverter` | — | （**不实现**，性能敏感由 serde 或用户代码承担） | 🚫 |

---

## 五、tx_di 特有：inner_init / async_init 阶段映射

| tx_di 阶段 | vernal 替代方案 | 状态 |
|---|---|---|
| `Component::inner_init(&mut self, store: &Store)`（构造后立即同步，**可以访问 store**） | vernal-core 不提供 `inner_init`；vernal-beans `Component::build(deps, store)` 自带 store 引用，可以在这一步完成所有同步初始化 | 🔶 |
| `Component::init(app: &Arc<App>)`（App 阶段同步） | vernal-context `Lifecycle::initialize()`（async，签名上对齐） | 🔶 |
| `Component::async_init(app) -> BoxFuture` | `Lifecycle::initialize() -> LifecycleFuture<'_>`（仅 async） | 🔶 |
| `Component::async_run(app, token)`（后台 task） | `Lifecycle::start(token)`（async），由 `vernal-context::application_startup_coordinator` 包装为 `tokio::spawn` | 🔶 |
| `Component::shutdown(&self)`（同步） | `Lifecycle::stop()`（async） | 🔶 |
| `ComponentMeta.has_async_run` 跳过未覆写 | `Lifecycle::start` 同样有默认空实现，调度器跳过未实现的组件 | ✅ |
| `tx_di_core::DepsTuple` 元组（最多 16 个 Arc 依赖） | vernal-beans 通过字段类型解析，无上限 | ✅ |

---

## 六、Rust 生态集成（基于 crates.io 调研的最终方案）

vernal-core 必须保持**默认零外部依赖**。本节列出经过实际调研的集成策略。

### 6.1 选型矩阵（canonical picks）

| 关注点 | 推荐 crate | 版本 | 许可证 | 整合方式 | 阶段 |
|---|---|---|---|---|---|
| 错误类型派生 | **`thiserror`** | `2.0.19` | MIT/Apache-2.0 | 文档推荐使用（业务 crate 自行 `#[derive(thiserror::Error)]`），vernal-core 不反向依赖 | S8 |
| 通用错误捕获（escape hatch） | **`anyhow`** | `1.0.104` | MIT/Apache-2.0 | **不在 vernal-core 依赖**，**但** vernal-core 的 `BoxError` / `SharedError` 与 anyhow 互转在 `vernal-bridge` 提供 | — |
| 深度错误链 + 上下文 | **`error-stack`** | `0.8.0` | MIT/Apache-2.0 | 可选（与 `thiserror` 互斥，`vernal-context` 集成） | — |
| checked 错误选项 | **`snafu`** | `0.9.2` (MSRV 1.81) | MIT/Apache-2.0 | 不采用（MSRV 高，且需要 selector structs，不符合 Rust 习惯） | — |
| 宏拼接 | **`pastey`** | `0.2.3` | MIT/Apache-2.0 | feature-gated `macros` | S8 |
| 分布式注册 | **`inventory`** | `0.3.24` | MIT/Apache-2.0 | 可选替代 linkme（仅在需要 dynamic lib 插件时引入） | S8 |
| 框架级单例 | **`once_cell`** | `1.21.4` | MIT/Apache-2.0 | feature-gated `once-cell` | S8 |
| 派生补充 | **`derive_more`** | `2.1.1` | MIT | feature-gated `derive-extras` | S8 |
| UUID v4 / v7 | **`uuid`** | `1.24.0` | MIT/Apache-2.0 | feature-gated `id/uuid_id.rs`（feature = `"id-uuid"`） | S7+S8 |
| ULID | **`ulid`** | `3.0.0` | MIT | 同上（feature = `"id-ulid"`） | S7+S8 |
| NanoId | **`nanoid`** | `0.5.0` | MIT | 同上（feature = `"id-nanoid"`，用于短 ID） | S7+S8 |
| Snowflake | **自实现** | — | — | 基于 `uuid v7` + epoch（**不引入** `snowflake` 1.3.0 已废弃） | S7 |
| 高精度 / WASM 时间 | **`web-time`** | `1.1.0` | MIT/Apache-2.0 | **替换**`std::time::Instant`，`StopWatch::new` 改为 `web_time::Instant::now()`（feature = `"time-web"`） | S8 |
| 日期时间 | **`time`** | `0.3.54` | MIT/Apache-2.0 | feature-gated `convert-time`（MSRV 1.88 完美对齐） | S5+S8 |
| 日期时间（备选） | **`chrono`** | `0.4.45` | MIT/Apache-2.0 | feature-gated `convert-chrono`（默认 features 关闭 + 仅启用 `clock` + `std`） | S5+S8 |
| URL | **`url`** | `2.5.8` | MIT/Apache-2.0 | feature-gated `convert-url` | S5+S8 |
| 字节 | **`bytes`** | `1.12.1` | MIT | feature-gated `convert-bytes` | S5+S8 |
| 跨大小写转换 | **`convert_case`** | `0.11.0` | MIT | feature-gated `case-conv` | S8 |
| 序列化 | **`serde`** | `1.0.229` | MIT/Apache-2.0 | 仅对 `ObjectId` / `VernalError` / `ErrorReport` 等用户可见类型做 derive | S7+S8 |
| 插入序 Map | **`indexmap`** | `2.14.0` | MIT/Apache-2.0 | **不在 vernal-core**（仅 `vernal-beans` 等上层使用） | — |
| 全局类型注册（与 linkme 并列） | **`inventory`** | `0.3.24` | MIT/Apache-2.0 | 可选替代 linkme（仅在需要 dynamic lib 插件时引入） | S8 |
| async fn in dyn traits | **`async-trait`** | `0.1.91` | MIT/Apache-2.0 | **不引入**（Rust 1.75+ 已有原生支持；vernal-core 不持有 dyn trait） | — |
| 日志 / 链路追踪 | **`tracing`** | `0.1.44` | MIT | **不引入**（归 `vernal-log`） | — |

### 6.2 显式排除的 crate

| 排除 | 原因 |
|---|---|
| `priority-queue` 2.7.0 | **LGPL-3.0 OR MPL-2.0 copyleft**，污染 Vernal 的 MIT 协议 |
| `instant` 0.1.13 | **已停止维护**（README 推荐 fork 或用 web-time） |
| `snowflake` 1.3.0 | 2017 年最后更新，已废弃 |
| `snowflake-rs` 0.1.1 | 自 2018 起废弃，依赖过时的 `time` 0.1 |
| `idgen` 0.1.2 | 2019 年最后更新，孤儿 crate |
| `smartstring` 1.0.1 | **MPL-2.0+ 许可证** 与 MIT 项目兼容性需逐文件评估 |
| `paste` 1.0.15 | 已被 `pastey` 替代 |
| `anyhow` | 与 vernal `SharedError` 体系冲突 |
| `tokio` | runtime 耦合，不应出现在 vernal-core |
| `bson` 3.1.0 | 引入整个 MongoDB 驱动符号，ObjectId 应自行实现 |
| `redis` 1.4.1 | 职责不属于 core 层 |
| `linkme` | 链接器黑魔法，cross-compile 痛点；备选 inventory |
| `conv` / `conv2` / `easy-cast` / `num_convert` | 都是数值转换 traits，**抽象错位**（Spring ConversionService 是 String → T 转换） |
| `qbench` | crates.io 上找不到具体包；改用 `web-time::Instant` + 手动聚合 |

### 6.3 集成原则

1. **vernal-core 保持零外部依赖**（默认 `cargo build`）：所有上述 crate 均作为 **可选 feature flag** 提供
2. **新 feature flag 命名**：经过 CRUD 测试的统一前缀：
   - `feature = "id-uuid"` / `"id-ulid"` / `"id-nanoid"`（ID 生成器后端）
   - `feature = "convert-uuid"` / `"convert-url"` / `"convert-time"` / `"convert-chrono"` / `"convert-bytes"`（类型转换器）
   - `feature = "case-conv"`（enum 大小写无关）
   - `feature = "serde"`（序列化）
   - `feature = "time-web"`（WASM 时间）
   - `feature = "error-derive"`（thiserror 派生宏提示）
   - `feature = "macros"`（pastey）
   - `feature = "once-cell"`（once_cell）
   - `feature = "derive-extras"`（derive_more）
3. **`VernalError::Infrastructure` 选用 `Arc<dyn Error>`**：与 Spring `NestedRuntimeException` 语义一致，**不**把 anyhow 拉进 vernal-core 的依赖图
4. **`BoxError` / `SharedError` 与 anyhow 互转**：在 `vernal-bridge` crate 提供 `impl From<anyhow::Error> for BoxError`、`From<BoxError> for anyhow::Error`，确保业务 crate 即便已用 anyhow 也能 0 成本接入 vernal-core
5. **`web_time::Instant` 替换 `std::time::Instant`**（**仅 feature = "time-web"** 时）：让 vernal-core **目标**支持 `wasm32-unknown-unknown`（虽然 vernal 暂时不发布 wasm 版）。`web-time` 在 native 是 `std::time::Instant` 的 alias，零额外开销
6. **`chrono` 必须 `default-features = false`**：避免 `iana-time-zone` 时区数据膨胀

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

### 6.5 建议的 `Cargo.toml` 增量变更（v2.0 完整版）

```toml
[package]
name = "vernal-core"
version.workspace = true
edition.workspace = true       # 2024
rust-version.workspace = true  # 1.88
license.workspace = true       # MIT
description = "Stable shared contracts for the Vernal framework."
publish = false

[dependencies]
# workspace level（已有）
linkme = { workspace = true }

# Tier 1：强烈推荐（基础设施）
thiserror = { version = "2.0", optional = true }
pastey = { version = "0.2", optional = true }
once_cell = { version = "1.21", optional = true }
inventory = { version = "0.3", optional = true }

# Tier 2：强烈推荐（用户可见类型）
uuid = { version = "1.24", optional = true, features = ["v4", "v7"] }
ulid = { version = "3.0", optional = true }
nanoid = { version = "0.5", optional = true }
url = { version = "2.5", optional = true }
bytes = { version = "1.12", optional = true }
time = { version = "0.3", optional = true, features = ["formatting", "parsing"] }
chrono = { version = "0.4", optional = true, default-features = false, features = ["clock", "std"] }
serde = { version = "1", optional = true }
derive_more = { version = "2.1", optional = true, features = ["display", "from", "error"] }

# Tier 3：可选（特定场景）
convert_case = { version = "0.11", optional = true }
web-time = { version = "1.1", optional = true, default-features = false }

[features]
default = []

# Tier 1
error-derive = ["dep:thiserror"]
macros = ["dep:pastey"]
once-cell = ["dep:once_cell"]
registry = ["dep:inventory"]

# Tier 2
id-uuid = ["dep:uuid"]
id-ulid = ["dep:ulid"]
id-nanoid = ["dep:nanoid"]
convert-uuid = ["id-uuid"]
convert-url = ["dep:url"]
convert-time = ["dep:time"]
convert-chrono = ["dep:chrono"]
convert-bytes = ["dep:bytes"]
serde = ["dep:serde"]
derive-extras = ["dep:derive_more"]

# Tier 3
case-conv = ["dep:convert_case"]
time-web = ["dep:web-time"]
```

> **最终编译目标**：
> - `cargo build -p vernal-core` 默认零外部依赖通过
> - `cargo build -p vernal-core --features "id-uuid,serde"` 增加 2 个依赖，启用 `ObjectId` 的 UUID v7 + JSON 序列化
> - 整个 vernal workspace 仅在 `vernal-bridge` / `vernal-context` 等上层按需启用 `chrono` / `thiserror`，**vernal-core 自身不依赖**

---

## 七、规范与命名一致性

| 来源 | 类型命名 | vernal-core 处理 |
|---|---|---|
| Spring `ConversionService` / `Ordered` / `Lifecycle` / `SmartLifecycle` | 全大驼峰 + 动词/名词 | 全部保留大驼峰（`ConversionService`、`Ordered`、`Lifecycle`、`SmartLifecycle`） |
| tx_di `Component` / `Scope` / `Store` / `BuildContext` / `App` | 同上 | 大驼峰，**但 `App` → `ApplicationContext`**（迁到 `vernal-context`） |
| tx_di `AppError` / `AppErrCode` / `AppResult` | `AppXxx` 命名族 | vernal 改为 `VernalError` / `ErrorCode` / **保留** `BoxError` / **`SharedError`** 反而是 vernal 新增 |
| tx_di `CodeMsg` | 短名 | 改为 `ErrorCode`（语义更广，包括 domain/code/message + into_vernal_error） |
| Rust 标准 `Init` / `Run` / `Stop` | 全小写动词 | vernal 用同样的小写动词：`start`、`initialize`、`stop`、`shutdown` |
| Spring 6.1 `Observation` | 大驼峰 | vernal 轻量版命名为 `Span`（避免与 `tokio` 等三方库命名冲突） |

---

## 八、目标文件分布（v2.0）

迁移完成后，vernal-core 应该新增下列子模块（**目标**）

```
crates/vernal-core/src/
├── lib.rs                              # FRAMEWORK_VERSION 等常量
├── failure.rs                          # BoxError / SharedError（已有）
├── app_lifecycle_phase.rs              # 🆕 重命名自 lifecycle_phase.rs（8 变体）
├── ordered.rs                          # INIT_SORT_* 常量（已有）+ 4 个新锚点 + HIGHEST_PRECEDENCE 别名
├── convert/
│   ├── mod.rs                          # ConversionService + can_convert + Convertible trait（已有）
│   ├── converter.rs                    # Converter<S, T> trait（已有）
│   ├── boolean_converter.rs            # bool（已有）
│   ├── number_converter.rs             # 13 数字类型（已有）
│   ├── string_converter.rs             # String（已有）
│   ├── option_converter.rs             # Option<T>（已有）
│   ├── enum_converter.rs               # convert_enum<T: FromStr>（已有）
│   ├── path_converter.rs               # 🆕 PathBuf（可选 feature = "convert-path"，std only）
│   ├── datetime_converter.rs           # 🆕 OffsetDateTime（可选 feature = "convert-time"）
│   ├── duration_converter.rs           # 🆕 Duration（std only）
│   ├── url_converter.rs                # 🆕 url::Url（可选 feature = "convert-url"）
│   ├── socket_addr_converter.rs        # 🆕 SocketAddr（std only）
│   └── bytes_converter.rs              # 🆕 bytes::Bytes（可选 feature = "convert-bytes"）
├── error/
│   ├── mod.rs                          # 重新导出（已有）
│   ├── vernal_error.rs                 # 4 变体 enum + From<String/&str>（已有 + S2 扩展）
│   ├── error_code.rs                   # ErrorCode trait + is_same_kind（已有）
│   ├── error_kind.rs                   # 4 分类（已有）
│   ├── error_domain.rs                 # 13 个子系统域常量（已有 10 + S3 新增 3）
│   ├── error_context.rs                # ErrorContext 键值对（已有）
│   ├── error_report.rs                 # 脱敏报告 + to_json（S10）
│   └── from_io.rs                      # 🆕 From<std::io::Error>（独立文件以便 feature gating）
├── id/
│   ├── mod.rs                          # 重新导出（已有）
│   ├── object_id.rs                    # ObjectId（已有）
│   ├── id_generator.rs                 # 🆕 IdGenerator trait（统一抽象）
│   ├── uuid_id.rs                      # 🆕 UUID v4/v7（可选 feature = "id-uuid"）
│   ├── ulid_id.rs                      # 🆕 ULID（可选 feature = "id-ulid"）
│   ├── nanoid_id.rs                    # 🆕 NanoId（可选 feature = "id-nanoid"）
│   ├── snowflake_id.rs                 # 🆕 自实现 Snowflake（基于 uuid v7 + epoch，无外部依赖）
│   └── serde_object_id.rs              # 🆕 serde 支持（可选 feature = "serde"）
├── time/
│   ├── mod.rs                          # 重新导出（已有）
│   ├── stop_watch.rs                   # StopWatch + short_summary + pretty_print_with_unit（S6 扩展）
│   └── stop_watch_unit.rs              # 🆕 StopWatchUnit 枚举（Nano/Micro/Milli/Second/Minute）
└── diagnostics/                        # 🆕 诊断模块（待 S9）
    ├── mod.rs                          # 模块入口
    ├── span.rs                         # Span 类型 + SpanReport
    ├── span_id.rs                      # SpanId（基于 ObjectId）
    └── attribute_value.rs              # AttributeValue 枚举（String/Int/Float/Bool）
```

**新增 .rs 文件数**：13
**修改 .rs 文件数**：6（重命名 `lifecycle_phase.rs` 为 `app_lifecycle_phase.rs`；`ordered.rs` 增加 4 锚点 + 2 别名；`stop_watch.rs` 增加 4 方法；`error_domain.rs` 增加 3 域；`error_report.rs` 增加 to_json；`vernal_error.rs` 增加 From<String/&str>）

---

## 九、验收清单（迁移完成时所有条目须 ✅）

- [ ] `vernal_core::VernalError` 测试覆盖 4 个变体的 Display/Debug/PartialEq/from_code/from_io（0 anyhow）
- [ ] `vernal_core::error::ErrorCode` trait 的 `is_same_kind()` 测试通过
- [ ] `vernal_core::error::ErrorReport::from_error(&internal_err)` 不泄露源错误链到 Display（防信息泄露）
- [ ] `vernal_core::INIT_SORT_INFRASTRUCTURE < INIT_SORT_BEAN_FACTORY < INIT_SORT_EVENT_LISTENER < INIT_SORT_MESSAGE_SOURCE < INIT_SORT_BUSINESS < INIT_SORT_APPLICATION < INIT_SORT_TASK < INIT_SORT_DEFAULT` 常量关系保持
- [ ] `vernal_core::HIGHEST_PRECEDENCE == INIT_SORT_INFRASTRUCTURE` + `LOWEST_PRECEDENCE == INIT_SORT_DEFAULT` Spring 风格别名成立
- [ ] `vernal_core::AppLifecyclePhase`（**已重命名**）的 8 个变体映射 Spring `SmartLifecycle` 8 状态
- [ ] `vernal_core::Convertible` 的 10 个内置实现（bool / 数字 / String / Option / enum / PathBuf / Duration / SocketAddr + OffsetDateTime / URL 各 feature-gated）全部通过 round-trip 测试
- [ ] `vernal_core::ConversionService::can_convert::<T>()` 判定 10 个内置类型返回正确
- [ ] `vernal_core::StopWatch::short_summary()` 输出与 Spring 6.1 `prettyPrint(TimeUnit.NANOSECONDS)` 字段对齐
- [ ] `vernal_core::StopWatch::pretty_print_with_unit(StopWatchUnit::Nano)` 输出含 ns 精度
- [ ] `vernal_core::diagnostics::Span`（S9）创建 / 嵌套 / 错误传播 / 退出测试通过
- [ ] `vernal_core` 在 `cargo build --no-default-features` 下零外部依赖通过
- [ ] `vernal_core::From<std::io::Error> for VernalError` 与 Spring `NestedIOException` 同语义（IO → Infrastructure）
- [ ] `vernal_core::From<String> for VernalError` + `From<&str> for VernalError` 与 tx_di `AppError::from("...")` 同语义
- [ ] `vernal_core::ObjectId` 与 4 个 feature-gated ID 后端（uuid/ulid/nanoid/snowflake）通过 `IdGenerator` trait 统一抽象
- [ ] `vernal_core::ObjectId` serde `Serialize`/`Deserialize` round-trip 测试通过（feature = "serde"）
- [ ] 所有公开类型均有中文 `///` rustdoc 注释（覆盖率 100%）
- [ ] `cargo doc -p vernal-core --no-deps` 无警告
- [ ] `cargo clippy -p vernal-core --all-targets -- -D warnings` 无警告