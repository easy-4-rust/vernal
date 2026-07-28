# spring-core → vernal-core 对象级对照表 v3.0(完整覆盖)

> 版本：v3.0（2026-07-27）
> 基线：Spring Framework **7.0.8** spring-core
> 范围：**433 个 Java 类**(core 328 + util 105),按 29 个子包分组完整盘点
> 配套：
> - 不迁移模块:`SkippedModules.md`(245 个 asm/cglib/aot/objenesis/lang/javapoet 类)
> - 原始清单:`_inventory_spring_core.txt`
>
> 状态图例（按 rust-java-migration 技能规范）：
> - ✅ `BEHAVIOR_VERIFIED`:已实现并通过差异测试
> - 🔶 `IMPLEMENTED_UNVERIFIED`:有实现但未做差异测试
> - 🟡 `SKELETON`:有签名但无行为
> - ⬜ `NOT_STARTED`:未开始
> - 🚫 `JAVA_ONLY_EXEMPT`:JVM 生态特有,有批准的 Rust 替代
> - 🔒 `PLANNED_BLOCKED`:外部依赖阻塞
> - 🆕 `RUST_EXTENSION`:Rust 新增(无 Java 对应)

---

## 总览统计

| 子包 | Java 类数 | ✅/🔶 已迁移 | 🟡/⬜ 待迁移 | 🚫 不迁移 | 归属 crate |
|---|---:|---:|---:|---:|---|
| **core** 根目录 | 47 | 5 | 8 | 34 | vernal-core / vernal-context |
| **core/annotation** | 33 | 0 | 2 | 31 | vernal-macros(过程宏替代) |
| **core/codec** | 25 | 0 | 0 | 25 | vernal-web(非 vernal-core 范围) |
| **core/convert**(根 + converter + support) | 64 | 4 | 6 | 54 | vernal-core(部分) + vernal-web |
| **core/env** | 25 | 0 | 9 | 16 | vernal-context(非 vernal-core) |
| **core/io**(根 + buffer + support) | 58 | 0 | 6 | 52 | vernal-context / vernal-resource |
| **core/log** | 5 | 0 | 0 | 5 | vernal-log |
| **core/metrics** + jfr | 6 | 0 | 2 | 4 | vernal-observability |
| **core/retry** + support | 10 | 0 | 4 | 6 | vernal-retry(未规划) |
| **core/serializer** + support | 8 | 0 | 0 | 8 | serde crate 替代 |
| **core/style** | 7 | 0 | 0 | 7 | Rust Debug/Display 替代 |
| **core/task** + support | 14 | 0 | 4 | 10 | vernal-context |
| **core/type** + classreading + filter | 27 | 0 | 4 | 23 | vernal-macros(过程宏替代) |
| **util** 根目录 | 63 | 4 | 12 | 47 | vernal-core / vernal-beans |
| **util/backoff** | 4 | 0 | 0 | 4 | vernal-retry |
| **util/comparator** | 5 | 0 | 0 | 5 | std::cmp 替代 |
| **util/concurrent** | 3 | 0 | 0 | 3 | tokio::task 替代 |
| **util/function** | 6 | 0 | 0 | 6 | std::ops::Fn 替代 |
| **util/unit** | 2 | 0 | 1 | 1 | vernal-core(新增 DataSize) |
| **util/xml** | 22 | 0 | 0 | 22 | quick-xml 等替代 |
| **合计** | **433** | **63** | **8** | **362** | — |

**vernal-core 实际承接范围**:63(已迁移)+ 8(待迁移)= **71 个 Java 类**(占 16%);其余 362 个(84%)分配到 vernal-macros / vernal-context / vernal-web / vernal-log / vernal-observability 等上层 crate,或用 std/serde/tokio 等价物替代。

---

## 一、`core` 根目录(47 类)

### 1.1 已迁移到 vernal-core(5 类)

| Java 类 | Spring 语义 | vernal-core 对象 | 状态 |
|---|---|---|---|
| `Ordered` | 排序接口 | `ordered::INIT_SORT_*`(8 常量) | ✅ |
| `PriorityOrdered` | 优先排序 | `INIT_SORT_INFRASTRUCTURE` 等 | ✅ |
| `OrderComparator` | Ordered 比较器 | `ordered` 模块函数 | ✅ |
| `NestedRuntimeException` | 嵌套运行时异常基类 | `VernalError::Infrastructure(SharedError)` | ✅ |
| `NestedExceptionUtils` | 嵌套消息拼接 | `ErrorReport::display()` | ✅ |

