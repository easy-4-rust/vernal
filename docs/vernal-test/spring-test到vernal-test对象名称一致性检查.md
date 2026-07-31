<!-- migration-doc: authority=historical canonical=对象名称一致性检查.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [对象名称一致性检查.md](对象名称一致性检查.md) 为准。

> **历史文档，非当前验收依据。** 当前版本见[对象名称一致性检查](对象名称一致性检查.md)。

# vernal-test 与 spring-test 对象名称一致性检查

> 检查时间：2026-07-27
> 基线：Spring Framework 7.0.8 spring-test（459 主 Java 类 + 22 Kotlin DSL + ~80 mock 类，合计 ~480 个公开对象）
> vernal-test：当前 **2 文件 / 25 行 / 1 个公开对象**（`TestContext` 骨架）

## 统计汇总

| 维度 | Spring | vernal | 说明 |
|------|--------|--------|------|
| Java 类总数 | ~480 | — | 剔除 package-info / 内部类 |
| Rust 文件总数 | — | 2 | 当前 `lib.rs` + `context.rs` |
| **已迁移对象** | 1 | 1 | `TestContext`（部分语义）|
| Spring 有 vernal 没有 | ~478 | — | 见下文分类 |
| vernal 有 Spring 没有 | — | 0 | — |
| **完全匹配（类型名一致）** | **1** | **1** | `TestContext` |
| **类型名一致但语义缩窄** | **0** | **0** | — |
| **类型名重命名（Rust 命名规范）** | 0 | 0 | — |
| **类型名合并（多个 Java 类 → 1 Rust）** | 0 | 0 | — |
| **类型名拆分（1 Java 类 → 多个 Rust）** | 0 | 0 | — |

---

## 一、已迁移且类型名一致的对象（1 个）

| Spring Java 类型 | vernal Rust 文件 | 状态 | 说明 |
|----------------|-----------------|------|------|
| `org.springframework.test.context.TestContext` | `crates/vernal-test/src/context.rs::TestContext` | 🔶 部分语义 | 当前仅 `name`，缺失 `test_class` / `test_method` / `application_context` / `mark_dirty` 等核心方法 |

> 注：vernal-test 当前实现是骨架，语义部分对齐（仅保留命名）。完成 S1 后才能视为完全匹配。

---

## 二、Spring 有但 vernal 没有的对象（按包分类）

### 2.1 `org.springframework.test.context`（核心接口）

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `TestContextManager` | ✅ PascalCase | `context/test_context_manager.rs` | 完全一致 |
| `TestContextBootstrapper` | ✅ PascalCase | `context/test_context_bootstrapper.rs` | 完全一致 |
| `BootstrapContext` | ✅ PascalCase | `context/bootstrap_context.rs` | 完全一致 |
| `BootstrapUtils` | ✅ PascalCase | `context/bootstrap_utils.rs` | 完全一致 |
| `BootstrapWith` | ✅ PascalCase | `context/bootstrap_with.rs` | 完全一致 |
| `AttributeAccessor` | ✅ PascalCase | `context/attribute_accessor.rs` | 完全一致 |
| `TestContextAnnotationUtils` | ✅ PascalCase | `context/test_context_annotation_utils.rs` | 完全一致 |
| `MethodInvoker` | ✅ PascalCase | `context/method_invoker.rs` | 完全一致 |
| `DefaultMethodInvoker` | ✅ PascalCase | `context/method_invoker.rs`（合并）| 类型名合并到 `method_invoker.rs` |
| `TestConstructor` | ✅ PascalCase | `context/test_constructor.rs` | 完全一致 |
| `NestedTestConfiguration` | ✅ PascalCase | `context/nested_test_configuration.rs` | 完全一致 |
| `ApplicationContextFailureProcessor` | ✅ PascalCase | `context/application_context_failure_processor.rs` | 完全一致 |
| `ContextLoadException` | ✅ PascalCase | `context/context_load_exception.rs` | 完全一致 |
| `DynamicPropertySource` | ✅ PascalCase | `context/dynamic_property_source.rs` | 完全一致 |
| `DynamicProperty` | ✅ PascalCase | `context/dynamic_property.rs` | 完全一致 |
| `DynamicPropertyRegistry` | ✅ PascalCase | `context/dynamic_property.rs`（合并）| 类型名合并 |
| `DynamicPropertyRegistrar` | ✅ PascalCase | `context/dynamic_property.rs`（合并）| 类型名合并 |
| `MergedContextConfiguration` | ✅ PascalCase | `context/merged_context_configuration.rs` | 完全一致 |
| `ContextConfigurationAttributes` | ✅ PascalCase | `context/context_configuration_attributes.rs` | 完全一致 |
| `ContextHierarchy` | ✅ PascalCase | `context/context_hierarchy.rs` | 完全一致 |
| `ContextCustomizer` | ✅ PascalCase | `context/context_customizer.rs` | 完全一致 |
| `ContextCustomizerFactory` | ✅ PascalCase | `context/context_customizer_factory.rs` | 完全一致 |
| `ContextCustomizerFactories` | ✅ PascalCase（复数）| `context/context_customizer_factories.rs` | 完全一致 |

