<!-- migration-doc: authority=historical canonical=语义迁移对照表.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [语义迁移对照表.md](语义迁移对照表.md) 为准。

> **历史文档，非当前验收依据。** 当前版本见[语义迁移对照表](语义迁移对照表.md)。

# spring-test → vernal-test 功能语义迁移对照表

> 基线：Spring Framework **7.0.8**（spring-test），共 459 个主 Java 类 + 22 个 Kotlin DSL 文件 + ~80 个 Mock 类。
>
> 迁移原则：**功能语义对齐，实现方式 Rust 化**。
> Java 的反射、注解、类路径扫描分别映射为：`trait` 对象 + 过程宏 + 编译期模块解析。
> 底层 Mock / 参数化 / HTTP Mock / 异步测试 等机制**复用 Rust 生态**，vernal-test 不重复实现。

状态图例：
- ✅ 已迁移并有测试
- 🔶 语义等价但形态不同（需要适配层）
- ⬜ 未迁移（在路线图中）
- 🚫 不迁移（明确边界）

---

## 一、核心 TestContext 体系

### 1.1 TestContext trait

| Java 接口 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `TestContext` | 封装测试执行的上下文（测试类/方法/实例 + ApplicationContext + 属性） | `TestContext` trait | 🔶 骨架（仅 name） |
| `TestContextManager` | 管理 TestContext 生命周期 + 注册 Listener + 触发回调 | `TestContextManager` struct | ⬜ |
| `TestContextBootstrapper` | 引导器：创建 TestContext + 注册 Listener + 加载 Context | `TestContextBootstrapper` trait | ⬜ |
| `BootstrapContext` | 引导期上下文（configuration + applicationContext + cache） | `BootstrapContext` struct | ⬜ |
| `BootstrapUtils` | 引导工具（解析 @BootstrapWith、@ContextConfiguration） | `BootstrapUtils` | ⬜ |
| `BootstrapWith` | 自定义引导器元注解 | `BootstrapWith` 派生宏 | ⬜ |
| `AttributeAccessor` | 通用属性访问（get/set/remove/has） | `AttributeAccessor` trait | ⬜ |
| `TestContextAnnotationUtils` | 注解查找工具（findAnnotation、isAnnotated） | `TestContextAnnotationUtils` | ⬜ |
| `MethodInvoker` | 方法调用抽象（用于反射调用 @BeforeTransaction 等） | `MethodInvoker` trait | ⬜ |
| `DefaultMethodInvoker` | 默认基于反射的 MethodInvoker | `DefaultMethodInvoker` | ⬜ |
| `TestConstructor` | 自动检测构造器注入 | `TestConstructor` | ⬜ |
| `NestedTestConfiguration` | @Nested 类配置继承策略 | `NestedTestConfiguration` enum | ⬜ |
| `ApplicationContextFailureProcessor` | 容器加载失败处理器 | `ApplicationContextFailureProcessor` trait | ⬜ |
| `ContextLoadException` | 容器加载异常 | `ContextLoadException` | ⬜ |
| `DynamicPropertySource` | 动态属性源 | `DynamicPropertySource` 宏 | ⬜ |
| `DynamicProperty` | 注册动态属性值（通过 Supplier 延迟求值） | `DynamicProperty` 宏 | ⬜ |
| `DynamicPropertyRegistry` | 动态属性注册表 | `DynamicPropertyRegistry` struct | ⬜ |
| `DynamicPropertyRegistrar` | 注册回调 | `DynamicPropertyRegistrar` trait | ⬜ |
| `MergedContextConfiguration` | 合并后的上下文配置（含父级） | `MergedContextConfiguration` | ⬜ |
| `ContextConfigurationAttributes` | @ContextConfiguration 的解析结果 | `ContextConfigurationAttributes` | ⬜ |
| `ContextHierarchy` | 多层 @ContextConfiguration 层级 | `ContextHierarchy` | ⬜ |

### 1.2 ContextLoader 体系

