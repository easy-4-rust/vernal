# spring-core + tx_di → vernal-core 对象级对照表 v2.0

> 版本：v2.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core（737 个 .java / 51 个顶级包 / 12.8 万行）
> + tx_di **dev**（414 个 .rs / 73,479 行 / tx-di-core + common/tx_error + common/tx_common）
> 现状基线：`vernal-framework/crates/vernal-core`（22 个 .rs / 1,417 行 / 7 模块 / 17 公开声明）
> 迁移原则：**功能语义对齐、命名 100% 保留、类型实现 Rust 化**

本文档对每一个 Spring / tx_di 源对象给出 vernal-core 中**已经存在 / 应当存在 / 故意省略** 的对应物，并标注状态。
状态图例：✅ 已实现 / 🔶 语义等价但形态不同 / ⬜ 待迁移 / 🚫 不迁移（语义由其他 crate 提供）/ 🆕 vernal-core 新增

> **路径纠正说明**：原任务描述里 `tx_di` 路径为 `workspace-github/tx_di`，实测在 `workspace-github-easy-4-rust/tx_di`。本表与配套 3 份文档均以实测路径为准。

---

## 一、Spring `org.springframework.core` 顶级类型

| Java 类 / 接口 | Spring 语义 | vernal-core 对象 | 状态 | 落地路径 / 备注 |
|---|---|---|---|---|
| `org.springframework.core.NestedRuntimeException` | 嵌套运行时异常基类 | `VernalError`（enum 4 变体） | 🆕 | `error/vernal_error.rs` |
| `org.springframework.core.NestedCheckedException` | 嵌套 checked 异常基类 | （不引入，复用 `std::error::Error` + `From<E>`） | 🚫 | Rust 无 checked/unchecked 之分 |
| `org.springframework.core.NestedExceptionUtils` | 嵌套消息拼接工具 | `ErrorReport::display()` + `Display` | 🆕 | `error/error_report.rs` |
| `org.springframework.core.ErrorCoded` | 错误码接口（`getErrorCode(): String`） | `error::ErrorCode` trait | 🆕 | `error/error_code.rs`（`#[derive(ErrorCode)]` 在 vernal-macros） |
| `org.springframework.core.NestedIOException` | IO 异常嵌套基类 | `From<std::io::Error> for VernalError` | 🆕 | `error/vernal_error.rs` |
| `org.springframework.core.AttributeAccessor` | 通用属性读写 | （不实现，由各子系统按需提供） | 🚫 | Spring 已被 `@Component` 注解等替代，vernal 用 BeanDefinition / TraitBinding |
| `org.springframework.core.AttributeAccessorSupport` | AttributeAccessor 默认实现 | （同上） | 🚫 | — |
| `org.springframework.core.Ordered` | 排序接口（`HIGHEST_PRECEDENCE` / `LOWEST_PRECEDENCE`） | `ordered::INIT_SORT_*`（i32 常量）+ Spring 别名 | 🔶 | `ordered.rs`：Spring 用 trait getter 取动态值，vernal 把 `init_sort` 编译期为常量，效率更高 |
| `org.springframework.core.PriorityOrdered` | 优先排序标记接口 | `INIT_SORT_INFRASTRUCTURE / BEAN_FACTORY / EVENT_LISTENER / MESSAGE_SOURCE` | 🔶 | Spring 是 trait 继承（`extends Ordered`），vernal 是排序值分层（值越小越先） |
| `org.springframework.core.annotation.Order` | `@Order` 注解 | `#[component(init_sort = N)]` 属性宏 | 🆕 | vernal-macros 提供（不在 core 范围内） |
| `org.springframework.core.annotation.OrderComparator` | Ordered 比较器 | 由 `vernal-core::ordered` 中的常量 + 调用方排序代码共同承担 | 🔶 | — |
| `org.springframework.core.SpringVersion` | 框架版本字符串 | `vernal_core::FRAMEWORK_VERSION`、`MINIMUM_RUST_VERSION`、`PROJECT_STATUS` | 🆕 | `lib.rs` |
| `org.springframework.util.Assert` | 断言工具 | （部分用 `vernal_beans::graph_error` 提供） | 🚫 | Spring 的运行时断言通过 `IllegalArgumentException` 表达，Rust 用类型系统替代 |
| `org.springframework.util.StopWatch` | 计时器（多任务分段） | `time::StopWatch` | ✅ | `time/stop_watch.rs`（已实现，多任务分段 + pretty_print） |
| `org.springframework.util.ObjectUtils` | 通用 Object 工具 | （不引入通用 Object 帮助，直接用 Rust 类型系统） | 🚫 | — |
| `org.springframework.util.ClassUtils` | Class 帮助 | （不引入） | 🚫 | Rust 用 `std::any::TypeId` |
| `org.springframework.util.SystemPropertyUtils` | 系统属性占位符 | （不引入，留给 `vernal-environment`） | 🚫 | — |
| `org.springframework.util.ResourceUtils` | 资源加载 | （同上） | 🚫 | — |
| `org.springframework.util.StringUtils` | 字符串工具 | （不引入；用 `std::string` / `convert_case` feature） | 🚫 | — |
| `org.springframework.util.CollectionUtils` | 集合工具 | （不引入；用 std collections） | 🚫 | — |
| `org.springframework.util.ReflectionUtils` | 反射工具 | （不引入；用 `TypeId` + 过程宏） | 🚫 | — |
| `org.springframework.util.MultiValueMap` | 多值 Map | （不引入；属于 `vernal-beans`） | 🚫 | — |
| `org.springframework.util.DefaultConversionService` | 默认 ConversionService 实现 | `VernalCore::ConversionService` 单 struct + 10 个内置 Convertible | 🆕 | `convert/mod.rs` |