### 2.2 `org.springframework.test.context.event`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `BeforeTestClassEvent` | ✅ PascalCase | `context/event/before_test_class_event.rs` | 完全一致 |
| `AfterTestClassEvent` | ✅ PascalCase | `context/event/after_test_class_event.rs` | 完全一致 |
| `BeforeTestMethodEvent` | ✅ PascalCase | `context/event/before_test_method_event.rs` | 完全一致 |
| `AfterTestMethodEvent` | ✅ PascalCase | `context/event/after_test_method_event.rs` | 完全一致 |
| `BeforeTestExecutionEvent` | ✅ PascalCase | `context/event/before_test_execution_event.rs` | 完全一致 |
| `AfterTestExecutionEvent` | ✅ PascalCase | `context/event/after_test_execution_event.rs` | 完全一致 |

### 2.3 `org.springframework.test.context.hint`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `TestContextHint` | ✅ PascalCase | `context/hint/test_context_hint.rs` | 完全一致 |
| `Search` | ✅ PascalCase | `context/hint/search_hints.rs` | 建议 `Search` → `SearchHints`（复数与文件对齐）|

### 2.4 `org.springframework.test.context.support`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `AnnotationContextLoaderUtils` | ✅ PascalCase | `context/support/annotation_context_loader_utils.rs` | 完全一致 |
| `DefaultTestContextBootstrapper` | ✅ PascalCase | `context/support/default_test_context_bootstrapper.rs` | 完全一致 |

### 2.5 `org.springframework.test.context.util`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `TestContextPathUtils` | ✅ PascalCase | `context/util/test_context_path_utils.rs` | 完全一致 |

### 2.6 `org.springframework.test.context.cache`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `ContextCache` | ✅ PascalCase | `context/cache/context_cache.rs` | 完全一致 |
| `DefaultContextCache` | ✅ PascalCase | `context/cache/default_context_cache.rs` | 完全一致 |
| `CacheAwareContextLoaderDelegate` | ✅ PascalCase | `context/cache/cache_aware_context_loader_delegate.rs` | 完全一致 |
| `ContextCacheUtils` | ✅ PascalCase | `context/cache/context_cache_utils.rs` | 完全一致 |

### 2.7 `org.springframework.test.context.aot`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `AotTestAttributes` | ✅ PascalCase | `context/aot/aot_test_attributes.rs` | 完全一致 |
| `AotTestAttributesFactory` | ✅ PascalCase | `context/aot/aot_test_attributes_factory.rs` | 完全一致 |
| `AotTestAttributesCodeGenerator` | ✅ PascalCase | `context/aot/aot_test_attributes_code_generator.rs` | 完全一致 |
| `AotTestContextInitializers` | ✅ PascalCase | `context/aot/aot_test_context_initializers.rs` | 完全一致 |
| `AotTestContextInitializersFactory` | ✅ PascalCase | `context/aot/aot_test_context_initializers_factory.rs` | 完全一致 |
| `AotTestContextInitializersCodeGenerator` | ✅ PascalCase | `context/aot/aot_test_context_initializers_code_generator.rs` | 完全一致 |
| `AotMergedContextConfiguration` | ✅ PascalCase | `context/aot/aot_merged_context_configuration.rs` | 完全一致 |

