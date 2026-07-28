# vernal-core 技术交接文档

> **对标**: Spring Framework `spring-core`（基础合同层）
> **crate**: `vernal-core`
> **现状**: 82 文件 / ~18482 行 / 16 子模块
> **edition**: 2024 / rustc 1.88.0 / MSRV 1.88.0
> **状态**: experimental
> **最后更新**: 2026-07-28

---

## 一、定位

vernal-core 是 Vernal 框架的**基础合同层**，对标 Spring Framework 的 `spring-core` 模块。
它定义了所有上层 crate（vernal-ioc、vernal-aop、vernal-context 等）共享的稳定 trait、
错误类型、类型转换、资源抽象和环境配置等基础设施契约。

**设计原则**:

- 零 `unsafe`（lib.rs 中 `#![allow(unused)]` 但不引入 unsafe 块）
- 所有公开 trait 必须是 dyn-compatible（object-safe）
- 所有公开类型必须满足 `Send + Sync + 'static`
- feature-gated 设计：核心 trait 默认可用，具体实现按 feature flag 启用
- 与 Spring API 的映射关系在文档和注释中保持一致

**与 Spring Core 的对应关系**:

| Spring Core 子系统 | vernal-core 子模块 | 状态 |
|---|---|---|
| `Ordered` / `PriorityOrdered` | `ordered` | 完成 |
| `NestedRuntimeException` | `error` | 完成 |
| `ConversionService` | `convert` | 完成 |
| `jackson-databind` / `woodstox` | `serialization` | 基础完成 |
| `Resource` / `ResourceLoader` | `resource` | 完成 |
| `Environment` / `PropertySource` | `environment` | 完成 |
| `TaskExecutor` / `AsyncTaskExecutor` | `task` | 完成 |
| `commons-logging` | `logging` | 完成 |
| `codec.Encoder/Decoder` | `codec` | 骨架 |
| `reactor-core` / `rxjava` | `async_runtime` | 骨架 |
| `SmartLifecycle` | `app_lifecycle_phase` | 完成 |
| `DataSize` / `DataUnit` | `util/unit` | 完成 |

---

## 二、组件清单详细

### 2.1 Ordered 排序

**对标**: `org.springframework.core.Ordered` / `PriorityOrdered`

#### Spring API

```java
public interface Ordered {
    int HIGHEST_PRECEDENCE = Integer.MIN_VALUE;
    int LOWEST_PRECEDENCE = Integer.MAX_VALUE;
    int getOrder();
}

public interface PriorityOrdered extends Ordered {
    // 标记接口：PriorityOrdered 总是排在 Ordered 之前
}
```

#### Rust 实现

```rust
// crates/vernal-core/src/ordered.rs

pub const HIGHEST_PRECEDENCE: i32 = i32::MIN + 1;  // -2147483647
pub const LOWEST_PRECEDENCE: i32 = i32::MAX;        //  2147483647

// 标准层级常量
pub const INIT_SORT_INFRASTRUCTURE: i32 = i32::MIN + 1;
pub const INIT_SORT_BEAN_FACTORY:   i32 = i32::MIN + 2;
pub const INIT_SORT_EVENT_LISTENER: i32 = -2_000_000_000;
pub const INIT_SORT_MESSAGE_SOURCE: i32 = -1_000_000_000;
pub const INIT_SORT_BUSINESS:       i32 = 0;
pub const INIT_SORT_APPLICATION:    i32 = i32::MAX - 100;
pub const INIT_SORT_TASK:           i32 = i32::MAX - 1;
pub const INIT_SORT_DEFAULT:        i32 = i32::MAX;

pub trait Ordered {
    fn order(&self) -> i32 { 0 }
    fn is_priority_ordered(&self) -> bool { false }
}

pub trait PriorityOrdered {
    fn order(&self) -> i32 { 0 }
    fn is_priority_ordered(&self) -> bool { true }
}

pub struct OrderComparator;

impl OrderComparator {
    pub fn compare(a_priority: bool, a_order: i32,
                   b_priority: bool, b_order: i32) -> std::cmp::Ordering;
    pub fn sort<T, F, G>(items: &mut [T], get_priority: F, get_order: G)
    where F: Fn(&T) -> bool, G: Fn(&T) -> i32;
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| dyn-compatible | `Ordered` 和 `PriorityOrdered` 均 object-safe | 满足 |
| Send + Sync | trait 方法仅 `&self` | 满足 |
| 默认值 | `order()` 默认返回 0 | 满足 |
| 偏序不变量 | INFRA < BEAN_FACTORY < EVENT < MSG < BIZ < APP < TASK < DEFAULT | 测试覆盖 |
| PriorityOrdered 优先 | 类型级标记，总是排在 Ordered 之前 | 满足 |

#### 待补齐

- [ ] `OrderComparator` 实现 `std::cmp::Ord` trait（目前仅提供静态方法）

---

### 2.2 错误体系

**对标**: `NestedRuntimeException` / `ErrorCode` / `AppError`（tx_di）

#### Spring API

```java
public class NestedRuntimeException extends RuntimeException {
    public NestedRuntimeException(String msg);
    public NestedRuntimeException(String msg, Throwable cause);
    @Nullable
    public Throwable getRootCause();
}