| Java 类/接口 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `ContextLoader` | 根据配置加载 ApplicationContext | `ContextLoader` trait | ⬜ |
| `SmartContextLoader` | ContextLoader 的智能版（含处理 MergedContextConfiguration） | `SmartContextLoader` trait | ⬜ |
| `AbstractContextLoader` | ContextLoader 骨架 | `AbstractContextLoader` | ⬜ |
| `AbstractGenericContextLoader` | 通用加载器骨架 | `AbstractGenericContextLoader` | ⬜ |
| `AbstractGenericWebContextLoader` | Web 版通用加载器骨架 | `AbstractGenericWebContextLoader` | ⬜ |
| `AbstractDelegatingSmartContextLoader` | 委托加载器骨架 | `AbstractDelegatingSmartContextLoader` | ⬜ |
| `AnnotationConfigContextLoader` | 加载 @Configuration 类（基于 vernal-context） | `AnnotationConfigContextLoader` | ⬜ |
| `AnnotationConfigContextLoaderUtils` | 注解配置加载工具 | `AnnotationConfigContextLoaderUtils` | ⬜ |
| `AnnotationConfigWebContextLoader` | Web 版注解配置加载 | `AnnotationConfigWebContextLoader` | ⬜ |
| `GenericContextLoader` | 通用加载器（YAML / JSON / TOML 配置） | `GenericContextLoader` | ⬜ |
| `GenericXmlContextLoader` | 通用 XML 加载器（映射到 YAML/JSON） | `GenericXmlContextLoader` | 🔶 XML 改 YAML/JSON |
| `AotContextLoader` | AOT 编译期加载器 | `AotContextLoader` | ⬜ |
| `ContextLoaderUtils` | 加载工具 | `ContextLoaderUtils` | ⬜ |

### 1.3 ContextCache 体系

| Java 类/接口 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `ContextCache` | ApplicationContext 缓存（按 key 缓存） | `ContextCache` trait | ⬜ |
| `DefaultContextCache` | 基于内存的 LRU 缓存 | `DefaultContextCache`（基于 `moka` crate） | 🔶 复用 moka |
| `CacheAwareContextLoaderDelegate` | 缓存感知的 ContextLoader 委托 | `CacheAwareContextLoaderDelegate` | ⬜ |
| `ContextCacheUtils` | 缓存工具 | `ContextCacheUtils` | ⬜ |

### 1.4 AOT 测试支持

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `AotTestAttributes` | AOT 测试属性 | `AotTestAttributes` | ⬜ |
| `AotTestAttributesFactory` | 工厂 | `AotTestAttributesFactory` | ⬜ |
| `AotTestAttributesCodeGenerator` | 代码生成器 | `AotTestAttributesCodeGenerator` | ⬜ |
| `AotTestContextInitializers` | AOT 上下文初始化器 | `AotTestContextInitializers` | ⬜ |
| `AotTestContextInitializersFactory` | 工厂 | `AotTestContextInitializersFactory` | ⬜ |
| `AotTestContextInitializersCodeGenerator` | 代码生成器 | `AotTestContextInitializersCodeGenerator` | ⬜ |
| `AotMergedContextConfiguration` | AOT 版 MergedContextConfiguration | `AotMergedContextConfiguration` | ⬜ |

### 1.5 BeanOverride（Bean 替换）

| Java 接口 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `BeanOverride` | 标记一个对象是 BeanOverride | `BeanOverride` trait | ⬜ |
| `BeanOverrideStrategy` | 替换策略（REPLACE / WRAPPER） | `BeanOverrideStrategy` enum | ⬜ |
| `BeanOverrideHandler` | 处理单个 override | `BeanOverrideHandler` trait | ⬜ |
| `BeanOverrideProcessor` | 处理 @BeanOverride 字段 | `BeanOverrideProcessor` trait | ⬜ |
| `BeanOverrideReflectiveProcessor` | 基于反射的实现 | `BeanOverrideReflectiveProcessor` | ⬜ |
| `BeanOverrideContextCustomizer` | 把 override 应用到容器 | `BeanOverrideContextCustomizer` | ⬜ |
| `BeanOverrideContextCustomizerFactory` | 工厂 | `BeanOverrideContextCustomizerFactory` | ⬜ |
| `BeanOverrideBeanFactoryPostProcessor` | 在容器启动前注册 override | `BeanOverrideBeanFactoryPostProcessor` | ⬜ |
| `BeanOverrideRegistry` | override 注册表 | `BeanOverrideRegistry` | ⬜ |