### 2.8 `org.springframework.test.context.bean`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `BeanOverride` | ✅ PascalCase | `context/bean/bean_override.rs` | 完全一致 |
| `BeanOverrideStrategy` | ✅ PascalCase | `context/bean/bean_override_strategy.rs` | 完全一致 |
| `BeanOverrideHandler` | ✅ PascalCase | `context/bean/bean_override_handler.rs` | 完全一致 |
| `BeanOverrideProcessor` | ✅ PascalCase | `context/bean/bean_override_processor.rs` | 完全一致 |
| `BeanOverrideReflectiveProcessor` | ✅ PascalCase | `context/bean/bean_override_reflective_processor.rs` | 完全一致 |
| `BeanOverrideContextCustomizer` | ✅ PascalCase | `context/bean/bean_override_context_customizer.rs` | 完全一致 |
| `BeanOverrideContextCustomizerFactory` | ✅ PascalCase | `context/bean/bean_override_context_customizer_factory.rs` | 完全一致 |
| `BeanOverrideBeanFactoryPostProcessor` | ✅ PascalCase | `context/bean/bean_override_bean_factory_post_processor.rs` | 完全一致 |
| `BeanOverrideRegistry` | ✅ PascalCase | `context/bean/bean_override_registry.rs` | 完全一致 |

### 2.9 `org.springframework.test.context.jdbc`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `JdbcTestUtils` | ✅ PascalCase | `jdbc/jdbc_test_utils.rs` | 完全一致 |
| `SimpleJdbcTestUtils` | ✅ PascalCase | `jdbc/simple_jdbc_test_utils.rs` | 完全一致 |

### 2.10 `org.springframework.test.context.junit` / `junit4` / `junit5`

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `SpringJUnitConfig` | ✅ PascalCase | `junit/spring_vernal_config.rs` | **建议重命名**：`Spring` → `Vernal` |
| `WebMvcTest` | ✅ PascalCase | `junit/web_vernal_config.rs` | **建议重命名**：去掉品牌名 |
| `SpringBootTest` | ✅ PascalCase | （依赖 vernal-boot 提供） | **建议重命名**：`VernalBootTest` |
| `SpringExtension` | ✅ PascalCase | `junit/vernal_extension.rs` | **建议重命名**：`VernalExtension` |
| `SpringRunner` | ✅ PascalCase | — | 🚫 不迁移（JUnit 4 停更）|

### 2.11 ContextLoader 子包（`org.springframework.test.context.support`）

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `ContextLoader` | ✅ PascalCase | `context/loader/context_loader.rs` | 完全一致 |
| `SmartContextLoader` | ✅ PascalCase | `context/loader/smart_context_loader.rs` | 完全一致 |
| `AbstractContextLoader` | ✅ PascalCase | `context/loader/abstract_context_loader.rs` | 完全一致 |
| `AbstractGenericContextLoader` | ✅ PascalCase | `context/loader/abstract_generic_context_loader.rs` | 完全一致 |
| `AbstractGenericWebContextLoader` | ✅ PascalCase | `context/loader/abstract_generic_web_context_loader.rs` | 完全一致 |
| `AbstractDelegatingSmartContextLoader` | ✅ PascalCase | `context/loader/abstract_delegating_smart_context_loader.rs` | 完全一致 |
| `AnnotationConfigContextLoader` | ✅ PascalCase | `context/loader/annotation_config_context_loader.rs` | 完全一致 |
| `AnnotationConfigContextLoaderUtils` | ✅ PascalCase | `context/loader/annotation_config_context_loader_utils.rs` | 完全一致 |
| `AnnotationConfigWebContextLoader` | ✅ PascalCase | `context/loader/annotation_config_web_context_loader.rs` | 完全一致 |
| `GenericContextLoader` | ✅ PascalCase | `context/loader/generic_context_loader.rs` | 完全一致 |
| `GenericXmlContextLoader` | ✅ PascalCase | `context/loader/generic_xml_context_loader.rs` | **建议加注释**：vernal 中 XML 改用 YAML/JSON |
| `AotContextLoader` | ✅ PascalCase | `context/loader/aot_context_loader.rs` | 完全一致 |
| `ContextLoaderUtils` | ✅ PascalCase | `context/loader/context_loader_utils.rs` | 完全一致 |