// tx_di AppError 模式
public class AppError {
    String domain;   // "ioc", "aop", "context"
    int code;        // 负数=框架错误, 正数=业务错误
    String message;  // 静态描述
}
```

#### Rust 实现

```rust
// crates/vernal-core/src/failure.rs
pub type BoxError    = Box<dyn Error + Send + Sync + 'static>;
pub type SharedError = Arc<dyn Error + Send + Sync + 'static>;

// crates/vernal-core/src/error/vernal_error.rs
#[derive(Debug)]
pub enum VernalError {
    Business {
        domain: &'static str,
        code: i32,
        message: &'static str,
    },
    WithContext {
        domain: &'static str,
        code: i32,
        message: &'static str,
        context: String,
    },
    WithContextEntries {
        domain: &'static str,
        code: i32,
        message: &'static str,
        context: ErrorContext,
    },
    Infrastructure(SharedError),
}

impl VernalError {
    pub const fn business(domain: &'static str, code: i32,
                          message: &'static str) -> Self;
    pub fn with_context(domain: &'static str, code: i32,
                        message: &'static str, context: impl Into<String>) -> Self;
    pub fn infrastructure<E: Error + Send + Sync + 'static>(error: E) -> Self;
    pub fn domain(&self) -> Option<&'static str>;
    pub fn code(&self) -> Option<i32>;
    pub fn message(&self) -> &'static str;
    pub fn is_infrastructure(&self) -> bool;
    pub fn report(&self) -> ErrorReport;
}

// crates/vernal-core/src/error/error_code.rs
pub trait ErrorCode: Send + Sync + 'static {
    fn domain(&self) -> &'static str;
    fn code(&self) -> i32;
    fn message(&self) -> &'static str;
    fn into_vernal_error(self) -> VernalError where Self: Sized;
    fn is_same_kind(&self, other: &dyn ErrorCode) -> bool;
}

// crates/vernal-core/src/error/error_context.rs
pub struct ErrorContext { entries: Vec<(&'static str, String)> }
impl ErrorContext {
    pub const fn new() -> Self;
    pub fn with(mut self, key: &'static str, value: impl Into<String>) -> Self;
    pub fn entries(&self) -> &[(&'static str, String)];
}

// crates/vernal-core/src/error/error_domain.rs
pub struct ErrorDomain;
impl ErrorDomain {
    pub const IOC: &'static str = "ioc";
    pub const AOP: &'static str = "aop";
    pub const CONTEXT: &'static str = "context";
    pub const WEB: &'static str = "web";
    pub const HTTP: &'static str = "http";
    pub const TOWER: &'static str = "tower";
    pub const DISCOVERY: &'static str = "discovery";
    pub const MACROS: &'static str = "macros";
    pub const CORE: &'static str = "core";
    pub const BRIDGE: &'static str = "bridge";
    pub const MIGRATION: &'static str = "migration";
    pub const SECURITY: &'static str = "security";
    pub const METRICS: &'static str = "metrics";
    pub const fn all() -> &'static [&'static str];  // 13 个域
}

// crates/vernal-core/src/error/error_kind.rs
pub enum ErrorKind { Business, Infrastructure, Validation, Internal }

// crates/vernal-core/src/error/error_report.rs
pub struct ErrorReport { /* 脱敏快照，不暴露 context 值 */ }
impl ErrorReport {
    pub fn from_error(error: &VernalError) -> Self;
    pub fn domain(&self) -> &'static str;
    pub fn code(&self) -> i32;
    pub fn message(&self) -> &'static str;
    pub fn context_entries(&self) -> usize;
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| Send + Sync | `VernalError` 编译期断言 | 满足（测试验证） |
| 零分配 Business 变体 | domain/code/message 均为 `&'static str` / `i32` | 满足 |
| 跨变体 PartialEq | 按 domain + code 比较，Infrastructure 按 Arc 指针 | 满足 |
| Display 脱敏 | `WithContextEntries` 不暴露 entry 值 | 满足（测试覆盖） |
| ErrorReport 脱敏 | Infrastructure 仅暴露 "internal error" | 满足（测试覆盖） |
| BoxError 双向转换 | `From<BoxError> for VernalError` + blanket `From<VernalError> for BoxError` | 满足 |
| thiserror 集成 | feature `error-derive` 可选启用 | 可选 |

#### 待补齐

- [ ] `From<VernalError> for std::io::Error` 反向转换
- [ ] `ErrorCode` 的 `#[derive(ErrorCode)]` 宏实现（在 vernal-macros 中）

---

### 2.3 类型转换

**对标**: `org.springframework.core.convert.ConversionService` / `Converter<S,T>`

#### Spring API

```java
public interface Converter<S, T> {
    T convert(S source);
}

public interface GenericConverter {
    Set<ConvertiblePair> getConvertibleTypes();
    Object convert(Object source, TypeDescriptor sourceType, TypeDescriptor targetType);
}

public interface ConditionalConverter {
    boolean matches(TypeDescriptor sourceType, TypeDescriptor targetType);
}

public interface ConversionService {
    boolean canConvert(Class<?> sourceType, Class<?> targetType);
    <T> T convert(Object source, Class<T> targetType);
}
```

#### Rust 实现

```rust
// crates/vernal-core/src/convert/converter.rs
pub trait Converter<S, T> {
    fn convert(&self, source: S) -> Result<T, ConversionError>;
}

// crates/vernal-core/src/convert/conditional_converter.rs
pub trait ConditionalConverter: Send + Sync {
    fn matches(&self, source_type: TypeId, target_type: TypeId) -> bool;
}

// crates/vernal-core/src/convert/generic_converter.rs
pub struct ConvertiblePair { pub source: TypeId, pub target: TypeId }
pub trait GenericConverter: ConditionalConverter {
    fn convertible_types(&self) -> HashSet<ConvertiblePair>;
    fn convert(&self, source: &str, target_type: TypeId) -> Result<String, ConversionError>;
}

// crates/vernal-core/src/convert/mod.rs
pub trait Convertible: Sized {
    fn from_str_value(value: &str) -> Result<Self, ConversionError>;
}

pub struct ConversionService;
impl ConversionService {
    pub fn convert<T: Convertible>(value: &str) -> Result<T, ConversionError>;
    pub fn can_convert<T: Convertible>() -> bool;
    pub fn get_shared_instance() -> &'static Self;
}

// crates/vernal-core/src/convert/converter_registry.rs
pub trait ConverterRegistry: Send + Sync {
    fn add_converter(&self, source_type: TypeId, target_type: TypeId,
                     converter: Box<dyn Fn(&str) -> Result<String, ConversionError>
                                    + Send + Sync>);
    fn remove_convertible(&self, source_type: TypeId, target_type: TypeId);
    fn can_convert(&self, source_type: TypeId, target_type: TypeId) -> bool;
}

pub struct TypeIdConverterRegistry;  // Mutex<HashMap<(TypeId,TypeId), Box<dyn Fn>>>
```

#### 内置转换器（10 种）

| 转换器 | 源→目标 | feature |
|---|---|---|
| `BooleanConverter` | `&str` → `bool`（true/1/yes/on） | 默认 |
| `NumberConverter` | `&str` → i8..i128, u8..u128, f32/f64 | 默认 |
| `StringConverter` | `&str` → `String`（恒等） | 默认 |
| `EnumConverter` | `&str` → `T: FromStr` | 默认 |
| `OptionConverter` | 空字符串 → `None` | 默认 |
| `PathConverter` | `&str` → `PathBuf` | 默认 |
| `DurationConverter` | `&str` → `Duration`（ISO-8601 + 简化语法） | 默认 |
| `SocketAddrConverter` | `&str` → `SocketAddr` | 默认 |
| `UrlConverter` | `&str` → `Url` | `convert-url` |
| `RegexConverter` | `&str` → `Regex` | `convert-regex` |

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| dyn-compatible | `Converter<S,T>` 因泛型参数不可 dyn | 已知限制 |
| `ConditionalConverter` dyn-compatible | 仅 `&self` 方法 | 满足 |
| `ConverterRegistry` dyn-compatible | 仅 `&self` 方法 | 满足 |
| Send + Sync | `TypeIdConverterRegistry` 通过 `Mutex` 保护 | 满足 |
| TypeId 替代 Class<?> | 运行时类型键使用 `std::any::TypeId` | 满足 |
| 零分配静态路径 | `Convertible` 基于 `from_str_value` | 满足 |

#### 待补齐

- [ ] `GenericConverter` 的 blanket impl（为 `Converter<S,T>` 自动实现）
- [ ] `chrono::NaiveDateTime` 转换器（feature `convert-chrono`）
- [ ] `Uuid` / `Bytes` 转换器完善

---

### 2.4 序列化

**对标**: `jackson-databind`（JSON）/ `woodstox-core`（XML）

#### Spring API

```java
// jackson-databind
ObjectMapper mapper = new ObjectMapper();
String json = mapper.writeValueAsString(object);
<T> T mapper.readValue(String json, Class<T> valueType);

// woodstox-core (XML)
XmlMapper xmlMapper = new XmlMapper();
String xml = xmlMapper.writeValueAsString(object);
```

#### Rust 实现

```rust
// crates/vernal-core/src/serialization/mod.rs
#[derive(Debug, Clone)]
pub enum SerializationError {
    Json(String),
    Xml(String),
    Other(String),
}

// feature = "json"
pub mod json {
    pub struct JsonCodec;
    impl JsonCodec {
        pub fn new() -> Self;
        pub fn to_string<T: serde::Serialize>(&self, value: &T)
            -> Result<String, SerializationError>;
        pub fn to_string_pretty<T: serde::Serialize>(&self, value: &T)
            -> Result<String, SerializationError>;
        pub fn from_str<T: serde::de::DeserializeOwned>(&self, json: &str)
            -> Result<T, SerializationError>;
        pub fn to_vec<T: serde::Serialize>(&self, value: &T)
            -> Result<Vec<u8>, SerializationError>;
        pub fn from_slice<T: serde::de::DeserializeOwned>(&self, slice: &[u8])
            -> Result<T, SerializationError>;
    }
}

// feature = "xml"
pub mod xml {
    pub struct XmlCodec;
    impl XmlCodec {
        pub fn new() -> Self;
        pub fn to_string<T: serde::Serialize>(&self, value: &T)
            -> Result<String, SerializationError>;
        pub fn from_str<T: serde::de::DeserializeOwned>(&self, xml: &str)
            -> Result<T, SerializationError>;
    }
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| serde 统一 trait | 所有 Codec 基于 `serde::Serialize` / `DeserializeOwned` | 满足 |
| feature-gated | `json` / `xml` 独立启用 | 满足 |
| 错误类型统一 | `SerializationError` 包含 Json/Xml/Other 三变体 | 满足 |
| YAML 支持 | `serde_yaml_ng` 后端 | 待实现 |

#### 待补齐

- [ ] `YamlCodec`（feature = "yaml"，后端 `serde_yaml_ng`）
- [ ] `PropertiesCodec`（feature = "properties"，后端 `java-properties`）

---

### 2.5 资源加载

**对标**: `org.springframework.core.io.Resource` / `ResourceLoader`

#### Spring API

```java
public interface Resource extends InputStreamSource {
    boolean exists();
    boolean isReadable();
    String getFilename();
    String getDescription();
    InputStream getInputStream() throws IOException;
}

public interface ResourceLoader {
    Resource getResource(String location);
    // "classpath:", "file:", "http://"
}
```

#### Rust 实现

```rust
// crates/vernal-core/src/resource/mod.rs
pub trait Resource: Send + Sync {
    fn exists(&self) -> bool;
    fn is_readable(&self) -> bool;
    fn filename(&self) -> Option<&str>;
    fn description(&self) -> String;
    fn read_bytes(&self) -> std::io::Result<Vec<u8>>;
    fn read_string(&self) -> std::io::Result<String> { /* 默认实现 */ }
}

pub trait ResourceLoader: Send + Sync {
    type Resource: Resource;
    fn load(&self, location: &str) -> std::io::Result<Self::Resource>;
}

// 内置实现
pub struct FileSystemResource { path: PathBuf }
pub struct ClassPathResource { path: String, content: Option<&'static [u8]> }
pub struct ByteArrayResource { description: String, bytes: Vec<u8> }
```

#### 位置前缀约定

| 前缀 | Spring 等价 | vernal 实现 |
|---|---|---|
| `classpath:` | `ClassPathResource` | `ClassPathResource`（编译时 `include_bytes!`） |
| `file:` | `FileSystemResource` | `FileSystemResource` |
| `url:` | `UrlResource` | 待实现（feature `convert-url`） |
| 无前缀 | `FileSystemResource` | `FileSystemResource` |

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| dyn-compatible | `Resource` trait object-safe | 满足（测试 `Box<dyn Resource>`） |
| Send + Sync | 所有 Resource 实现者 | 满足 |
| 异步读取 | `async fn read_bytes()` | 待实现 |
| 资源模式匹配 | `classpath*:`, `**/*.xml` | 待实现 |

#### 待补齐

- [ ] `AsyncResource` trait（`async fn read_bytes()`）
- [ ] `UrlResource`（feature `convert-url`）
- [ ] `ResourcePatternResolver`（`getResources("classpath*:*.xml")`）

---

### 2.6 环境配置

**对标**: `org.springframework.core.env.Environment` / `PropertySource`

#### Spring API

```java
public interface PropertySource<T> {
    String getName();
    Object getProperty(String name);
}
public interface Environment extends PropertyResolver {
    String[] getActiveProfiles();
    boolean acceptsProfiles(String... profiles);
}
```

#### Rust 实现

```rust
// crates/vernal-core/src/environment/mod.rs
pub trait PropertySource: Send + Sync {
    fn name(&self) -> &str;
    fn get_property(&self, key: &str) -> Option<String>;
    fn contains_property(&self, key: &str) -> bool { /* 默认实现 */ }
}

pub struct MapPropertySource {
    name: String,
    properties: HashMap<String, String>,
}

pub struct SystemEnvironmentPropertySource { name: String }

pub struct PropertySources {
    sources: Vec<Box<dyn PropertySource>>,
}
impl PropertySources {
    pub fn new() -> Self;
    pub fn add_first(&mut self, source: Box<dyn PropertySource>);
    pub fn add_last(&mut self, source: Box<dyn PropertySource>);
    pub fn get_property(&self, key: &str) -> Option<String>;
    pub fn contains_property(&self, key: &str) -> bool;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

pub trait Environment: Send + Sync {
    fn get_property(&self, key: &str) -> Option<String>;
    fn get_property_with_default(&self, key: &str, default: &str) -> String;
    fn contains_property(&self, key: &str) -> bool;
    fn get_active_profiles(&self) -> Vec<String>;
    fn get_default_profiles(&self) -> Vec<String>;
    fn accepts_profiles(&self, profiles: &[&str]) -> bool;
}

pub struct StandardEnvironment {
    property_sources: PropertySources,
    active_profiles: Vec<String>,
    default_profiles: Vec<String>,
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| dyn-compatible | `PropertySource` / `Environment` 均 object-safe | 满足 |
| Send + Sync | 所有 trait 和实现 | 满足 |
| addFirst/addLast | 优先级链：先添加的优先级更高 | 满足（测试覆盖） |
| Profile 支持 | `get_active_profiles` / `accepts_profiles` | 满足 |
| 属性占位符解析 | `${key:default}` 语法 | 待集成 |

#### 待补齐

- [ ] `PropertiesPropertySource`（从 `java-properties` 文件加载）
- [ ] `YamlPropertySourceLoader`（从 YAML 文件加载）

---

### 2.7 任务执行

**对标**: `org.springframework.core.task.TaskExecutor` / `AsyncTaskExecutor`

#### Spring API

```java
public interface TaskExecutor {
    void execute(Runnable task);
}
public interface AsyncTaskExecutor extends TaskExecutor {
    Future<?> submit(Runnable task);
}
```

#### Rust 实现

```rust
// crates/vernal-core/src/task/mod.rs
pub trait TaskExecutor: Send + Sync {
    fn execute(&self, task: impl FnOnce() + Send + 'static);
    fn name(&self) -> &str;
}

pub trait AsyncTaskExecutor: TaskExecutor {
    async fn execute_async(&self, task: impl FnOnce() + Send + 'static)
        -> Result<(), TaskError>;
}

#[derive(Debug, Clone)]
pub enum TaskError {
    Rejected(String),
    Timeout(String),
    ExecutionFailed(String),
}

pub struct SimpleTaskExecutor { name: String }
impl TaskExecutor for SimpleTaskExecutor {
    fn execute(&self, task: impl FnOnce() + Send + 'static) { task(); }
    fn name(&self) -> &str { &self.name }
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| dyn-compatible | `TaskExecutor` object-safe | 满足 |
| Send + Sync | 所有 trait 和实现 | 满足 |
| tokio 集成 | `TokioRuntime::spawn` 封装 `tokio::spawn` | 骨架完成（feature `async-runtime`） |
| 线程池支持 | `ThreadPoolTaskExecutor` | 待实现 |

#### 待补齐

- [ ] `TokioTaskExecutor`（实现 `AsyncTaskExecutor`，内部调用 `tokio::spawn`）
- [ ] `ThreadPoolTaskExecutor`（带队列容量和拒绝策略）

---

### 2.8 日志

**对标**: `org.apache.commons.logging`（日志门面）

#### Spring API

```java
public interface Log {
    boolean isDebugEnabled();
    boolean isInfoEnabled();
    boolean isWarnEnabled();
    boolean isErrorEnabled();
    void debug(Object message);
    void info(Object message);
    void warn(Object message);
    void error(Object message);
}

public abstract class LogFactory {
    public static Log getLog(Class<?> clazz);
}
```

#### Rust 实现

```rust
// crates/vernal-core/src/logging/mod.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel { Trace, Debug, Info, Warn, Error }

pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
    fn trace(&self, message: &str) { self.log(LogLevel::Trace, message); }
    fn debug(&self, message: &str) { self.log(LogLevel::Debug, message); }
    fn info(&self, message: &str)  { self.log(LogLevel::Info, message); }
    fn warn(&self, message: &str)  { self.log(LogLevel::Warn, message); }
    fn error(&self, message: &str) { self.log(LogLevel::Error, message); }
    fn is_enabled(&self, _level: LogLevel) -> bool { true }
}

// 内置实现
pub struct ConsoleLogger { name: String, min_level: LogLevel }
pub struct NoOpLogger;

// feature = "tracing"
pub mod tracing_logger {
    pub struct TracingLogger { name: String }
    impl Logger for TracingLogger {
        fn log(&self, level: LogLevel, message: &str) {
            // 调用 tracing::info!() / tracing::warn!() 等
        }
    }
}
```

#### 约束表

| 约束项 | 要求 | 当前状态 |
|---|---|---|
| dyn-compatible | `Logger` trait object-safe | 满足 |
| Send + Sync | 所有 trait 和实现 | 满足 |
| tracing 集成 | `TracingLogger` 桥接 tracing 宏 | 满足（feature `tracing`） |
| 级别过滤 | `ConsoleLogger::set_level` / `is_enabled` | 满足 |
| 结构化日志 | key-value 字段支持 | 待实现 |

#### 待补齐

- [ ] `TracingLogger` 的结构化字段支持（`tracing::info!(key = value, ...)`）
- [ ] `LogFactory` 全局注册表（按模块名获取 Logger）

---

## 三、关键技术要求

### 3.1 选型基线

| 用途 | Rust crate | 版本 | Spring 等价 |
|---|---|---|---|
| 序列化框架 | `serde` | 1.x | Jackson API |
| JSON 后端 | `serde_json` | 1.0 | jackson-databind |
| XML 后端 | `quick-xml` | 0.41 | woodstox-core |
| YAML 后端 | `serde_yaml_ng` | 待引入 | snakeyaml |
| Properties | `java-properties` | 待引入 | java.util.Properties |
| 错误派生 | `thiserror` | 2.0 | Lombok @Getter |
| 日志门面 | `tracing` | workspace | commons-logging |
| 正则表达式 | `regex` | 1.11 | java.util.regex |
| 异步运行时 | `tokio` | workspace | reactor-core |
| 命令行解析 | `clap` | 4.5 | jopt-simple |
| Web 服务器 | `hyper` | 1.9 | jetty-io |

### 3.2 安全约束

```text
零 unsafe 策略:
  - lib.rs 不包含任何 unsafe 块
  - 所有依赖 crate 的 unsafe 使用通过 feature flag 隔离
  - Rust 2024 edition 中 set_var 需要 unsafe 块（已通过 #[allow(unused)] 处理）

dyn-compatible 要求:
  - 所有公开 trait 必须是 object-safe
  - 泛型方法（如 Converter<S,T>）不可 dyn，通过 Convertible trait 静态分发补偿
  - GenericConverter 通过 TypeId 运行时匹配实现多对多转换

Send + Sync + 'static:
  - 所有公开类型必须满足这三个 bound
  - 编译期断言: fn assert_send_sync<T: Send + Sync + 'static>() {}
  - 运行时共享通过 Arc<Mutex<T>> 或 Arc<AtomicXxx> 实现
```

### 3.3 Feature Flag 策略

```text
Tier 1（基础设施，强烈推荐）:
  error-derive   → thiserror 派生宏
  macros         → pastey 宏拼接
  once-cell      → once_cell 单例
  registry       → inventory 分布式注册

Tier 2（用户可见类型，强烈推荐）:
  id-uuid / id-ulid / id-nanoid  → ID 生成器后端
  convert-uuid / convert-url / convert-time / convert-chrono  → 类型转换器
  serde          → 派生宏支持
  derive-extras  → derive_more 补充
  convert-regex  → 正则转换器
  mime / mime-ext / mime-sniff  → MIME 类型支持
  digest         → SHA-256 摘要

Tier 3（可选，特定场景）:
  case-conv      → enum 大小写无关转换
  time-web       → WASM 兼容时间

Spring Core 对应:
  tracing        → 日志门面
  json           → JSON 序列化
  xml            → XML 解析
  async-runtime  → 异步运行时
  cli            → 命令行解析
  web            → Web 服务器
  test-mock      → Mock 框架
  json-schema    → JSON Schema 验证
  test-async     → 异步测试
```

### 3.4 错误域注册表

当前已注册 13 个错误域（`ErrorDomain::all()`），命名约束：snake_case、长度 <= 16 字节、唯一性（编译期 + 测试验证）。
错误码约定：负数 = 框架错误，正数 = 业务错误，零保留。

| 域 | 说明 | 域 | 说明 |
|---|---|---|---|
| `ioc` | IoC 容器 | `tower` | Tower 集成 |
| `aop` | AOP 拦截器 | `discovery` | 组件发现 |
| `context` | 应用上下文 | `macros` | 过程宏 |
| `web` | Web 协议 | `core` | 框架核心 |
| `http` | HTTP 传输 | `bridge` | 桥接层 |
| `migration` | 数据库迁移 | `security` | 安全 |
| `metrics` | 监控指标 | | |

---

## 四、架构

### 4.1 模块依赖图

```text
vernal-core (lib.rs)
  +-- ordered             (排序常量 + Ordered/PriorityOrdered trait)
  +-- error               (VernalError + ErrorCode + ErrorDomain + ErrorReport)
  +-- convert             (ConversionService + Converter + GenericConverter)
  +-- resource            (Resource + ResourceLoader + FileSystem/ClassPath/ByteArray)
  +-- environment         (Environment + PropertySource + PropertySources)
  +-- task                (TaskExecutor + AsyncTaskExecutor + SimpleTaskExecutor)
  +-- logging             (Logger + ConsoleLogger + TracingLogger)
  +-- serialization       (JsonCodec + XmlCodec)
  +-- codec               (Encoder + Decoder)
  +-- async_runtime       (RuntimeType + TokioRuntime)
  +-- app_lifecycle_phase (8 态生命周期)
  +-- diagnostics         (Span + SpanId + AttributeValue)
  +-- id                  (IdGenerator + NanoId/Snowflake/Ulid/Uuid/ObjectId)
  +-- time                (StopWatch + StopWatchUnit)
  +-- util                (AntPathMatcher + MimeType + DataSize + ...)
  +-- failure / conventions / constants / spring_properties / spring_version
```

### 4.2 类型层次

```text
BoxError ──From──> VernalError { Business, WithContext, WithContextEntries, Infrastructure }
                      │ report()
                      v
                   ErrorReport (脱敏快照)

ErrorCode trait ──into_vernal_error()──> VernalError::Business
```

---

## 五、验收标准

### 5.1 编译验收

```bash
# 1. 默认编译（零依赖）
cargo build -p vernal-core

# 2. 全 feature 编译
cargo build -p vernal-core --all-features

# 3. 单独 feature 编译
cargo build -p vernal-core --features json
cargo build -p vernal-core --features xml
cargo build -p vernal-core --features tracing
cargo build -p vernal-core --features async-runtime
```

### 5.2 测试验收

```bash
# 全量测试
cargo test -p vernal-core --all-features

# 单模块测试（按需）
cargo test -p vernal-core --lib ordered
cargo test -p vernal-core --lib error
cargo test -p vernal-core --lib convert
cargo test -p vernal-core --lib resource
cargo test -p vernal-core --lib environment
cargo test -p vernal-core --lib task
cargo test -p vernal-core --lib logging
```

### 5.3 质量门禁

| 检查项 | 命令 | 通过标准 |
|---|---|---|
| 零 unsafe | `grep -r "unsafe" crates/vernal-core/src/` | 无 unsafe 块 |
| Clippy | `cargo clippy -p vernal-core --all-features -- -D warnings` | 零警告 |
| 格式化 | `cargo fmt -p vernal-core --check` | 无 diff |
| 文档 | `cargo doc -p vernal-core --no-deps` | 零警告 |
| MSRV | `cargo build -p vernal-core`（rustc 1.88） | 编译通过 |

### 5.4 dyn-compatible / Send + Sync 验收

```rust
// 编译期验证：所有公开 trait 是 object-safe
fn _assert_object_safe() {
    let _: Box<dyn Resource>;
    let _: Box<dyn PropertySource>;
    let _: Box<dyn Environment>;
    let _: Box<dyn TaskExecutor>;
    let _: Box<dyn Logger>;
    let _: Box<dyn ConverterRegistry>;
    let _: Box<dyn ConditionalConverter>;
}

// 编译期验证：所有公开类型满足 Send + Sync + 'static
fn _assert_send_sync<T: Send + Sync + 'static>() {}
fn _check_bounds() {
    _assert_send_sync::<VernalError>();
    _assert_send_sync::<ErrorContext>();
    _assert_send_sync::<MapPropertySource>();
    _assert_send_sync::<PropertySources>();
    _assert_send_sync::<StandardEnvironment>();
    _assert_send_sync::<SimpleTaskExecutor>();
    _assert_send_sync::<ConsoleLogger>();
    _assert_send_sync::<TypeIdConverterRegistry>();
    _assert_send_sync::<ClosureGenericConverter>();
    _assert_send_sync::<FileSystemResource>();
    _assert_send_sync::<ByteArrayResource>();
}
```

---

## 六、相关文档

| 文档 | 路径 | 说明 |
|---|---|---|
| 对象名称一致性检查 | `docs/vernal-core/vernal-core对象名称一致性检查.md` | Spring→vernal 命名映射 |
| 对象级对照表 | `docs/vernal-core/vernal-core对象级对照表.md` | 逐对象映射关系 |
| 语义迁移对照表 | `docs/vernal-core/vernal-core语义迁移对照表.md` | 语义级迁移指南 |
| 迁移路线图 | `docs/vernal-core/vernal-core迁移路线图.md` | 实施计划 |
| 序列化框架迁移 | `docs/vernal-core/SerializationFramework-Migration.md` | 序列化选型 |
| MIME 集成 | `docs/vernal-core/MimeIntegration.md` | MIME 类型支持 |
| 跳过模块说明 | `docs/vernal-core/SkippedModules.md` | 不迁移的模块 |
| Cargo.toml | `crates/vernal-core/Cargo.toml` | 依赖和 feature 定义 |

---

> **文档版本**: v3
> **维护者**: Vernal Framework Team
> **下次审查**: 每个 milestone 结束时更新