### 1.6 Hint（提示系统）

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `TestContextHint` | 提示当前 TestContext 支持的能力 | `TestContextHint` | ⬜ |
| `Search` | 搜索支持 hint | `Search` | ⬜ |

---

## 二、TestExecutionListener 体系

### 2.1 trait 与回调事件

| Java 接口/类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `TestExecutionListener` | 测试执行监听器（7 个回调） | `TestExecutionListener` trait | ⬜ |
| `TestExecutionListeners` | @TestExecutionListeners 注解 + 合并模式 | `TestExecutionListeners` 宏 + `MergeMode` enum | ⬜ |
| `MergeMode` | 合并模式（ALL / MERGE_WITH_DEFAULTS） | `MergeMode` enum | ⬜ |
| `BeforeTestClassEvent` | 测试类开始前事件 | `BeforeTestClassEvent` | ⬜ |
| `AfterTestClassEvent` | 测试类结束后事件 | `AfterTestClassEvent` | ⬜ |
| `BeforeTestMethodEvent` | 测试方法开始前事件 | `BeforeTestMethodEvent` | ⬜ |
| `AfterTestMethodEvent` | 测试方法结束后事件 | `AfterTestMethodEvent` | ⬜ |
| `BeforeTestExecutionEvent` | 测试方法执行前事件 | `BeforeTestExecutionEvent` | ⬜ |
| `AfterTestExecutionEvent` | 测试方法执行后事件 | `AfterTestExecutionEvent` | ⬜ |

### 2.2 内置 Listener 实现

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `TransactionalTestExecutionListener` | 事务管理（@Transactional + @Commit + @Rollback） | `TransactionalTestExecutionListener` | ⬜ |
| `SqlScriptsTestExecutionListener` | @Sql 脚本执行 | `SqlScriptsTestExecutionListener` | ⬜ |
| `SqlScriptsRegistrar` | SQL 注册器 | `SqlScriptsRegistrar` | ⬜ |
| `DependencyInjectionTestExecutionListener` | 依赖注入到测试实例 | `DependencyInjectionTestExecutionListener` | ⬜ |
| `AbstractDirtiesContextTestExecutionListener` | @DirtiesContext 处理骨架 | `AbstractDirtiesContextTestExecutionListener` | ⬜ |
| `DirtiesContextTestExecutionListener` | 标记脏上下文（类/方法执行后） | `DirtiesContextTestExecutionListener` | ⬜ |
| `EventPublishingTestExecutionListener` | ApplicationEvent 桥接 | `EventPublishingTestExecutionListener` | ⬜ |
| `ApplicationEventsTestExecutionListener` | 捕获测试期间发布的事件 | `ApplicationEventsTestExecutionListener` | ⬜ |
| `ApplicationEventsHolder` | 事件持有者 | `ApplicationEventsHolder` | ⬜ |
| `ApplicationEventsApplicationListener` | 事件监听器包装 | `ApplicationEventsApplicationListener` | ⬜ |
| `BeanOverrideTestExecutionListener` | 处理 BeanOverride | `BeanOverrideTestExecutionListener` | ⬜ |
| `AotTestExecutionListener` | AOT 测试处理 | `AotTestExecutionListener` | ⬜ |
| `CommonCachesTestExecutionListener` | 清空缓存 | `CommonCachesTestExecutionListener` | ⬜ |

---

## 三、注解（annotation/）

### 3.1 上下文配置注解