### 2.12 TestExecutionListener 体系

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `TestExecutionListener` | ✅ PascalCase | `listener/test_execution_listener.rs` | 完全一致 |
| `TestExecutionListeners` | ✅ PascalCase | `listener/test_execution_listeners.rs` | 完全一致 |
| `TransactionalTestExecutionListener` | ✅ PascalCase | `listener/transactional_test_execution_listener.rs` | 完全一致 |
| `SqlScriptsTestExecutionListener` | ✅ PascalCase | `listener/sql_scripts_test_execution_listener.rs` | 完全一致 |
| `SqlScriptsRegistrar` | ✅ PascalCase | `listener/sql_scripts_registrar.rs` | 完全一致 |
| `DependencyInjectionTestExecutionListener` | ✅ PascalCase | `listener/dependency_injection_test_execution_listener.rs` | 完全一致 |
| `AbstractDirtiesContextTestExecutionListener` | ✅ PascalCase | `listener/dirties_context_test_execution_listener.rs` | 完全一致 |
| `DirtiesContextTestExecutionListener` | ✅ PascalCase | `listener/dirties_context_test_execution_listener.rs`（合并）| 类型名合并 |
| `EventPublishingTestExecutionListener` | ✅ PascalCase | `listener/event_publishing_test_execution_listener.rs` | 完全一致 |
| `ApplicationEventsTestExecutionListener` | ✅ PascalCase | `listener/application_events_test_execution_listener.rs` | 完全一致 |
| `ApplicationEventsHolder` | ✅ PascalCase | `listener/event_publishing_test_execution_listener.rs`（合并）| 类型名合并 |
| `ApplicationEventsApplicationListener` | ✅ PascalCase | `listener/event_publishing_test_execution_listener.rs`（合并）| 类型名合并 |
| `BeanOverrideTestExecutionListener` | ✅ PascalCase | `listener/bean_overriding_test_execution_listener.rs` | **注意**：Rust 文件名 `bean_overriding`（动名词）而非 `bean_override`，类型名仍 `BeanOverrideTestExecutionListener` |
| `AotTestExecutionListener` | ✅ PascalCase | `listener/aot_test_execution_listener.rs` | 完全一致 |
| `CommonCachesTestExecutionListener` | ✅ PascalCase | `listener/common_caches_test_execution_listener.rs` | 完全一致 |

### 2.13 注解（`org.springframework.test.annotation`）

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `ContextConfiguration` | ✅ PascalCase | `annotation/context_configuration.rs` | 完全一致 |
| `ActiveProfiles` | ✅ PascalCase | `annotation/active_profiles.rs` | 完全一致 |
| `ActiveProfilesResolver` | ✅ PascalCase | `annotation/active_profiles_resolver.rs` | 完全一致 |
| `ActiveProfilesUtils` | ✅ PascalCase | `annotation/active_profiles_utils.rs` | 完全一致 |
| `TestPropertySource` | ✅ PascalCase | `annotation/test_property_source.rs` | 完全一致 |
| `TestPropertySources` | ✅ PascalCase | `annotation/test_property_sources.rs` | 完全一致 |
| `DirtiesContext` | ✅ PascalCase | `annotation/dirties_context.rs` | 完全一致 |
| `HierarchyMode` | ✅ PascalCase | `annotation/dirties_context.rs`（合并）| 类型名合并 |
| `Commit` | ✅ PascalCase | `annotation/commit.rs` | **存在冲突**：`commit` 与 Cargo 标准库同名，建议改 `commit_annotation.rs` 或 `vernal_commit.rs` |
| `Rollback` | ✅ PascalCase | `annotation/rollback.rs` | 完全一致 |
| `BeforeTransaction` | ✅ PascalCase | `annotation/before_transaction.rs` | 完全一致 |
| `AfterTransaction` | ✅ PascalCase | `annotation/after_transaction.rs` | 完全一致 |
| `Sql` | ✅ PascalCase | `annotation/sql.rs` | 完全一致 |
| `SqlGroup` | ✅ PascalCase | `annotation/sql_group.rs` | 完全一致 |
| `Timed` | ✅ PascalCase | `annotation/timed.rs` | 完全一致 |
| `Repeat` | ✅ PascalCase | `annotation/repeat.rs` | 完全一致 |
| `ProfileValueSourceConfiguration` | ✅ PascalCase | `annotation/profile_value_source.rs` | 完全一致 |
| `IfProfileValue` | ✅ PascalCase | `annotation/if_profile_value.rs` | 完全一致 |
| `ExpectedException` | ✅ PascalCase | `annotation/expected_exception.rs` | 🚫 不迁移 |