> **Spring 子包统计**：spring-core 共 51 个顶级包，本表覆盖最关键的 18 个；剩余 33 个包（如 `codec / io.buffer / log / metrics / retry / serializer / task` 等）属于 **vernal-context / vernal-web / vernal-tx** 等上层 crate，不下沉到 vernal-core。

---

## 二、Spring 类型转换（`org.springframework.core.convert`）

| Java 类 / 接口 | Spring 语义 | vernal-core 对象 | 状态 | 落地路径 |
|---|---|---|---|---|
| `ConversionService` | 类型转换服务入口（`canConvert` + `convert`） | `convert::ConversionService`（struct + 静态方法 + `can_convert`） | ✅ | `convert/mod.rs` |
| `GenericConversionService` | 可扩展的 ConversionService 默认实现 | （不引入，vernal 用过程宏或手动 impl `Convertible`） | 🚫 | Spring 用 `Map<Pair, Converter>` 运行时查表，Rust 通过 trait 静态分发等价 |
| `DefaultConversionService` | 注册一组内置 Converter | 内置 10 个 `Convertible` 实现（bool / 数字 / String / Enum / Option / PathBuf / Duration / URL / SocketAddr / 自定义） | 🆕 | `convert/*.rs` |
| `Converter<S, T>` | 单向转换器接口 | `convert::Converter<S, T>`（trait，方法签名等价） | ✅ | `convert/converter.rs` |
| `ConverterFactory<S, R>` | 类型族转换器工厂 | `impl<T: FromStr> Convertible for T` blanket impl（覆盖整个 enum 族） | 🔶 | `convert/enum_converter.rs` |
| `GenericConverter` | 通用多对多转换器 | （不引入，trait 静态分发已覆盖） | 🚫 | — |
| `ConversionFailedException` | 转换失败异常 | `convert::ConversionError`（`std::error::Error` 实现） | 🆕 | `convert/mod.rs` |
| `ConditionalConverter` | 条件转换器 | （trait 静态分发时不需要，调用方 `where T: FromStr` 即条件） | 🚫 | — |
| `TypeDescriptor` | 类型描述 | `std::any::TypeId` + 反射由 type-level 替代 | 🚫 | — |

### 2.1 Spring 内置 `Converter`（完整覆盖）