| Java 注解 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `@ContextConfiguration` | 指定配置类/资源位置 | `#[ContextConfiguration(...)]` 宏 | ⬜ |
| `@ContextHierarchy` | 多层上下文 | `#[ContextHierarchy(...)]` 宏 | ⬜ |
| `@ActiveProfiles` | 激活 profile | `#[ActiveProfiles(...)]` 宏 | ⬜ |
| `ActiveProfilesResolver` | 动态解析 profile | `ActiveProfilesResolver` trait | ⬜ |
| `ActiveProfilesUtils` | 工具 | `ActiveProfilesUtils` | ⬜ |

### 3.2 属性源注解

| Java 注解 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `@TestPropertySource` | 测试属性源 | `#[TestPropertySource(...)]` 宏 | ⬜ |
| `@TestPropertySources` | 多个属性源 | `#[TestPropertySources(...)]` 宏 | ⬜ |
| `@DynamicPropertySource` | 动态属性源（函数式） | `#[DynamicPropertySource]` 宏 | ⬜ |

### 3.3 缓存控制注解

| Java 注解 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `@DirtiesContext` | 标记脏上下文 | `#[DirtiesContext]` 宏 | ⬜ |
| `HierarchyMode` | 脏上下文层级模式 | `HierarchyMode` enum | ⬜ |

### 3.4 事务注解

| Java 注解 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `@Transactional` | 事务方法（来自 spring-tx） | `#[Transactional]` 宏（来自 vernal-tx） | 🔶 复用 vernal-tx |
| `@Commit` | 测试后提交事务（覆盖默认 rollback） | `#[Commit]` 宏 | ⬜ |
| `@Rollback` | 测试后回滚事务 | `#[Rollback(false)]` 宏 | ⬜ |
| `@BeforeTransaction` | 事务前回调 | `#[BeforeTransaction]` 宏 | ⬜ |
| `@AfterTransaction` | 事务后回调 | `#[AfterTransaction]` 宏 | ⬜ |

### 3.5 SQL 注解

| Java 注解 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `@Sql` | 执行 SQL 脚本 | `#[Sql(...)]` 宏 | ⬜ |
| `@SqlGroup` | 多 SQL 分组 | `#[SqlGroup(...)]` 宏 | ⬜ |

### 3.6 测试元注解

| Java 注解 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `@Timed` | 限时测试（超时断言） | `#[Timed(millis = ...)]` 宏 | ⬜ |
| `@Repeat` | 重复测试 | `#[Repeat(n)]` 宏 | ⬜ |
| `@ProfileValueSourceConfiguration` | profile 值源 | `#[ProfileValueSourceConfiguration(...)]` 宏 | ⬜ |
| `@IfProfileValue` | 条件执行 | `#[IfProfileValue(...)]` 宏 | ⬜ |
| `@ExpectedException` | 期望异常（已 deprecated） | 不迁移 | 🚫 |

---

## 四、事务测试（transaction/）

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `TestTransaction` | 测试事务管理（start / end / isActive / flagForCommit / flagForRollback） | `TestTransaction` | ⬜ |
| `TransactionAssert` | 事务状态断言 | `TransactionAssert` | ⬜ |
| `TransactionalTestUtil` | 事务测试工具 | `TransactionalTestUtil` | ⬜ |

---

## 五、JDBC 测试（jdbc/）

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `JdbcTestUtils` | JDBC 通用测试工具（countRows / deleteFromTables） | `JdbcTestUtils` | ⬜ |
| `SimpleJdbcTestUtils` | 简单 JDBC 测试工具 | `SimpleJdbcTestUtils` | ⬜ |

---

## 六、Web 测试（web/）