### 2.14 事务测试（`org.springframework.test.context.transaction`）

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `TransactionalTestExecutionListener` | ✅ PascalCase | `transaction/transactional_test_execution_listener.rs` | 完全一致（与 listener/ 同名，做语义对齐）|
| `TestTransaction` | ✅ PascalCase | `transaction/test_transaction.rs` | 完全一致 |
| `TransactionAssert` | ✅ PascalCase | `transaction/transaction_assert.rs` | 完全一致 |
| `TransactionalTestUtil` | ✅ PascalCase | `transaction/transactional_test_util.rs` | 完全一致 |

### 2.15 Web Servlet（`org.springframework.test.web.servlet`）

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `MockMvc` | ✅ PascalCase | `web/servlet/mock_mvc.rs` | 完全一致 |
| `MockMvcBuilder` | ✅ PascalCase | `web/servlet/mock_mvc_builder.rs` | 完全一致 |
| `ConfigurableMockMvcBuilder` | ✅ PascalCase | `web/servlet/mock_mvc_builder.rs`（合并）| 类型名合并 |
| `StandaloneMockMvcBuilder` | ✅ PascalCase | `web/servlet/mock_mvc_builders.rs` | 完全一致 |
| `DefaultMockMvcBuilder` | ✅ PascalCase | `web/servlet/mock_mvc_builders.rs`（合并）| 类型名合并 |
| `MockMvcBuilders` | ✅ PascalCase | `web/servlet/mock_mvc_builders.rs`（合并）| 类型名合并 |
| `MockMvcResultHandlers` | ✅ PascalCase | `web/servlet/mock_mvc_result_handlers.rs` | 完全一致 |
| `PrintMockMvcResultHandler` | ✅ PascalCase | `web/servlet/mock_mvc_result_handlers.rs`（合并）| 类型名合并 |
| `ResultActions` | ✅ PascalCase | `web/servlet/result_actions.rs` | 完全一致 |
| `DefaultResultActions` | ✅ PascalCase | `web/servlet/result_actions.rs`（合并）| 类型名合并 |
| `MockHttpServletRequestBuilder` | ✅ PascalCase | `web/servlet/request_builder.rs` | 完全一致 |
| `MockMultipartHttpServletRequestBuilder` | ✅ PascalCase | `web/servlet/request_builder.rs`（合并）| 类型名合并 |
| `StatusResultMatchers` | ✅ PascalCase | `web/servlet/result/status_result_matchers.rs` | 完全一致 |
| `HeaderResultMatchers` | ✅ PascalCase | `web/servlet/result/header_result_matchers.rs` | 完全一致 |
| `CookieResultMatchers` | ✅ PascalCase | `web/servlet/result/cookie_result_matchers.rs` | 完全一致 |
| `ContentResultMatchers` | ✅ PascalCase | `web/servlet/result/content_result_matchers.rs` | 完全一致 |
| `JsonPathResultMatchers` | ✅ PascalCase | `web/servlet/result/json_path_result_matchers.rs` | 完全一致 |
| `XpathResultMatchers` | ✅ PascalCase | `web/servlet/result/xpath_result_matchers.rs` | 完全一致 |
| `ModelResultMatchers` | ✅ PascalCase | `web/servlet/result/model_result_matchers.rs` | 完全一致 |
| `ViewResultMatchers` | ✅ PascalCase | `web/servlet/result/view_result_matchers.rs` | 完全一致 |
| `FlashAttributeResultMatchers` | ✅ PascalCase | `web/servlet/result/flash_attribute_result_matchers.rs` | 完全一致 |
| `RequestResultMatchers` | ✅ PascalCase | `web/servlet/result/request_result_matchers.rs` | 完全一致 |
| `MockCookie` | ✅ PascalCase | `web/servlet/cookie/mock_cookie.rs` | 完全一致 |