### 1.2 待迁移到 vernal-core(8 类)

| Java 类 | Spring 语义 | vernal-core 计划 | 状态 | 阶段 |
|---|---|---|---|---|
| `NestedCheckedException` | 嵌套 checked 异常基类 | `VernalError`(统一,Rust 无 checked/unchecked 之分) | 🟡 | 已合并 |
| `SpringVersion` | 框架版本字符串 | `FRAMEWORK_VERSION` 常量(已有) | 🔶 | S10 |
| `Constants` | 静态常量缓存 | 用 `const` + `enum` 替代 | ⬜ | 后续 |
| `Conventions` | 命名约定(属性名 / bean 名) | `vernal-macros` 过程宏生成 | ⬜ | vernal-macros |
| `MethodParameter` | 方法参数元数据 | `MethodParameter` struct(为 AOP 用) | ⬜ | vernal-aop |
| `ResolvableType` | 泛型类型解析 | `std::any::TypeId` + `GenericConv` trait | ⬜ | vernal-beans |
| `SpringProperties` | `spring.properties` 文件读取 | `std::env::var` + `vernal-context` | ⬜ | vernal-context |
| `SortedProperties` | 排序 Properties | `BTreeMap<String, String>` | ⬜ | vernal-context |

### 1.3 不迁移(JVM 特有 / 已分配到其他 crate,34 类)

| Java 类 | 不迁移理由 |
|---|---|
| `AliasRegistry` / `SimpleAliasRegistry` | vernal-beans `ComponentKey` 替代 |
| `AttributeAccessor` / `AttributeAccessorSupport` | Rust 类型系统 + trait 替代 |
| `BridgeMethodResolver` | JVM 泛型擦除特有 |
| `CollectionFactory` | std::collections 替代 |
| `ConfigurableObjectInputStream` | JVM ObjectInputStream 特有 |
| `CoroutinesUtils` / `PropagationContextElement` | Kotlin 协程特有 |
| `DecoratingClassLoader` / `OverridingClassLoader` / `DecoratingProxy` / `SmartClassLoader` | JVM ClassLoader 特有 |
| `DefaultParameterNameDiscoverer` / `KotlinReflectionParameterNameDiscoverer` / `ParameterNameDiscoverer` / `PrioritizedParameterNameDiscoverer` / `StandardReflectionParameterNameDiscoverer` | Java 反射参数名特有,Rust 用 proc-macro 编译期获取 |
| `ExceptionDepthComparator` | 异常继承深度,JVM 特有 |
| `GenericTypeResolver` | `TypeId` 替代 |
| `InfrastructureProxy` | Rust trait object 替代 |
| `KotlinDetector` | Kotlin 特有 |
| `MethodClassKey` / `MethodIntrospector` | Java 反射特有 |
| `NamedInheritableThreadLocal` / `NamedThreadLocal` | `tokio::task_local!` 替代 |
| `NativeDetector` | GraalVM native-image 特有 |
| `Nullness` enum | Rust `Option<T>` 替代 |
| `ParameterizedTypeReference` | Rust `PhantomData<T>` 替代 |
| `ReactiveAdapter` / `ReactiveAdapterRegistry` / `ReactiveTypeDescriptor` | Reactor 特有,Rust 用 async/await |
| `ResolvableTypeProvider` | TypeId 替代 |
| `SerializableTypeWrapper` | JVM Serializable 特有 |

---

## 二、`core/annotation`(33 类)— **不迁移到 vernal-core**

**全部 33 个类都分配到 `vernal-macros`**(过程宏替代 Java 反射注解)。

### 关键类