| Java Converter | 语义 | vernal-core 实现 | 状态 |
|---|---|---|---|
| `StringToBooleanConverter` | String→bool（"true"/"1"/"yes"/"on"...） | `impl Convertible for bool` | ✅ |
| `StringToNumberConverterFactory` | String→Number（家族） | `impl_convertible_for_number!` 宏为 13 个数字类型统一实现 | ✅ |
| `StringToEnumConverterFactory` | String→Enum | `convert_enum::<T>()` 函数 + 任意 `T: FromStr` 自动满足 `Convertible` | ✅ |
| `StringToCharacterConverter` | String→char | （不实现，按 Spring 文档已 deprecated） | 🚫 |
| `StringToCharsetConverter` | String→Charset | （不实现，留给通用 std 替代） | 🚫 |
| `StringToUUIDConverter` | String→UUID | `feature = "convert-uuid"` → `impl Convertible for uuid::Uuid` | 🆕（待 S5） |
| `StringToDateConverter` | String→LocalDate / LocalDateTime | `feature = "convert-time"` 或 `"convert-chrono"` → `impl Convertible for OffsetDateTime / NaiveDateTime` | 🆕（待 S5） |
| `StringToInstantConverter` | String→Instant | `feature = "convert-time"` → `impl Convertible for time::OffsetDateTime` | 🆕（待 S5） |
| `StringToDurationConverter` | String→Duration | `impl Convertible for std::time::Duration`（std only，解析 `"30s"/"5m"/"1h"`） | 🆕（待 S5） |
| `StringToURLConverter` | String→URL | `feature = "convert-url"` → `impl Convertible for url::Url` | 🆕（待 S5） |
| `StringToPathConverter` | String→Path | `impl Convertible for std::path::PathBuf`（std only） | 🆕（待 S5） |
| `NumberToNumberConverterFactory` | Number→Number | （不引入，原生 `as` cast 即可） | 🚫 |
| `DateToStringConverter` | 日期→String | （同上，feature-gated chrono/time 互转） | 🆕 |
| `CollectionToCollectionConverter` | 集合→集合 | （不引入，性能敏感的集合映射由 serde 或用户代码承担） | 🚫 |
| `ArrayToCollectionConverter` | 数组→集合 | （不引入） | 🚫 |
| `ObjectToStringConverter` | Object→String | `Display` trait blanket impl | 🔶 |
| `IdToEntityConverter` | ID→实体 | （不引入，业务代码自行实现） | 🚫 |
| `FallibleConverter` | 可失败的 Converter | `Result<T, ConversionError>` 即等价的可失败转换 | 🆕 |
| `StreamConverter` | String→Stream | （不引入，属于 vernal-web） | 🚫 |
| `ByteArrayConverter` | byte[] ↔ String | （不引入，属于 vernal-web；feature = "convert-bytes" 可选） | 🆕（待 S5） |

---

## 三、Spring 资源与 IO（`org.springframework.core.io`）— **不迁移到 vernal-core**

| Java 类 | Spring 语义 | vernal-core 对象 | 状态 | 说明 |
|---|---|---|---|---|
| `Resource` / `InputStreamSource` | 资源抽象 | （不引入） | 🚫 | 资源抽象留给 `vernal-context::resources` 或独立 `vernal-resource` crate |
| `ClassPathResource` / `FileSystemResource` 等 | 各类 Resource | （同上） | 🚫 | — |
| `ResourceLoader` / `ResourcePatternResolver` | 资源加载器 | （同上） | 🚫 | — |

---

## 四、Spring 元注解 / 注解工具（`org.springframework.core.annotation`）— **不在 vernal-core 范围**

`AnnotationAttributes` / `AnnotationUtils` / `AnnotationAwareOrderComparator` 等注解工具**完全跳过**：Rust 用过程宏 + syn 直接生成代码，不存在 Java 那种运行时反射元注解层。vernal 对等位置是 `vernal-macros`，不是 vernal-core。

---

## 五、Spring Observation API（6.1 新增，**轻量版迁移到 S9**）

| Java 类 | Spring 语义 | vernal-core 对象 | 状态 | 备注 |
|---|---|---|---|---|
| `Observation` | 观测上下文（OpenTelemetry 兼容） | `diagnostics::Span`（轻量版，无 OTel 集成） | 🆕（待 S9） | `diagnostics/span.rs` |
| `ObservationRegistry` | 全局观测注册中心 | （不引入，由 `vernal-context::startup_observation` 负责） | 🚫 | — |
| `ObservedAspect` | AOP 集成观测 | （不引入，由 `vernal-aspects` 提供） | 🚫 | — |
| `GlobalObservationRegistry` | 全局观测单例 | （不引入） | 🚫 | — |

---

## 六、Spring Task / Scheduling 抽象（`org.springframework.core.task`）— **不迁移到 vernal-core**

| Java 类 | 语义 | vernal-core 对象 | 状态 |
|---|---|---|---|
| `TaskExecutor` | 任务执行器 | （不引入，归 `vernal-context::async_task`） | 🚫 |
| `AsyncTaskExecutor` | 异步任务执行器 | （同上） | 🚫 |
| `TaskDecorator` | 任务装饰器 | （同上） | 🚫 |

---

## 七、Spring Retry 抽象（`org.springframework.core.retry`）— **不迁移到 vernal-core**