### 2.16 Web Reactive / Client

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `WebTestClient` | ✅ PascalCase | `web/reactive/web_test_client.rs` | 完全一致 |
| `WebTestClientBuilder` | ✅ PascalCase | `web/reactive/web_test_client_builder.rs` | 完全一致 |
| `RestTestClient` | ✅ PascalCase | `web/client/rest_test_client.rs` | 完全一致 |

### 2.17 Mock 对象（`org.springframework.mock.*`）

全部 ~50 个类 PascalCase，与 Rust 文件 snake_case 一一对应，**类型名完全一致**。

例如：
- `MockEnvironment` → `mock/env/mock_environment.rs`
- `MockHttpServletRequest` → `mock/http/mock_http_servlet_request.rs`
- `MockServletContext` → `mock/web/mock_servlet_context.rs`
- `MockServerWebExchange` → `mock/web/reactive/mock_server_web_exchange.rs`

### 2.18 工具类（`org.springframework.test.util`）

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `ReflectionTestUtils` | ✅ PascalCase | `util/reflection_test_utils.rs` | 完全一致 |
| `AopTestUtils` | ✅ PascalCase | `util/aop_test_utils.rs` | 完全一致 |
| `AssertionErrors` | ✅ PascalCase | `util/assertion_errors.rs` | 完全一致 |
| `AnnotationTestUtils` | ✅ PascalCase | `util/annotation_test_utils.rs` | 完全一致 |

### 2.19 HTTP / JSON / Validation

| Java 类型 | 类型名 | Rust 计划文件名 | 一致性建议 |
|---------|--------|---------------|----------|
| `MediaTypeAssert` | ✅ PascalCase | `http/media_type_assertions.rs` | **建议**：与同名 trait `MockHttpOutputMessage` 等区分，文件名用 `media_type_assertions.rs`（复数）|
| `HeaderAssertions` | ✅ PascalCase | `http/header_assertions.rs` | 完全一致 |
| `CookieAssertions` | ✅ PascalCase | `http/cookie_assertions.rs` | 完全一致 |
| `ContentRequestMatchers` | ✅ PascalCase | `http/request_matcher.rs`（合并）| 类型名合并到 `request_matcher.rs` |
| `ContentResultMatchers` | ✅ PascalCase | `http/content_result_matchers.rs` | 完全一致 |
| `JsonPathExpectationsHelper` | ✅ PascalCase | `json/json_path_assertions.rs`（合并）| 类型名合并 |
| `JsonPathAssert` | ✅ PascalCase | `json/json_path_assert.rs` | 完全一致 |
| `JsonValueAssert` | ✅ PascalCase | `json/json_value_assert.rs` | 完全一致 |
| `BindingResultAssert` | ✅ PascalCase | `validation/binding_result_assert.rs` | 完全一致 |

---

## 三、命名一致但语义缩窄的对象

| Java 类型 | Rust 类型 | 缩窄点 |
|---------|----------|--------|
| `TestContext` | `TestContext` | 当前 Rust 只暴露 `name: String`；Java 有 `test_class` / `test_method` / `application_context` / `mark_dirty` 等 |

> 这部分会在 S1 阶段补全。

---

## 四、需要重命名的对象（Rust 命名规范或冲突）

| Java 类型 | 建议 Rust 命名 | 理由 |
|---------|---------------|------|
| `SpringExtension` | `VernalExtension` | 品牌一致（vernal-*）|
| `SpringRunner` | （不迁移）| JUnit 4 停更 |
| `SpringJUnitConfig` | `VernalTestConfig` 或 `VernalConfig` | 品牌一致 |
| `SpringBootTest` | `VernalBootTest` | 品牌一致 |
| `Search` | `SearchHints` | 与 Rust 文件名 `search_hints.rs` 对齐 |

> 注：vernal-* 项目中一贯不保留 Spring 品牌名（参见 vernal-core 不叫 `spring-core-rs`）。