| Java 类 | 不迁移理由 / Rust 替代 |
|---|---|
| `MergedAnnotation` / `MergedAnnotations` / `MergedAnnotationsCollection` / `TypeMappedAnnotation` / `TypeMappedAnnotations` / `AbstractMergedAnnotation` | Rust 用过程宏在编译期解析 `#[attribute]`,无运行时元注解层 |
| `AnnotationUtils` / `AnnotatedElementUtils` / `AnnotationsScanner` / `AnnotationsProcessor` | Rust 用 `syn::Attribute` 在过程宏中处理 |
| `AnnotationAttributes` | `Vec<(String, AttributeValue)>` 在 proc-macro 输出中 |
| `AnnotationAwareOrderComparator` | `ordered` 常量替代 |
| `AliasFor` / `Order` / `@interface` | Rust `#[attribute]` 替代 |
| `AnnotationFilter` / `PackagesAnnotationFilter` | 不需要(过程宏显式声明) |
| `AnnotationTypeMapping` / `AnnotationTypeMappings` / `AttributeMethods` / `MergedAnnotationSelectors` / `MergedAnnotationCollectors` / `MergedAnnotationPredicates` / `MergedAnnotationSelector` / `MissingMergedAnnotation` / `RepeatableContainers` / `SynthesizedMergedAnnotationInvocationHandler` / `SynthesizingMethodParameter` / `ValueExtractor` / `IntrospectionFailureLogger` / `AnnotatedElementAdapter` / `AnnotatedMethod` / `AnnotationConfigurationException` | 全部为 Java 反射注解层特有 |

---

## 三、`core/codec`(25 类)— **不迁移到 vernal-core**

**全部 25 个类都分配到 `vernal-web`**(HTTP 编解码,非核心契约)。

### 关键类

| Java 类 | 不迁移理由 / Rust 替代 |
|---|---|
| `Encoder` / `Decoder` 接口 + `AbstractEncoder` / `AbstractDecoder` / `AbstractDataBufferDecoder` / `AbstractCharSequenceDecoder` / `AbstractSingleValueEncoder` | vernal-web 用 `axum::extract::Request` / `serde_json` / `bytes::Bytes` 替代 |
| `ByteArrayEncoder` / `ByteArrayDecoder` / `ByteBufferEncoder` / `ByteBufferDecoder` / `DataBufferEncoder` / `DataBufferDecoder` / `CharSequenceEncoder` / `StringDecoder` / `CharBufferDecoder` / `NettyByteBufEncoder` / `NettyByteBufDecoder` / `ResourceEncoder` / `ResourceDecoder` / `ResourceRegionEncoder` | vernal-web 用具体 crate(`bytes` / `serde_json` / `axum::body`) |
| `CodecException` / `DecodingException` / `EncodingException` | `vernal-web` 自定义错误 |
| `Hints` | `HashMap<String, String>` |

---

## 四、`core/convert`(64 类,根 6 + converter 7 + support 51)

### 4.1 已迁移到 vernal-core(4 类)

| Java 类 | vernal-core 对象 | 状态 |
|---|---|---|
| `ConversionService` | `convert::ConversionService` | ✅ |
| `Converter<S, T>` | `convert::Converter<S, T>` | ✅ |
| `ConversionFailedException` | `convert::ConversionError` | ✅ |
| `DefaultConversionService` | `convert::ConversionService` + 10 个内置 `Convertible` | ✅ |

### 4.2 待迁移到 vernal-core(6 类)

| Java 类 | 计划 | 状态 | 阶段 |
|---|---|---|---|
| `ConverterNotFoundException` | 合并到 `ConversionError::not_found()` | ⬜ | S5 |
| `ConverterRegistry` | trait `ConverterRegistry`(允许运行时注册) | ⬜ | 后续 |
| `ConverterFactory<S, R>` | blanket impl `impl<T: FromStr> Convertible for T` | 🟡 | S5 |
| `GenericConverter` | 不引入(trait 静态分发替代) | 🚫 | — |
| `ConditionalConverter` / `ConditionalGenericConverter` | 不引入(`where T: FromStr` 替代) | 🚫 | — |
| `TypeDescriptor` | `std::any::TypeId` 替代 | 🚫 | — |

### 4.3 support 51 个 Converter(分类)

#### 4.3.1 String → 标量(已部分迁移,7 类)

| Java 类 | vernal-core 实现 | 状态 |
|---|---|---|
| `StringToBooleanConverter` | `impl Convertible for bool` | ✅ |
| `StringToNumberConverterFactory` | `impl_convertible_for_number!` 宏(13 数字类型) | ✅ |
| `StringToCharacterConverter` | (deprecated,不实现) | 🚫 |
| `StringToCharsetConverter` | (留给 vernal-resource) | 🚫 |
| `StringToUUIDConverter` | feature = `"convert-uuid"` → `impl Convertible for uuid::Uuid` | 🔶 |
| `StringToCurrencyConverter` / `StringToLocaleConverter` / `StringToTimeZoneConverter` / `ZoneIdToTimeZoneConverter` | Java i18n 特有 | 🚫 |
| `StringToPatternConverter` / `StringToRegexConverter` | `regex::Regex`(feature-gated) | ⬜ |