| Java 类 | 语义 | vernal-core 对象 | 状态 |
|---|---|---|---|
| `RetryTemplate` | 重试模板 | （不引入，归 `vernal-tx` 或独立 `vernal-retry` crate） | 🚫 |

---

## 八、Spring Serializer（`org.springframework.core.serializer`）— **不迁移到 vernal-core**

| Java 类 | 语义 | vernal-core 对象 | 状态 |
|---|---|---|---|
| `Serializer` / `Deserializer` | 序列化抽象 | （不引入，由 serde feature 处理） | 🚫 |

---

## 九、tx_di `tx-di-core/src/*` 对象

| tx_di 类型 | tx_di 语义 | vernal-core 对象 | 状态 | 备注 |
|---|---|---|---|---|
| `tx_di_core::App` | 运行时应用（持有 store / metas / shutdown_token） | （不属于 vernal-core，由 `vernal-context::ApplicationContext` 等价） | 🚫 | — |
| `tx_di_core::BuildContext` | 构建上下文（load config + topo sort + auto register） | （同上） | 🚫 | — |
| `tx_di_core::InnerContext = DashMap<TypeId, CompRef>` | 全局类型擦除组件表 | （同上，留给 `vernal-beans`） | 🚫 | — |
| `tx_di_core::Component` trait | 组件 trait（关联类型 `Deps` + 5 个生命周期钩子） | （同上的位置；vernal-core 提供 `Lifecycle` 抽象的特征 + `AppLifecyclePhase` 枚举） | 🔶 | vernal-beans 的 `Component` 涵盖 `Deps`/`build`，5 个生命周期钩子映射到 vernal-context 的 `Lifecycle` |
| `tx_di_core::Component::inner_init / init / async_init / async_run / shutdown` | 5 个生命周期钩子 | `Lifecycle::initialize / start / stop`（3 个） + IoC 容器内建 `init / async_init / shutdown`（3 个） | 🔶 | tx_di 5 阶段 vs vernal 3 阶段：vernal 把 tx_di 的 inner_init（build 之后立刻同步）合并到 IoC 容器自身；async_run（后台长任务）属于 ApplicationContext 的 spawn 责任而非 Component 自身 |
| `tx_di_core::Component::init_sort() -> i32` | 同一拓扑深度的组件按 init_sort 排序 | `vernal_core::ordered::INIT_SORT_DEFAULT`（默认 10000）+ `INIT_SORT_INFRASTRUCTURE`（`i32::MIN + 1`）/`INIT_SORT_BUSINESS`（0）/`INIT_SORT_APPLICATION`（`i32::MAX - 1`） | ✅ | Spring `Ordered` 和 tx_di `init_sort` 都是 i32；vernal 用常量代替函数指针，更适合 Rust 编译期评估 |
| `tx_di_core::Component::trait_impls()` | 列出该组件实现的 trait（`TypeId` 数组） | （属于 vernal-beans 范围：`vernal_beans::TraitBinding`） | 🚫 | — |
| `tx_di_core::scope::Scope` enum | Singleton / Prototype | `vernal_core::ordered` 不提供 Scope；Scope 由 `vernal-beans::component_scope` 定义 | 🚫 | Scope 与 Ordered 是不同维度，Spring 也分 `BeanDefinition#scope` 和 `Ordered` 两套 |
| `tx_di_core::ComponentMeta` | 组件元数据（factory / init_fn / async_init_fn / shutdown_fn 函数指针 + scope / dep_type_ids） | （不属于 vernal-core，由 vernal-beans 的 `ComponentProvider` 通过 trait 静态分发承担） | 🚫 | vernal 不需要 `ComponentMeta` 这种扁平函数指针表，因为 Rust trait 静态分发已经做到零开销 |
| `tx_di_core::registry::COMPONENT_REGISTRY` | linkme 分布式切片 | （同上） | 🚫 | — |
| `tx_di_core::lifecycle::App::build() / run() / waiting_exit() / shutdown()` | App 生命周期方法 | `vernal_context::ApplicationContext::refresh() / close()` | 🚫 | — |
| `tx_di_core::Store` | 类型擦除组件存储（DashMap） | （同上） | 🚫 | — |
| `tx_di_core::topology::topo_sort()` | 拓扑排序算法 | （属于 vernal-beans） | 🚫 | — |
| `tx_di_core::config::AppAllConfig` | 整体配置（TOML 加载 + 点分路径访问） | （不引入；归 `vernal-context::application_environment`） | 🚫 | — |