---

## 五、合并到同一 Rust 文件的对象

为了遵循"一文件一对象"原则，但对**强耦合的小型辅助类**允许合并：

| Java 类型（多个）| Rust 文件（合并）| 合并理由 |
|----------------|---------------|---------|
| `DefaultMethodInvoker` | `method_invoker.rs` | 仅 5 行模板类 |
| `DynamicProperty` + `DynamicPropertyRegistry` + `DynamicPropertyRegistrar` | `dynamic_property.rs` | 三者强耦合 |
| `HierarchyMode` | `dirties_context.rs` | 仅 enum |
| `BeanOverrideTestExecutionListener` | （共享 listener trait 实现）| 与 listener 体系合一 |
| `DirtiesContextTestExecutionListener` | `dirties_context_test_execution_listener.rs` | 与 abstract 合并 |
| `ApplicationEventsHolder` + `ApplicationEventsApplicationListener` | `event_publishing_test_execution_listener.rs` | 内部支撑类 |
| `MockMvcBuilders` + `DefaultMockMvcBuilder` + `StandaloneMockMvcBuilder` | `mock_mvc_builders.rs` | 工厂 + 默认实现 |
| `MockMvcResultHandlers` + `PrintMockMvcResultHandler` | `mock_mvc_result_handlers.rs` | 工厂 + 默认实现 |
| `ResultActions` + `DefaultResultActions` | `result_actions.rs` | trait + 默认实现 |
| `RequestResultMatchers` + `ContentRequestMatchers` | `request_matcher.rs` | 紧密相关 |
| `JsonPathExpectationsHelper` + `JsonPathAssert` | `json/json_path_assert.rs` | 内部支撑 |

---

## 六、vernal 特有（Spring 没有）的对象

当前 **0 个** vernal 特有对象。

未来计划新增：
- `VernalExtension`（替代 `SpringExtension`）
- `vernal_test!` / `with_context!` 宏
- `VernalBootTest`（替代 `SpringBootTest`）
- `ContainerRunner`（替代 `ApplicationContextRunner`）
- `vernal-context` 桥接的 `VernalTestContext` 实现

---

## 七、一致性总结

| 类别 | 数量 | 占比 |
|------|------|------|
| **完全匹配（PascalCase ↔ snake_case）** | 1 / ~480 | **0.2%**（当前仅 1 个骨架对象）|
| 计划完全匹配（S1~S12 完成） | ~470 / ~480 | **98%**（预计）|
| 合并到同文件 | ~12 / ~480 | **2.5%** |
| 重命名（Rust 命名规范） | ~3 / ~480 | **0.6%** |
| 不迁移 | ~5 / ~480 | **1%**（JUnit 4 / TestNG / @Ignore 等）|
| 拆分（1 Java → 多 Rust） | 0 | 0 |

---

## 八、检查结论

✅ **类型命名规则**：vernal-test 计划完全沿用 Spring 的 PascalCase 类名 + Rust 的 snake_case 文件名映射规则，与 vernal-expression / vernal-aspects 一致。

⚠️ **品牌名问题**：`SpringExtension` / `SpringRunner` / `SpringJUnitConfig` / `SpringBootTest` 等 4 个 API 名含 `Spring`，建议重命名为 `Vernal*`（对齐 vernal 框架整体命名风格）。

⚠️ **潜在冲突**：`Commit` 注解与 Cargo 标准库 `commit` 不直接冲突（仅在 `std::process` 命名空间），但建议文件用 `commit_annotation.rs` 以避免歧义。

✅ **合并策略**：12 处 Java 类型合并到 1 个 Rust 文件，全部为内部支撑类或 trait 默认实现，符合"一文件一对象"原则。

✅ **不迁移对象**已明确：JUnit 4 / TestNG / @Ignore / @ExpectedException（deprecated）等 5 处。

✅ **vernal 特有**：当前 0 个，未来 5 个计划新增（均为 Spring 没有的 Rust 化创新）。

> 当前对标度：~0.2%。完成 S1~S12 后预计可达 ~98%。
> 完成度主要取决于 S2（注解宏）、S4（ContextLoader）、S8（Mock 对象）三个阶段。