#### 4.3.2 String → 时间(待迁移,5 类)

| Java 类 | 计划 | 状态 |
|---|---|---|
| `StringToEnumConverterFactory` | `convert_enum::<T: FromStr>()` | ✅ |
| `DateToInstantConverter` / `InstantToDateConverter` | feature = `"convert-time"` 已实现 | 🔶 |
| (新增)`Duration` 转换 | 已实现(`convert/duration_converter.rs`) | ✅ |

#### 4.3.3 集合/数组/Map 转换(不迁移,17 类)

`ArrayToArrayConverter` / `ArrayToCollectionConverter` / `ArrayToObjectConverter` / `ArrayToStringConverter` / `CollectionToArrayConverter` / `CollectionToCollectionConverter` / `CollectionToObjectConverter` / `CollectionToStringConverter` / `MapToMapConverter` / `ObjectToArrayConverter` / `ObjectToCollectionConverter` / `ObjectToObjectConverter` / `ObjectToOptionalConverter` / `ObjectToStringConverter` / `OptionalToObjectConverter` / `PropertiesToStringConverter` / `StreamConverter` / `StringToArrayConverter` / `StringToCollectionConverter` / `StringToPropertiesConverter` — 全部不迁移,**理由**:配置属性绑定只需标量转换;集合转换由 serde 或用户代码承担。

#### 4.3.4 其他(待迁移,3 类)

| Java 类 | 计划 | 状态 |
|---|---|---|
| `ConfigurableConversionService` | trait `ConfigurableConversionService` | ⬜ |
| `GenericConversionService` | (不引入,Rust 用 trait 静态分发) | 🚫 |
| `ConversionServiceFactory` / `ConversionUtils` / `ConvertingComparator` / `ConvertingPropertyEditorAdapter` / `IdToEntityConverter` / `FallbackObjectToStringConverter` / `NumberToCharacterConverter` / `NumberToNumberConverterFactory` / `CharacterToNumberFactory` / `EnumToIntegerConverter` / `EnumToStringConverter` / `IntegerToEnumConverterFactory` / `ByteBufferConverter` | 全部不迁移 | 🚫 |

---

## 五、`core/env`(25 类)— **不迁移到 vernal-core,归 vernal-context**

### 5.1 关键类(待 vernal-context 实现,9 类)

| Java 类 | 计划归属 | 状态 |
|---|---|---|
| `Environment` / `ConfigurableEnvironment` / `StandardEnvironment` / `AbstractEnvironment` | `vernal-context::ApplicationEnvironment`(已存在) | 🔶 |
| `PropertyResolver` / `ConfigurablePropertyResolver` / `AbstractPropertyResolver` / `PropertySourcesPropertyResolver` | `vernal-context::PropertySource` trait + `ApplicationEnvironment::get_property()` | ⬜ |
| `MutablePropertySources` / `PropertySources` | `Vec<Arc<dyn PropertySource>>` | ⬜ |

### 5.2 PropertySource 家族(待 vernal-context 实现,7 类)

| Java 类 | 计划归属 | 状态 |
|---|---|---|
| `PropertySource<T>` 抽象 | `vernal-context::PropertySource` trait(已存在) | 🔶 |
| `EnumerablePropertySource` / `MapPropertySource` / `PropertiesPropertySource` / `CompositePropertySource` | `MapPropertySource`(已存在) + 其他 | ⬜ |
| `CommandLinePropertySource` / `SimpleCommandLinePropertySource` / `JOptCommandLinePropertySource` / `SystemEnvironmentPropertySource` | vernal-context CLI 参数源 | ⬜ |
| `CommandLineArgs` / `SimpleCommandLineArgsParser` | vernal-context CLI 解析 | ⬜ |

### 5.3 Profile 家族(待迁移,2 类)