### 9.1 tx_di 错误体系（`common/tx_error/src/*`）

| tx_di 类型 | tx_di 语义 | vernal-core 对象 | 状态 | 落地路径 |
|---|---|---|---|---|
| `tx_error::AppError` enum | `ErrCode / WithContext / Internal(anyhow::Error)` 三变体 | `vernal_core::error::VernalError`：`Business / WithContext / WithContextEntries / Infrastructure(SharedError)` 四变体 | 🆕 | `error/vernal_error.rs` |
| `tx_error::AppResult<T>` | `Result<T, AppError>` | `Result<T, BoxError>` 或 `Result<T, VernalError>` | 🆕 | — |
| `tx_error::AppErrCode` struct（domain + code + message） | 归一化错误码值类型 | `vernal_core::error::ErrorCode` trait：`domain() / code() / message()` | 🆕 | `error/error_code.rs` |
| `tx_error::CodeMsg` trait | 把枚举转换为 AppErrCode | `ErrorCode::into_vernal_error()` + 派生宏 `ErrorCode`（vernal-macros） | 🆕 | trait 自身在 vernal-core，派生宏在 vernal-macros |
| `tx_error::AppErrCode::PartialEq` | 只比较 domain + code | `VernalError::PartialEq` 同语义 | 🆕 | — |
| `tx_error::AppError::domain() / code() / message() / context() / internal()` 访问器 | 同名访问器 | 同名访问器（`domain() / code() / message() / context() / is_infrastructure()`） | 🆕 | — |
| `tx_error::AppError::is_same_kind()` | 同类错误比较 | `ErrorCode::is_same_kind()` | 🆕 | — |
| `tx_error::AppError::is_internal()` | 是否为内部错误 | `VernalError::is_infrastructure()` | 🆕 | — |
| `tx_error::AppError::err_code()` | 提取 AppErrCode | `ErrorCode::into_vernal_error()` | 🆕 | — |
| `tx_error::AppError::full_message()` | 完整消息字符串 | `ErrorReport::display()` + `Display` | 🆕 | — |
| `tx_error::From<anyhow::Error>` | anyhow 自动转换 | `From<BoxError>` + `From<SharedError>`（vernal 用 `Arc<dyn Error>` 替代 anyhow，避免额外依赖） | 🔶 | — |
| `tx_error::From<std::io::Error> / From<serde_json::Error> / From<toml::de::Error>` | 常见标准库错误 From | `From<std::io::Error>` only（vernal-core 只保留 IO，其他交给业务层） | 🔶 | — |
| `tx_error::From<String> / From<&str>` | 字符串错误捕获 | `From<String> for VernalError` + `From<&str> for VernalError`（**新增 S2**） | 🆕（待 S2） | — |
| `tx_error::log_err(e, err)` | 日志 + 返回统一错误 | （不属于 vernal-core，留给 `vernal-log`） | 🚫 | — |
| `tx_error::axum_support` | axum 集成 | （不引入；HTTP 错误映射归 `vernal-web`） | 🚫 | — |

### 9.2 tx_di `common/crates/tx_common/src/*`

| tx_di 类型 | tx_di 语义 | vernal-core 对象 | 状态 |
|---|---|---|---|
| `tx_common::ApiR<T> / ApiRes<T> / RCode / FormattedDateTime` | HTTP 统一响应结构 | （不属于 vernal-core，留给 `vernal-web`） | 🚫 |
| `tx_common::id::*` | ID 生成（UUID / Snowflake） | `vernal_core::id::ObjectId` + 4 个 feature-gated 后端（uuid / ulid / nanoid / snowflake） | 🆕（**强化**：tx_common 的 ID 模块被 vernal-core 接管，提供 5 种 ID 后端） |
| `tx_common::page::*` | 分页模型 | （不属于 vernal-core，留给 `vernal-web`） | 🚫 |
| `tx_common::date::*` | 日期工具 | （不属于 vernal-core，vernal-core 只保留 `StopWatch`） | 🚫 |
| `tx_common::api_r::*` | API 响应工厂 | （同上） | 🚫 |

---

## 十、vernal-core 已有但需要扩展的对象

下列对象 vernal-core 已经具备，但需要按 Spring / tx_di 语义补强：