### 6.1 MockMvc

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockMvc` | 执行请求 + 返回结果的执行器 | `MockMvc` | ⬜ |
| `MockMvcBuilder` | Builder | `MockMvcBuilder` trait | ⬜ |
| `ConfigurableMockMvcBuilder` | 可配置 Builder | `ConfigurableMockMvcBuilder` trait | ⬜ |
| `StandaloneMockMvcBuilder` | 单 Controller Builder | `StandaloneMockMvcBuilder` | ⬜ |
| `DefaultMockMvcBuilder` | 默认 Builder | `DefaultMockMvcBuilder` | ⬜ |
| `MockMvcBuilders` | 工厂方法 | `MockMvcBuilders` | ⬜ |
| `ResultActions` | 结果 + 断言链 | `ResultActions` | ⬜ |
| `DefaultResultActions` | 默认实现 | `DefaultResultActions` | ⬜ |
| `MockMvcResultHandlers` | 结果处理器（print） | `MockMvcResultHandlers` | ⬜ |
| `PrintMockMvcResultHandler` | print 实现 | `PrintMockMvcResultHandler` | ⬜ |
| `MockHttpServletRequestBuilder` | 请求构造器 | `MockHttpServletRequestBuilder` | ⬜ |
| `MockMultipartHttpServletRequestBuilder` | multipart 请求构造器 | `MockMultipartHttpServletRequestBuilder` | ⬜ |

### 6.2 Result Matchers（断言匹配器）

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `StatusResultMatchers` | 状态码断言（isOk / isNotFound 等） | `StatusResultMatchers` | ⬜ |
| `HeaderResultMatchers` | 响应头断言 | `HeaderResultMatchers` | ⬜ |
| `CookieResultMatchers` | Cookie 断言 | `CookieResultMatchers` | ⬜ |
| `ContentResultMatchers` | 响应体断言 | `ContentResultMatchers` | ⬜ |
| `JsonPathResultMatchers` | JSON Path 断言 | `JsonPathResultMatchers`（依赖 `serde_json` / `jsonpath_lib`） | 🔶 复用 jsonpath-rust |
| `XpathResultMatchers` | XPath 断言 | `XpathResultMatchers`（依赖 `sxd-xpath`） | 🔶 复用 sxd-xpath |
| `ModelResultMatchers` | Model 断言 | `ModelResultMatchers` | ⬜ |
| `ViewResultMatchers` | View 断言 | `ViewResultMatchers` | ⬜ |
| `FlashAttributeResultMatchers` | Flash Attribute 断言 | `FlashAttributeResultMatchers` | ⬜ |
| `RequestResultMatchers` | 请求侧断言 | `RequestResultMatchers` | ⬜ |

### 6.3 WebTestClient（Reactive）

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `WebTestClient` | 响应式 Web 测试客户端 | `WebTestClient` | ⬜ |
| `WebTestClientBuilder` | Builder | `WebTestClientBuilder` | ⬜ |

### 6.4 RestTestClient

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `RestTestClient` | REST 客户端测试 | `RestTestClient` | ⬜ |

### 6.5 Kotlin DSL（移植为 Rust builder API）

| Kotlin 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockMvcExtensions.kt` | MockMvc Kotlin DSL | `MockMvcExtensions` | ⬜ |
| `MockMvcResultMatchersDsl.kt` | ResultMatchers DSL | `MockMvcResultMatchersDsl` | ⬜ |
| `StatusResultMatchersDsl.kt` | Status DSL | `StatusResultMatchersDsl` | ⬜ |
| ... | ... | ... | ⬜ |

> 注：Rust 中以 builder + 方法链 + 宏组合实现等价 DSL，避免单独文件。

---

## 七、Mock 对象（mock/）