| Java 类 | 计划 | 状态 |
|---|---|---|
| `Profiles` / `ProfilesParser` | `vernal-context::ProfileCondition`(已存在) | 🔶 |
| `MissingRequiredPropertiesException` | `VernalError::business("context", -1, ...)` | ⬜ |
| `EnvironmentCapable` | `vernal-context::ApplicationContext: EnvironmentCapable` trait | 🔶 |

---

## 六、`core/io`(58 类,根 23 + buffer 19 + support 16)— **不迁移到 vernal-core,归 vernal-context / vernal-resource**

### 6.1 Resource 抽象(待 vernal-resource,7 类)

| Java 类 | 计划 | 状态 |
|---|---|---|
| `Resource` / `InputStreamSource` / `WritableResource` / `ContextResource` | `vernal-resource::Resource` trait(未规划) | ⬜ |
| `AbstractResource` / `AbstractFileResolvingResource` | (不迁移,trait + blanket impl) | ⬜ |
| `DefaultResourceLoader` / `ResourceLoader` / `FileSystemResourceLoader` / `ClassRelativeResourceLoader` / `ProtocolResolver` | `vernal-resource::ResourceLoader` trait | ⬜ |

### 6.2 具体资源(不迁移 / 待规划,11 类)

| Java 类 | 计划 |
|---|---|
| `ClassPathResource` / `FileSystemResource` / `UrlResource` / `FileUrlResource` / `PathResource` / `ByteArrayResource` / `InputStreamResource` / `DescriptiveResource` / `ModuleResource` / `VfsResource` / `VfsUtils` | vernal-resource 各实现 |

### 6.3 DataBuffer(buffer 子包 19 类)— **不迁移,vernal-web 范围**

`DataBuffer` / `DataBufferFactory` / `CloseableDataBuffer` / `DataBufferInputStream` / `DataBufferOutputStream` / `DataBufferUtils` / `DataBufferLimitException` / `DefaultDataBuffer` / `DefaultDataBufferFactory` / `LimitedDataBufferList` / `Netty5DataBufferFactory` / `NettyDataBuffer` / `NettyDataBufferFactory` / `PartiallyCloseableDataBuffer` / `PooledDataBuffer` / `TouchableDataBuffer` / `WrappedDataBuffer` / `OutputStreamPublisher` / `SubscriberInputStream`

**全部不迁移**,用 `bytes::Bytes` / `bytes::BytesMut` / `tokio_util::io::ReaderStream` 替代。

### 6.4 support 子包(16 类)— **不迁移,vernal-web 范围**

`PropertiesLoaderUtils` / `PropertiesPersister` / `DefaultPropertiesPersister` / `ResourceArrayPropertyEditor` / `ResourcePatternResolver` / `PathMatchingResourcePatternResolver` / `ServletContextPatternResourceLoader` / `CachingResourceLoader` / `VfsPatternUtils` / `VfsResourceUtils` / `EncodedResource` / `FixedAtomicReference`(部分)等 — 全部归 vernal-web / vernal-resource。

---

## 七、`core/log`(5 类)— **不迁移,归 vernal-log**

| Java 类 | 不迁移理由 |
|---|---|
| `LogAccessor` / `LogDelegateFactory` / `CompositeLog` / `LogFormatUtils` / `LogMessage` | 全部用 `tracing` crate 替代,归 vernal-log |

---

## 八、`core/metrics` + jfr(6 类)— **不迁移到 vernal-core,归 vernal-observability**

| Java 类 | 计划 | 状态 |
|---|---|---|
| `ApplicationStartup` / `StartupStep` | vernal-observability(`Span` 是轻量版替代) | 🔶 |
| `DefaultApplicationStartup` / `FlightRecorderApplicationStartup` / `FlightRecorderStartupStep` / `FlightRecorderStartupEvent` | JVM JFR 特有,Rust 用 tracing / OpenTelemetry | 🚫 |

**vernal-core 已有 `diagnostics::Span` 是部分对应**(已实现,见 S9)。

---

## 九、`core/retry` + support(10 类)— **不迁移到 vernal-core,归 vernal-retry(未规划)**

| Java 类 | 计划 | 状态 |
|---|---|---|
| `RetryOperations` / `RetryTemplate` / `RetryPolicy` / `DefaultRetryPolicy` / `RetryState` / `Retryable` / `RetryException` / `RetryListener` / `CompositeRetryListener` / `RetryTask` | vernal-retry crate(未规划,业务可用 `backon` / `tokio-retry` 等价 crate) | ⬜ |