| vernal-core 对象 | 现有实现 | 待补强项 | 阶段 |
|---|---|---|---|
| `VernalError`（enum 4 变体） | `Business / WithContext / WithContextEntries / Infrastructure` | 新增 `From<tokio::task::JoinError>`（feature-gated）和 `From<String> / From<&str>` 的标准库 From 实现 | S2 |
| `BoxError = Box<dyn Error + Send + Sync + 'static>` | 已实现 | 配合 `thiserror` 派生：让 vernal 各子系统的错误类型 `#[derive(thiserror::Error)]` 后能 `?` 自动转 `VernalError::Infrastructure` | S8 |
| `SharedError = Arc<dyn Error + Send + Sync + 'static>` | 已实现 | 同上 | S8 |
| `LifecyclePhase`（8 变体：Created / Initializing / Initialized / Starting / Started / Stopping / Stopped / Failed） | 已实现 | **重命名为 `AppLifecyclePhase`**（与 `vernal-context::LifecyclePhase` 重名） | S1 |
| `INIT_SORT_INFRASTRUCTURE / BUSINESS / APPLICATION / DEFAULT` 常量 | 已实现 | 增加 `INIT_SORT_BEAN_FACTORY`、`INIT_SORT_EVENT_LISTENER`、`INIT_SORT_MESSAGE_SOURCE`、`INIT_SORT_TASK` 4 个 Spring 标准锚点；增加 `HIGHEST_PRECEDENCE / LOWEST_PRECEDENCE` Spring 风格别名 | S4 |
| `ObjectId` | 已实现（基于系统时间 + PID + 计数器） | 增加 `From<String>` 和 serde 支持；4 个新 ID 后端 trait 抽象 | S7 |
| `ConversionService::convert<T>` | 已实现（5 个内建） | 增加 `DateTime / Url / Duration / PathBuf / SocketAddr / Uuid` 等配置绑定常用类型；新增 `can_convert::<T>()` 判定方法 | S5 |
| `convert_enum` | 已实现 | 暴露 `FromStr + Display` 自动支持（已实现） | ✅ |
| `StopWatch::pretty_print()` | 已实现 | 增加 `short_summary()` / `pretty_print_with_unit(unit)` / `task_info_array()` / `start_with_ticks()`；新增 `StopWatchUnit` 枚举 | S6 |
| `FRAMEWORK_VERSION` / `MINIMUM_RUST_VERSION` / `PROJECT_STATUS` | 已实现 | 增加 `VERNAL_VERSION` 别名 + `wasm32_supported` 特性常量 | S10 |
| `ErrorKind`（Business/Infrastructure/Validation/Internal） | 已实现 | 增加 `Unauthorized / Forbidden / RateLimited` 等 HTTP 语义变体（**注意**：HTTP 语义更适合放在 `vernal-core` 还是 `vernal-web` 由迁移路线图 S4 决定） | S10 |
| `ErrorReport`（脱敏报告） | 已实现 | 增加 `to_json()` 输出 + 字段脱敏黑名单 | S10 |
| `ErrorDomain`（10 域） | 已实现 | 增加 `MIGRATION / SECURITY / METRICS` 三个域常量 | S3 |
| `ErrorContext`（键值对） | 已实现 | 增加 `Display` 完整输出（可选 + `Display::full()` 方法） | S10 |

---

## 十一、tx_di `tx_error::axum_support` 等 Feature-gated 模块（**不迁移**）

| tx_di 模块 | 语义 | vernal-core 对应 | 状态 |
|---|---|---|---|
| `tx_error::axum_support` | axum 集成 | （不引入；按 Spring-style 分层，HTTP 错误映射归 `vernal-web`） | 🚫 |
| `tx_di_core::aop::*` | AOP 拦截器 | （不引入；按 Spring-style 分层，归 `vernal-aop` + `vernal-aspects`） | 🚫 |
| `tx_di_core::config::AppAllConfig` | 整体配置 | （不引入；归 `vernal-context::application_environment`） | 🚫 |

---

## 十二、汇总统计

| 来源 | Java/Rust 类计数 | 已迁移 | 待迁移 | 不迁移 |
|---|---|---|---|---|
| `spring-core` 顶级 | 23 | 3 ✅ | 4 ⬜ | 16 🚫 |
| `spring-core.convert` | 9 接口 + 19+ Converter | 6 ✅ | 5 🆕 | 17 🚫 |
| `spring-core` 资源/IO/注解 | 80+ | 0 | 0 | 80+ 🚫 |
| `spring-core.task` | 3 | 0 | 0 | 3 🚫 |
| `spring-core.retry` | 5 | 0 | 0 | 5 🚫 |
| `spring-core.serializer` | 4 | 0 | 0 | 4 🚫 |
| `spring-core.Observation`（6.1+） | 4 | 0 | 1 🆕 | 3 🚫 |
| `tx_di_core` | 12 类型 | 1 ✅ | 0 | 11 🚫 |
| `tx_error` | 12 类型 | 8 ✅/🆕 | 3 🆕（待 S2） | 1 🚫 |
| `tx_common` | 5 模块 | 1 ✅ | 1 🆕（待 S7） | 3 🚫 |
| **合计** | **~180** | **19** | **14** | **~145** |