### 7.1 mock.env

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockEnvironment` | 模拟 Spring Environment | `MockEnvironment` | ⬜ |
| `MockPropertySource` | 模拟 PropertySource | `MockPropertySource` | ⬜ |
| `MockPropertyResolver` | 模拟 PropertyResolver | `MockPropertyResolver` | ⬜ |
| `MockBeanFactory` | 模拟 BeanFactory | `MockBeanFactory` | ⬜ |
| `MockPrototypeTargetSource` | 模拟原型 TargetSource | `MockPrototypeTargetSource` | ⬜ |

### 7.2 mock.http

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockHttpServletRequest` | 模拟 HttpServletRequest | `MockHttpServletRequest` | ⬜ |
| `MockHttpServletResponse` | 模拟 HttpServletResponse | `MockHttpServletResponse` | ⬜ |
| `MockHttpSession` | 模拟 HttpSession | `MockHttpSession` | ⬜ |
| `MockMultipartFile` | 模拟 MultipartFile | `MockMultipartFile` | ⬜ |
| `MockMultipartHttpServletRequest` | 模拟 Multipart 请求 | `MockMultipartHttpServletRequest` | ⬜ |
| `MockPart` | 模拟 Part | `MockPart` | ⬜ |
| `MockHttpOutputMessage` | 模拟 HTTP 输出 | `MockHttpOutputMessage` | ⬜ |
| `MockHttpInputMessage` | 模拟 HTTP 输入 | `MockHttpInputMessage` | ⬜ |

### 7.3 mock.http.client

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockClientHttpRequest` | 模拟客户端请求 | `MockClientHttpRequest` | ⬜ |
| `MockClientHttpResponse` | 模拟客户端响应 | `MockClientHttpResponse` | ⬜ |
| `MockClientHttpRequestFactory` | 模拟请求工厂 | `MockClientHttpRequestFactory` | ⬜ |
| `MockAsyncClientHttpRequest` | 模拟异步请求 | `MockAsyncClientHttpRequest` | ⬜ |

### 7.4 mock.http.server

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockServerHttpRequest` | 模拟服务端请求 | `MockServerHttpRequest` | ⬜ |
| `MockServerHttpResponse` | 模拟服务端响应 | `MockServerHttpResponse` | ⬜ |
| `MockReactiveServerHttpRequest` | 模拟响应式服务端请求 | `MockReactiveServerHttpRequest` | ⬜ |
| `MockServerWebExchange` | 模拟 WebExchange | `MockServerWebExchange` | ⬜ |

### 7.5 mock.web

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockServletContext` | 模拟 ServletContext | `MockServletContext` | ⬜ |
| `MockRequestDispatcher` | 模拟 RequestDispatcher | `MockRequestDispatcher` | ⬜ |
| `MockServletConfig` | 模拟 ServletConfig | `MockServletConfig` | ⬜ |
| `MockFilterChain` | 模拟 FilterChain | `MockFilterChain` | ⬜ |
| `MockFilterConfig` | 模拟 FilterConfig | `MockFilterConfig` | ⬜ |
| `MockAsyncContext` | 模拟 AsyncContext | `MockAsyncContext` | ⬜ |
| `MockDispatcherType` | 模拟 DispatcherType | `MockDispatcherType` | ⬜ |
| `MockErrorPageRegister` | 模拟错误页注册 | `MockErrorPageRegister` | ⬜ |
| `MockPageContext` | 模拟 PageContext | `MockPageContext` | ⬜ |
| `MockServletContextListener` | 模拟 ServletContextListener | `MockServletContextListener` | ⬜ |

### 7.6 mock.web.reactive

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockServerHttpRequest` | 响应式版 | `MockServerHttpRequest` | ⬜ |
| `MockServerHttpResponse` | 响应式版 | `MockServerHttpResponse` | ⬜ |
| `MockServerWebExchange` | 响应式版 | `MockServerWebExchange` | ⬜ |
| `MockServerCookie` | 响应式 Cookie | `MockServerCookie` | ⬜ |