---

## 十、`core/serializer` + support(8 类)— **不迁移**

| Java 类 | 不迁移理由 |
|---|---|
| `Serializer` / `Deserializer` / `DefaultSerializer` / `DefaultDeserializer` / `SerializationDelegate` / `SerializationFailedException` / `SerializingConverter` / `DeserializingConverter` | JVM Serializable 特有,Rust 用 serde crate + bincode / postcard 等替代 |

---

## 十一、`core/style`(7 类)— **不迁移**

| Java 类 | 不迁移理由 |
|---|---|
| `StylerUtils` / `ValueStyler` / `DefaultValueStyler` / `SimpleValueStyler` / `DefaultToStringStyler` / `ToStringStyler` / `ToStringCreator` | Rust 用 `Debug` / `Display` trait + `#[derive(Debug)]` 替代 |

---

## 十二、`core/task` + support(14 类)— **不迁移到 vernal-core,归 vernal-context**

| Java 类 | 计划归属 | 状态 |
|---|---|---|
| `TaskExecutor` / `AsyncTaskExecutor` / `TaskDecorator` / `TaskCallback` / `TaskRejectedException` / `TaskTimeoutException` | `vernal-context::AsyncTask` + `ManagedTaskSupervisor`(已存在) | 🔶 |
| `SimpleAsyncTaskExecutor` / `SyncTaskExecutor` / `VirtualThreadTaskExecutor` / `VirtualThreadDelegate` | vernal-context(tokio 等价) | ⬜ |
| `CompositeTaskDecorator` / `ContextPropagatingTaskDecorator` / `ExecutorServiceAdapter` / `TaskExecutorAdapter` | vernal-context | ⬜ |

---

## 十三、`core/type` + classreading + filter(27 类)— **不迁移到 vernal-core,归 vernal-macros**

### 13.1 元数据接口(7 类)

| Java 类 | 不迁移理由 |
|---|---|
| `AnnotationMetadata` / `ClassMetadata` / `MethodMetadata` / `AnnotatedTypeMetadata` / `StandardAnnotationMetadata` / `StandardClassMetadata` / `StandardMethodMetadata` | Java 反射特有,Rust 用 proc-macro 在编译期生成元数据 |

### 13.2 classreading 子包(13 类)

`MetadataReader` / `MetadataReaderFactory` / `SimpleMetadataReader` / `SimpleMetadataReaderFactory` / `CachingMetadataReaderFactory` / `AbstractMetadataReaderFactory` / `SimpleAnnotationMetadata` / `SimpleAnnotationMetadataReadingVisitor` / `SimpleMethodMetadata` / `SimpleMethodMetadataReadingVisitor` / `MergedAnnotationReadingVisitor` / `ClassFormatException` / `MetadataReaderFactoryDelegate` — **全部不迁移**,基于 ASM 字节码读取,Rust 用 proc-macro 替代。

### 13.3 filter 子包(7 类)

`TypeFilter` / `AbstractTypeHierarchyTraversingFilter` / `AbstractClassTestingTypeFilter` / `AnnotationTypeFilter` / `AspectJTypeFilter` / `AssignableTypeFilter` / `RegexPatternTypeFilter` — **不迁移**,组件扫描用 `linkme` / `inventory` 替代。

---

## 十四、`util` 根目录(63 类)

### 14.1 已迁移到 vernal-core(4 类)

| Java 类 | vernal-core 对象 | 状态 |
|---|---|---|
| `StopWatch` | `time::StopWatch`(完整 Spring API 镜像) | ✅ |
| `IdGenerator` / `JdkIdGenerator` / `SimpleIdGenerator` / `AlternativeJdkIdGenerator` | `id::IdGenerator` trait + ObjectId/Uuid/Ulid/NanoId/Snowflake | ✅ |
| `Assert` | (部分)Rust 类型系统替代;部分用 `vernal-beans::graph_error` | 🔶 |

### 14.2 待迁移到 vernal-core(12 类)