迁移后 vernal-core 实际增加约 **14 个**待迁移 + **3 个新增**（S2 From 实现 / S7 ID 后端 / S9 Span），总计约 **17 个**新增 .rs 文件 / 子模块，达成 spring-core + tx_error 的最小可用子集。

---

## 十三、命名保留与命名冲突一览（详见《对象名称一致性检查.md》）

| Spring / tx_di 名称 | vernal-core 保留 | 重名 / 重定义风险 | 处理 |
|---|---|---|---|
| `ConversionService` | ✅（vernal-core 已有） | 与 `spring-core.convert.ConversionService` 行为等价但实现差异显著（trait vs 反射） | 文档注明 |
| `Ordered` | ✅（用 `INIT_SORT_*` 常量） | Spring `Ordered.getOrder()` 是 trait 方法；vernal 用常量 | 文档注明 |
| `LifecyclePhase` | ⬜ 重命名 | 与 vernal-context 同名 | **重命名为 `AppLifecyclePhase`**（路线图 S1） |
| `ErrorCode` | ✅ | 与 Spring `ErrorCoded` 略有不同 | 文档注明 |
| `BoxError` | ✅ | Spring 没有对应；tx_di 没有对应 | vernal 新增 |
| `SharedError` | ✅ | 同上 | vernal 新增 |
| `ObjectId` | ✅（24 位 hex） | Spring 没有对应；tx_common 用 UUID | vernal 选型；feature-gated 兼容 `uuid` / `ulid` / `nanoid` / `snowflake` |
| `StopWatch` | ✅ | Spring `org.springframework.util.StopWatch` 完全等价 | 文档注明 |
| `Convertible` | ✅ | Spring 没有对应 | vernal 新增 trait |
| `Converter<S, T>` | ✅ | Spring `org.springframework.core.convert.converter.Converter` 完全对齐 | 模块路径镜像 |
| `CodeMsg` | 🔶 | tx_error trait，vernal 用 `ErrorCode` 替代 | 不直接保留 |
| `AppError` | 🔶 | 重命名为 `VernalError` | 不直接保留 |
| `ErrorKind` | 🆕 | Spring `ErrorCoded` 没有此枚举；tx_di 没有 | vernal 新增 |
| `ErrorContext` | 🆕 | Spring `NestedExceptionUtils` 没有此结构；tx_di 没有 | vernal 新增 |
| `ErrorReport` | 🆕 | Spring `ErrorResponseException` 6.1 类似但更重；tx_di 没有 | vernal 新增（轻量版） |
| `ErrorDomain` | 🆕 | Spring namespace；tx_di `#[err("DI")]` 属性 | vernal 统一为常量 |
| `Span` | 🆕（待 S9） | Spring 6.1 `Observation` | vernal 轻量版 |

---

## 十四、Rust 生态集成建议（**新增章节**）

vernal-core 默认**零外部依赖**。通过 feature flag 集成下列 crate（经过 crates.io 实际调研，**Tier 1+2 推荐必选，Tier 3 可选**）：