### 7.7 mock.web.server

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MockWebServiceConnection` | 模拟 WebService 连接 | `MockWebServiceConnection` | ⬜ |
| `MockWebServiceMessage` | 模拟 WebService 消息 | `MockWebServiceMessage` | ⬜ |
| `MockWebServiceMessageFactory` | 模拟工厂 | `MockWebServiceMessageFactory` | ⬜ |

---

## 八、HTTP / JSON / Validation 断言

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `MediaTypeAssert` | MediaType 断言 | `MediaTypeAssert` | ⬜ |
| `HeaderAssertions` | Header 断言 | `HeaderAssertions` | ⬜ |
| `CookieAssertions` | Cookie 断言（test/http pkg） | `CookieAssertions` | ⬜ |
| `ContentRequestMatchers` | 请求体匹配器 | `ContentRequestMatchers` | ⬜ |
| `ContentResultMatchers` | 响应体匹配器 | `ContentResultMatchers` | ⬜ |
| `JsonPathExpectationsHelper` | JSON Path 工具 | `JsonPathExpectationsHelper` | ⬜ |
| `JsonPathAssert` | JSON Path 断言 | `JsonPathAssert` | ⬜ |
| `JsonValueAssert` | JSON 值断言 | `JsonValueAssert` | ⬜ |
| `BindingResultAssert` | BindingResult 断言 | `BindingResultAssert` | ⬜ |

---

## 九、工具类（util/）

| Java 类 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `ReflectionTestUtils` | 反射工具（访问私有字段、调用私有方法） | `ReflectionTestUtils` | ⬜ |
| `AopTestUtils` | AOP 工具（解包代理对象为目标对象） | `AopTestUtils` | ⬜ |
| `AssertionErrors` | 断言错误工具 | `AssertionErrors` | ⬜ |
| `AnnotationTestUtils` | 注解测试工具 | `AnnotationTestUtils` | ⬜ |

---

## 十、过程宏（vernal-test-macros）

| 宏 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `#[vernal_test]` | 标记测试函数，自动加载上下文 + 注入依赖 | `#[vernal_test]` 宏 | ⬜ |
| `#[with_context(...)]` | 指定测试上下文 | `#[with_context(...)]` 宏 | ⬜ |
| `#[mock_bean(...)]` | 注入 Mock Bean | `#[mock_bean(...)]` 宏 | ⬜ |
| `#[spy_bean(...)]` | 注入 Spy Bean | `#[spy_bean(...)]` 宏 | ⬜ |
| `#[derive(SpringTestAttribute)]` | 测试属性派生 | 派生宏 | ⬜ |

---

## 十一、JUnit 集成层（junit/）

| Java 类/注解 | 语义 | Rust 实现 | 状态 |
|---|---|---|---|
| `SpringExtension` | JUnit Jupiter Extension | `VernalExtension` trait | ⬜ |
| `@SpringJUnitConfig` | 组合 @ExtendWith(SpringExtension) + @ContextConfiguration | `#[VernalTest(config = ...)]` 宏 | ⬜ |
| `SpringRunner` | JUnit 4 Runner | 不迁移（JUnit 4 已停更） | 🚫 |
| `@WebMvcTest` | Web MVC 测试切片 | `#[WebMvcTest(...)]` 宏 | ⬜ |
| `@SpringBootTest` | 启动完整应用 | 依赖 `vernal-boot` 提供 | 🔶 复用 vernal-boot |
| `@JdbcTest` | JDBC 测试切片 | `#[JdbcTest(...)]` 宏 | ⬜ |
| `@DataJpaTest` | JPA 测试切片 | 依赖 `vernal-data` | 🔶 复用 vernal-data |
| `@JsonTest` | JSON 测试 | `#[JsonTest(...)]` 宏 | ⬜ |

---

## 十二、底层依赖（Rust 生态复用）