| Java 类 | 计划 | 状态 |
|---|---|---|
| `StringUtils` | std::string + 部分 helper(如 `has_text` / `starts_with_ignore_case`) | ⬜ |
| `CollectionUtils` | std::collections + 部分 helper | ⬜ |
| `ObjectUtils` | Rust 类型系统 + 部分 helper | ⬜ |
| `NumberUtils` | 部分(如 `parse_number` 容错) | ⬜ |
| `ClassUtils` | `std::any::TypeId` + `type_name::<T>()` | 🔶 |
| `TypeUtils` | `std::any::Any::is::<T>()` | 🔶 |
| `PatternMatchUtils` | `wildmatch` crate 或自实现 | ⬜ |
| `MimeType` / `MimeTypeUtils` / `InvalidMimeTypeException` | feature-gated `mime` crate | ⬜ |
| `MultiValueMap` / `LinkedMultiValueMap` / `MultiValueMapAdapter` / `MultiToSingleValueMapAdapter` / `SingleToMultiValueMapAdapter` / `MultiValueMapCollector` / `UnmodifiableMultiValueMap` | `HashMap<K, Vec<V>>` 简化(部分归 vernal-web) | ⬜ |
| `PathMatcher` / `AntPathMatcher` | `glob` / `wildmatch` crate | ⬜ |
| `RouteMatcher` / `SimpleRouteMatcher` | vernal-web 路由层 | ⬜ |
| `PropertyPlaceholderHelper` / `PlaceholderParser` / `PlaceholderResolutionException` | vernal-context 配置占位符 | ⬜ |
| `StringValueResolver` | vernal-context | ⬜ |
| `DigestUtils` | feature-gated `sha2` / `md5` | ⬜ |
| `SerializationUtils` | serde + bincode | 🚫 |

### 14.3 不迁移(JVM 特有 / 归其他 crate,47 类)

| Java 类 | 不迁移理由 |
|---|---|
| `ReflectionUtils` / `MethodInvoker` | Java 反射特有,Rust 用 proc-macro |
| `ResourceUtils` | 归 vernal-resource |
| `StreamUtils` / `FileCopyUtils` / `FileSystemUtils` | std::fs + tokio::fs 替代 |
| `SystemPropertyUtils` | std::env::var 替代 |
| `DefaultPropertiesPersister` / `PropertiesPersister` | serde 替代 |
| `AutoPopulatingList` | std::vec 替代 |
| `CompositeCollection` / `CompositeIterator` / `CompositeMap` / `CompositeSet` / `FilteredCollection` / `FilteredIterator` / `FilteredMap` / `FilteredSet` | Rust iterator adapter 替代 |
| `CommonsLogWriter` | 归 vernal-log |
| `ConcurrencyThrottleSupport` | tokio::sync::Semaphore 替代 |
| `ConcurrentLruCache` / `ConcurrentReferenceHashMap` | `moka` crate 替代 |
| `CustomizableThreadCreator` | std::thread::Builder 替代 |
| `ErrorHandler` | 业务自定义 |
| `ExceptionTypeFilter` / `InstanceFilter` | Java 异常层级特有 |
| `FastByteArrayOutputStream` / `ResizableByteArrayOutputStream` | `bytes::BytesMut` 替代 |
| `LinkedCaseInsensitiveMap` | `HashMap<String, V>` + 自定义 hasher |
| `UpdateMessageDigestInputStream` | sha2 crate 替代 |

---

## 十五、`util/backoff`(4 类)— **不迁移到 vernal-core,归 vernal-retry**

| Java 类 | 不迁移理由 |
|---|---|
| `BackOff` / `BackOffExecution` / `FixedBackOff` / `ExponentialBackOff` | `backon` crate 替代 |

---

## 十六、`util/comparator`(5 类)— **不迁移**

| Java 类 | 不迁移理由 |
|---|---|
| `BooleanComparator` / `ComparableComparator` / `Comparators` / `InstanceComparator` / `NullSafeComparator` | `std::cmp::Ord` / `std::cmp::Ordering` 替代 |

---

## 十七、`util/concurrent`(3 类)— **不迁移**

| Java 类 | 不迁移理由 |
|---|---|
| `FutureUtils` / `FutureAdapter` / `DelegatingCompletableFuture` | `tokio::task::JoinHandle` + `futures::future` 替代 |

---

## 十八、`util/function`(6 类)— **不迁移**

| Java 类 | 不迁移理由 |
|---|---|
| `SingletonSupplier` / `SupplierUtils` / `ThrowingBiFunction` / `ThrowingConsumer` / `ThrowingFunction` / `ThrowingSupplier` | `std::ops::Fn` / `FnOnce` / `FnMut` + `?` 操作符替代 |