| 关注点 | 推荐 crate | 版本 | 许可证 | 阶段 | Feature 名称 | 说明 |
|---|---|---|---|---|---|---|
| `Error` 派生宏（业务 crate 用） | `thiserror` | `2.0.19` | MIT/Apache-2.0 | S8 | `error-derive` | **强烈推荐** —— `VernalError` 手写 Display/Error，用 `#[derive(Error)]` 可精简代码且零运行时开销 |
| 宏拼接 | `pastey` | `0.2.3` | MIT/Apache-2.0 | S8 | `macros` | 现代 `paste` 继任者，体积小、proc-macro 更简洁 |
| 分布式注册 | `inventory` | `0.3.24` | MIT/Apache-2.0 | S8 | `registry` | stable Rust 上"非宏插件"的最成熟方案，备选 `linkme` |
| 框架级单例 | `once_cell` | `1.21.4` | MIT/Apache-2.0 | S8 | `once-cell` | std::sync::OnceLock 不支持 lazy fn，once_cell 是 vernal-core 标配 |
| 派生补充 | `derive_more` | `2.1.1` | MIT | S8 | `derive-extras` | 补充 thiserror 的 `Display`/`From` 等派生 |
| UUID v4/v7 | `uuid` | `1.24.0` | MIT/Apache-2.0 | S7+S8 | `id-uuid` / `convert-uuid` | 全局唯一 ID，事实标准 |
| ULID | `ulid` | `3.0.0` | MIT | S7+S8 | `id-ulid` | 时间排序 ID（分布式场景） |
| NanoId | `nanoid` | `0.5.0` | MIT | S7+S8 | `id-nanoid` | 短 URL-friendly ID |
| URL 解析 | `url` | `2.5.8` | MIT/Apache-2.0 | S5+S8 | `convert-url` | web 栈配置属性绑定必备 |
| 字节 | `bytes` | `1.12.1` | MIT | S5+S8 | `convert-bytes` | 网络栈配置属性绑定 |
| 高精度时间 | `time` | `0.3.54` | MIT/Apache-2.0 | S5+S8 | `convert-time` | MSRV 1.88 与 vernal-core 完美对齐 |
| 日期时间 | `chrono` | `0.4.45` | MIT/Apache-2.0 | S5+S8 | `convert-chrono` | 生态最广（Spring 用户熟悉 `LocalDateTime`），但需关闭默认 features |
| 序列化 | `serde` | `1.0.229` | MIT/Apache-2.0 | S7+S8 | `serde` | 仅对 `ObjectId` / `VernalError` 等用户可见类型做 derive |
| enum 大小写转换 | `convert_case` | `0.11.0` | MIT | S8 | `case-conv` | enum 名→string，vernal-beans/vernal-aop 可能用到 |
| WASM 时间 | `web-time` | `1.1.0` | MIT/Apache-2.0 | S8（**默认禁用**） | `time-web` | 仅当 vernal 计划支持 wasm32 时启用 |

### 14.1 排除决策（**绝不引入**）

| Crate | 排除原因 |
|---|---|
| `anyhow` 1.0.104 | 与 vernal `SharedError` 体系冲突；`From<anyhow>` 互转在 `vernal-bridge` 提供 |
| `tokio` 1.53.1 | runtime 耦合，vernal-core 必须保持 runtime-agnostic；由 `vernal-context` 引入 |
| `priority-queue` 2.7.0 | **LGPL-3.0 / MPL-2.0 copyleft**，污染 Vernal 的 MIT 协议 |
| `instant` 0.1.13 | **已停止维护**（README 推荐 fork 或用 web-time） |
| `snowflake` 1.3.0 | 2017 年最后更新，完全停滞；**自实现**（基于 uuid v7 + epoch） |
| `snowflake-rs` 0.1.1 | 自 2018 年起废弃，依赖过时的 `time` 0.1 |
| `idgen` 0.1.2 | 2019 年最后更新，孤儿 crate |
| `smartstring` 1.0.1 | **MPL-2.0+ 许可证** 与 MIT 项目兼容性需逐文件评估 |
| `paste` 1.0.15 | 已被 `pastey` 替代 |
| `conv` / `conv2` / `easy-cast` / `num_convert` | 抽象错位（这些是数值转换，Spring `ConversionService` 是 String→T） |
| `bson` 3.1.0 | 引入整个 MongoDB 驱动符号，ObjectId 应自行实现 |
| `redis` 1.4.1 | 职责不属于 core 层 |
| `async-trait` 0.1.91 | Rust 1.75+ 原生支持；vernal-core 不持有 dyn trait |
| `tracing` 0.1.44 | 归 `vernal-log`，vernal-core 自身不打印日志 |

### 14.2 集成原则

1. vernal-core 默认 `cargo build` 通过，无任何外部依赖
2. 所有上述 crate 均以 **可选 feature flag** 形式提供
3. `VernalError::Infrastructure` 用 `Arc<dyn Error>` 替代 anyhow
4. `BoxError` / `SharedError` 与 anyhow 的互转在 `vernal-bridge` crate 提供
5. `web-time` 在 native 是 `std::time::Instant` 的 alias，零额外开销
6. `chrono` 必须 `default-features = false` + 仅启用 `clock` + `std`，否则体积 + 时区数据膨胀
7. `time` 0.3.54 的 MSRV 恰好是 1.88，与 vernal-core 完美对齐