| 语义层 | Rust crate | Spring 对应 |
|--------|-----------|------------|
| 测试上下文（setup/teardown 共享） | [`test-context`](https://crates.io/crates/test-context) | TestContext（轻量子集） |
| Mock 生成 | [`mockall`](https://crates.io/crates/mockall) | Mockito 集成 |
| 参数化测试 | [`rstest`](https://crates.io/crates/rstest) | @ParameterizedTest |
| 用例表驱动 | [`test-case`](https://crates.io/crates/test-case) | @ValueSource/@CsvSource |
| HTTP 服务端 Mock | [`wiremock`](https://crates.io/crates/wiremock) / [`mockito`](https://crates.io/crates/mockito) | MockRestServiceServer |
| Web 端到端 | [`axum-test`](https://crates.io/crates/axum-test) | WebTestClient |
| 异步测试 | [`tokio-test`](https://crates.io/crates/tokio-test) | @Async 测试 |
| 属性化测试 | [`proptest`](https://crates.io/crates/proptest) | jqwik 集成（可选） |
| 断言增强 | [`pretty_assertions`](https://crates.io/crates/pretty_assertions) | AssertJ |
| 缓存 | [`moka`](https://crates.io/crates/moka) | DefaultContextCache |
| JSON Path | [`jsonpath-rust`](https://crates.io/crates/jsonpath-rust) | JsonPathExpectationsHelper |
| XPath | [`sxd-xpath`](https://crates.io/crates/sxd-xpath) | XpathResultMatchers |
| AOP 解包 | `vernal-aop` | AopTestUtils |

> 上面这些底层能力 vernal-test 不重新实现，而是通过 `[dev-dependencies]` 或
> `#[test]`/`#[tokio::test]` + trait 默认方法桥接。

---

## 十三、明确不迁移

| Java 类 | 不迁移原因 |
|--------|----------|
| `org.springframework.test.context.junit4.*` 全部 | JUnit 4 已停更 |
| `org.springframework.test.context.testng.*` 全部 | TestNG 不在范围 |
| `@EnabledIfSystemProperty` / `@EnabledIfEnvironmentVariable` / `@EnabledIf` | 用 `#[cfg(...)]` 或 `test-context` 的条件执行 |
| `org.springframework.test.context.web.WebDelegatingSmartContextLoader` 等 | 7.0 deprecated |
| `@Ignore` | 用 `#[ignore]`（libtest 内置） |
| `@ExpectedException` | 7.0 deprecated，用 `#[should_panic]` |
| `ApplicationContextRunner`（来自 spring-test）| 改用 `vernal-context` 的 `ContainerRunner` |

---

## 十四、统计

| 模块 | Java 类/注解 | 已迁移 | 语义对齐 | 未迁移 | 不迁移 | 对标度 |
|------|------------|------|--------|------|------|------|
| TestContext 体系 | 80 | 0 | 1 | 79 | 0 | 1% |
| ContextLoader 体系 | 13 | 0 | 0 | 13 | 0 | 0% |
| ContextCache 体系 | 4 | 0 | 0 | 4 | 0 | 0% |
| AOT | 7 | 0 | 0 | 7 | 0 | 0% |
| BeanOverride | 9 | 0 | 0 | 9 | 0 | 0% |
| Hint | 2 | 0 | 0 | 2 | 0 | 0% |
| TestExecutionListener 体系 | 22 | 0 | 0 | 22 | 0 | 0% |
| 注解 | 24 | 0 | 1 | 23 | 1 | 5% |
| 事务测试 | 4 | 0 | 0 | 4 | 0 | 0% |
| JDBC 测试 | 3 | 0 | 0 | 3 | 0 | 0% |
| Web Servlet | ~40 | 0 | 0 | 40 | 0 | 0% |
| Web Reactive | 3 | 0 | 0 | 3 | 0 | 0% |
| Web Client | 1 | 0 | 0 | 1 | 0 | 0% |
| Kotlin DSL | 18 | 0 | 0 | 18 | 0 | 0% |
| Mock 对象 | ~50 | 0 | 0 | 50 | 0 | 0% |
| HTTP/JSON/Validation | ~10 | 0 | 0 | 10 | 0 | 0% |
| Util | 4 | 0 | 0 | 4 | 0 | 0% |
| JUnit 集成 | ~8 | 0 | 2 | 5 | 1 | 25% |
| **合计** | **~302** | **0** | **4** | **297** | **2** | **1.3%** |

> 当前 vernal-test 仅有 `TestContext` 骨架（语义部分对齐）+ 复用 vernal-tx 的
> `@Transactional` + vernal-boot 的 `@SpringBootTest`。完整迁移路线图见
> `spring-test到vernal-test迁移路线图.md`。