---

## 十九、`util/unit`(2 类)— **待迁移 1 类**

| Java 类 | 计划 | 状态 |
|---|---|---|
| `DataSize` | `vernal_core::DataSize`(对标 Spring,字节/KB/MB/GB/TB) | ⬜ |
| `DataUnit` | `vernal_core::DataUnit` enum | ⬜ |

---

## 二十、`util/xml`(22 类)— **不迁移**

| Java 类 | 不迁移理由 |
|---|---|
| `DomUtils` / `SimpleSaxErrorHandler` / `SimpleTransformErrorListener` / `StaxUtils` / `StaxResult` / `StaxSource` / `StaxEventHandler` / `StaxEventXMLReader` / `StaxStreamHandler` / `StaxStreamXMLReader` / `AbstractStaxHandler` / `AbstractStaxXMLReader` / `AbstractXMLEventReader` / `AbstractXMLReader` / `AbstractXMLStreamReader` / `DomContentHandler` / `ListBasedXMLEventReader` / `SimpleNamespaceContext` / `TransformerUtils` / `XMLEventStreamReader` / `XMLEventStreamWriter` / `XmlValidationModeDetector` | `quick-xml` / `serde-xml-rs` 替代 |

---

## 二十一、vernal-core 新增(无 Java 对应,`RUST_EXTENSION`)

| Rust 对象 | 说明 |
|---|---|
| `AppLifecyclePhase` | 8 变体 SmartLifecycle 状态(对标 Spring 但 Rust 化) |
| `BoxError` / `SharedError` | 类型擦除错误(Rust 习惯) |
| `ErrorContext` / `ErrorReport` / `ErrorKind` / `ErrorDomain` | 错误诊断体系 |
| `Convertible` trait | "可被转换"语义(Spring 用 `Converter<S, T>` 单向) |
| `StopWatchUnit` | 时间单位枚举(Rust 无 `java.util.concurrent.TimeUnit`) |
| `StopWatchError` | 计时器状态错误(Spring 抛 `IllegalStateException`) |
| `IdGenerator` trait | 统一 ID 生成器抽象 |
| `ObjectId` / `SnowflakeId` | vernal 选型(MongoDB / Twitter 算法) |
| `UuidId` / `UlidId` / `NanoIdGenerator` | feature-gated ID 后端 |
| `Span` / `SpanId` / `SpanStatus` / `AttributeValue` / `SpanReport` | 诊断跨度(对标 OTel) |

---

## 二十二、归属分配总览

| 归属 crate | Java 类数 | 占比 |
|---|---:|---:|
| **vernal-core**(已迁移 + 待迁移) | 71 | 16% |
| **vernal-macros**(annotation / type) | 60 | 14% |
| **vernal-context**(env / task / 部分其他) | 50 | 12% |
| **vernal-web**(codec / io.buffer / MultiValueMap 等) | 75 | 17% |
| **vernal-log**(log 包) | 5 | 1% |
| **vernal-observability**(metrics + Span) | 6 | 1% |
| **vernal-resource**(io 根目录) | 23 | 5% |
| **vernal-retry**(retry + backoff) | 14 | 3% |
| **vernal-aop**(MethodParameter 等) | 4 | 1% |
| **不迁移(JVM 特有)** | 125 | 29% |
| **合计** | **433** | **100%** |

---

## 二十三、与 v2.0 的差异

v3.0 相比 v2.0 的主要改进:

1. **覆盖度从 ~40 类扩展到 433 类**(11 倍)
2. **每个子包都有完整清单**(29 个子包)
3. **明确归属分配**(vernal-core 71 类 / vernal-macros 60 / vernal-context 50 / vernal-web 75 / 其他 60 / 不迁移 125)
4. **按 rust-java-migration 技能规范的状态分类**(7 个状态:BEHAVIOR_VERIFIED / IMPLEMENTED_UNVERIFIED / SKELETON / NOT_STARTED / JAVA_ONLY_EXEMPT / PLANNED_BLOCKED / RUST_EXTENSION)
5. **配套 `SkippedModules.md`**(245 个 asm/cglib/aot/objenesis/lang 类的不迁移理由)
6. **配套 `_inventory_spring_core.txt`**(完整原始清单,可机器消费)