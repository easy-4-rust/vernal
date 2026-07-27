# spring-test → vernal-test 对象级对照表（验收清单）

> 基线：Spring Framework **7.0.8**，spring-test 共 **459 个 Java 主类 + 22 个 Kotlin 主文件 + ~80 个 mock 类**（合并后约 480 个公开对象）。
> 本文档与 `spring-test到vernal-test语义迁移对照表.md` 互补：语义表回答"功能有没有"，
> 本表回答"**每个 Java 对象落在哪个 Rust 文件**"，是结构拆分与补缺的验收清单。

## 目标工程结构（Cargo crate）

```
vernal-test/
├── Cargo.toml                       # crate 根
└── src/
    ├── lib.rs                       # crate 门面（只做 mod 声明 + re-export）
    │
    ├── context/                     # 对标 org.springframework.test.context
    │   ├── mod.rs
    │   ├── test_context.rs          # TestContext trait
    │   ├── test_context_manager.rs  # TestContextManager
    │   ├── test_context_bootstrapper.rs
    │   ├── bootstrap_context.rs
    │   ├── bootstrap_utils.rs
    │   ├── bootstrap_with.rs
    │   ├── attribute_accessor.rs
    │   ├── test_context_annotation_utils.rs
    │   ├── method_invoker.rs        # MethodInvoker + DefaultMethodInvoker
    │   ├── test_constructor.rs
    │   ├── nested_test_configuration.rs
    │   ├── application_context_failure_processor.rs
    │   ├── context_load_exception.rs
    │   ├── dynamic_property_source.rs
    │   ├── dynamic_property.rs      # DynamicProperty + DynamicPropertyRegistry + DynamicPropertyRegistrar
    │   ├── merged_context_configuration.rs
    │   ├── context_configuration_attributes.rs
    │   ├── context_hierarchy.rs
    │   ├── context_customizer.rs
    │   ├── context_customizer_factory.rs
    │   ├── context_customizer_factories.rs
    │   ├── loader/
    │   │   ├── mod.rs
    │   │   ├── context_loader.rs
    │   │   ├── smart_context_loader.rs
    │   │   ├── abstract_context_loader.rs
    │   │   ├── abstract_generic_context_loader.rs
    │   │   ├── abstract_generic_web_context_loader.rs
    │   │   ├── abstract_delegating_smart_context_loader.rs
    │   │   ├── annotation_config_context_loader.rs
    │   │   ├── annotation_config_context_loader_utils.rs
    │   │   ├── annotation_config_web_context_loader.rs
    │   │   ├── generic_context_loader.rs
    │   │   ├── generic_xml_context_loader.rs
    │   │   ├── aot_context_loader.rs
    │   │   └── context_loader_utils.rs
    │   ├── cache/
    │   │   ├── mod.rs
    │   │   ├── context_cache.rs
    │   │   ├── default_context_cache.rs
    │   │   ├── cache_aware_context_loader_delegate.rs
    │   │   └── context_cache_utils.rs
    │   ├── aot/
    │   │   ├── mod.rs
    │   │   ├── aot_test_attributes.rs
    │   │   ├── aot_test_attributes_factory.rs
    │   │   ├── aot_test_attributes_code_generator.rs
    │   │   ├── aot_test_context_initializers.rs
    │   │   ├── aot_test_context_initializers_factory.rs
    │   │   ├── aot_test_context_initializers_code_generator.rs
    │   │   └── aot_merged_context_configuration.rs
    │   ├── bean/
    │   │   ├── mod.rs
    │   │   ├── bean_override.rs
    │   │   ├── bean_override_strategy.rs
    │   │   ├── bean_override_handler.rs
    │   │   ├── bean_override_processor.rs
    │   │   ├── bean_override_reflective_processor.rs
    │   │   ├── bean_override_context_customizer.rs
    │   │   ├── bean_override_context_customizer_factory.rs
    │   │   ├── bean_override_bean_factory_post_processor.rs
    │   │   ├── bean_override_registry.rs
    │   │   └── bean_override_test_execution_listener.rs       # 重定向到 listener/
    │   ├── event/
    │   │   ├── mod.rs
    │   │   ├── before_test_class_event.rs
    │   │   ├── after_test_class_event.rs
    │   │   ├── before_test_method_event.rs
    │   │   ├── after_test_method_event.rs
    │   │   ├── before_test_execution_event.rs
    │   │   └── after_test_execution_event.rs
    │   ├── hint/
    │   │   ├── mod.rs
    │   │   ├── test_context_hint.rs
    │   │   └── search_hints.rs
    │   ├── observation/
    │   │   └── mod.rs
    │   ├── support/
    │   │   ├── mod.rs
    │   │   ├── annotation_context_loader_utils.rs
    │   │   └── default_test_context_bootstrapper.rs
    │   └── util/
    │       └── mod.rs
    │
    ├── listener/                    # 对标 TestExecutionListener 体系
    │   ├── mod.rs
    │   ├── test_execution_listener.rs    # trait + 7 个回调
    │   ├── test_execution_listeners.rs   # @TestExecutionListeners + MergeMode
    │   ├── transactional_test_execution_listener.rs
    │   ├── sql_scripts_test_execution_listener.rs
    │   ├── sql_scripts_registrar.rs
    │   ├── dependency_injection_test_execution_listener.rs
    │   ├── dirties_context_test_execution_listener.rs
    │   ├── event_publishing_test_execution_listener.rs
    │   ├── application_events_test_execution_listener.rs
    │   ├── bean_overriding_test_execution_listener.rs
    │   ├── aot_test_execution_listener.rs
    │   └── common_caches_test_execution_listener.rs
    │
    ├── annotation/                  # 对标 org.springframework.test.annotation
    │   ├── mod.rs
    │   ├── context_configuration.rs
    │   ├── active_profiles.rs            # @ActiveProfiles + Resolver
    │   ├── active_profiles_resolver.rs
    │   ├── active_profiles_utils.rs
    │   ├── test_property_source.rs
    │   ├── test_property_sources.rs
    │   ├── dirties_context.rs            # @DirtiesContext + HierarchyMode
    │   ├── commit.rs
    │   ├── rollback.rs
    │   ├── before_transaction.rs
    │   ├── after_transaction.rs
    │   ├── sql.rs                        # @Sql
    │   ├── sql_group.rs                  # @SqlGroup
    │   ├── timed.rs                      # @Timed + TimedAnnotationTestContextHandler
    │   ├── repeat.rs                     # @Repeat
    │   ├── profile_value_source.rs
    │   ├── if_profile_value.rs
    │   └── expected_exception.rs         # 可选
    │
    ├── transaction/                 # 对标 org.springframework.test.context.transaction + jdbc
    │   ├── mod.rs
    │   ├── transactional_test_execution_listener.rs   # 正式实现
    │   ├── test_transaction.rs
    │   ├── transaction_assert.rs
    │   └── transactional_test_util.rs
    │
    ├── jdbc/                        # 对标 org.springframework.test.jdbc + test.context.jdbc
    │   ├── mod.rs
    │   ├── jdbc_test_utils.rs
    │   ├── simple_jdbc_test_utils.rs
    │   └── expected_exception.rs            # 测试版
    │
    ├── web/                         # 对标 org.springframework.test.web.*
    │   ├── mod.rs
    │   ├── servlet/
    │   │   ├── mod.rs
    │   │   ├── mock_mvc.rs
    │   │   ├── mock_mvc_builder.rs
    │   │   ├── mock_mvc_builders.rs        # StandaloneMockMvcBuilder + DefaultMockMvcBuilder
    │   │   ├── mock_mvc_result_handlers.rs
    │   │   ├── result_actions.rs
    │   │   ├── request_builder.rs
    │   │   ├── cookie/
    │   │   │   ├── mod.rs
    │   │   │   ├── mock_cookie.rs
    │   │   │   └── cookie_assertions.rs
    │   │   ├── result/
    │   │   │   ├── mod.rs
    │   │   │   ├── status_result_matchers.rs
    │   │   │   ├── header_result_matchers.rs
    │   │   │   ├── cookie_result_matchers.rs
    │   │   │   ├── content_result_matchers.rs
    │   │   │   ├── json_path_result_matchers.rs
    │   │   │   ├── xpath_result_matchers.rs
    │   │   │   ├── model_result_matchers.rs
    │   │   │   ├── view_result_matchers.rs
    │   │   │   └── flash_attribute_result_matchers.rs
    │   │   └── dsl/
    │   │       ├── mod.rs
    │   │       ├── mock_mvc_extensions.rs
    │   │       ├── mock_mvc_result_matchers_dsl.rs
    │   │       ├── mock_mvc_result_handlers_dsl.rs
    │   │       ├── status_result_matchers_dsl.rs
    │   │       ├── header_result_matchers_dsl.rs
    │   │       ├── cookie_result_matchers_dsl.rs
    │   │       ├── content_result_matchers_dsl.rs
    │   │       ├── json_path_result_matchers_dsl.rs
    │   │       ├── xpath_result_matchers_dsl.rs
    │   │       ├── model_result_matchers_dsl.rs
    │   │       ├── view_result_matchers_dsl.rs
    │   │       ├── flash_attribute_result_matchers_dsl.rs
    │   │       ├── request_result_matchers_dsl.rs
    │   │       ├── result_actions_dsl.rs
    │   │       ├── mock_http_servlet_request_dsl.rs
    │   │       └── mock_multipart_http_servlet_request_dsl.rs
    │   ├── reactive/
    │   │   ├── mod.rs
    │   │   ├── web_test_client.rs
    │   │   ├── web_test_client_builder.rs
    │   │   └── web_test_client_extensions.rs
    │   └── client/
    │       ├── mod.rs
    │       ├── rest_test_client.rs
    │       └── rest_test_client_extensions.rs
    │
    ├── mock/                        # 对标 org.springframework.mock.*
    │   ├── mod.rs
    │   ├── env/
    │   │   ├── mod.rs
    │   │   ├── mock_environment.rs
    │   │   ├── mock_property_source.rs
    │   │   ├── mock_property_resolver.rs
    │   │   ├── mock_bean_factory.rs
    │   │   └── mock_prototype_target_source.rs
    │   ├── http/
    │   │   ├── mod.rs
    │   │   ├── mock_http_servlet_request.rs
    │   │   ├── mock_http_servlet_response.rs
    │   │   ├── mock_http_session.rs
    │   │   ├── mock_multipart_file.rs
    │   │   ├── mock_multipart_http_servlet_request.rs
    │   │   ├── mock_part.rs
    │   │   ├── mock_http_output_message.rs
    │   │   └── mock_http_input_message.rs
    │   ├── http/client/
    │   │   ├── mod.rs
    │   │   ├── mock_client_http_request.rs
    │   │   ├── mock_client_http_response.rs
    │   │   └── mock_client_http_request_factory.rs
    │   ├── http/server/
    │   │   ├── mod.rs
    │   │   ├── mock_server_http_request.rs
    │   │   ├── mock_server_http_response.rs
    │   │   ├── mock_reactive_server_http_request.rs
    │   │   └── mock_server_web_exchange.rs
    │   ├── web/
    │   │   ├── mod.rs
    │   │   ├── mock_servlet_context.rs
    │   │   ├── mock_request_dispatcher.rs
    │   │   ├── mock_servlet_config.rs
    │   │   ├── mock_filter_chain.rs
    │   │   ├── mock_filter_config.rs
    │   │   ├── mock_async_context.rs
    │   │   ├── mock_dispatcher_type.rs
    │   │   ├── mock_error_page_register.rs
    │   │   ├── mock_page_context.rs
    │   │   └── mock_servlet_context_listener.rs
    │   ├── web/reactive/
    │   │   ├── mod.rs
    │   │   ├── mock_server_http_request.rs        # reactive 版（与 http/server 区分）
    │   │   ├── mock_server_http_response.rs
    │   │   ├── mock_server_web_exchange.rs
    │   │   └── mock_server_cookie.rs
    │   └── web/server/
    │       ├── mod.rs
    │       ├── mock_web_service_connection.rs
    │       ├── mock_web_service_message.rs
    │       └── mock_web_service_message_factory.rs
    │
    ├── http/                        # 对标 org.springframework.test.http
    │   ├── mod.rs
    │   ├── media_type_assertions.rs
    │   ├── header_assertions.rs
    │   ├── cookie_assertions.rs
    │   ├── json_content_assert.rs
    │   ├── json_path_assertions.rs
    │   ├── xpath_assertions.rs
    │   └── request_matcher.rs
    │
    ├── json/                        # 对标 org.springframework.test.json
    │   ├── mod.rs
    │   ├── json_path_assert.rs
    │   └── json_value_assert.rs
    │
    ├── validation/                  # 对标 org.springframework.test.validation
    │   ├── mod.rs
    │   └── binding_result_assert.rs
    │
    ├── util/                        # 对标 org.springframework.test.util
    │   ├── mod.rs
    │   ├── reflection_test_utils.rs
    │   ├── aop_test_utils.rs
    │   ├── assertion_errors.rs
    │   └── annotation_test_utils.rs
    │
    ├── junit/                       # JUnit 集成层
    │   ├── mod.rs
    │   ├── vernal_extension.rs
    │   ├── spring_vernal_config.rs
    │   ├── vernal_runner.rs
    │   └── web_vernal_config.rs
    │
    ├── macros/                     # 单独 crate vernal-test-macros
    │   ├── lib.rs
    │   ├── vernal_test.rs           # #[vernal_test]
    │   ├── with_context.rs          # #[with_context]
    │   ├── mock_bean.rs             # #[mock_bean]
    │   ├── spy_bean.rs              # #[spy_bean]
    │   └── spring_test_attribute.rs # #[derive(SpringTestAttribute)]
    │
    └── integrations/                # 与 vernal-* 集成
        ├── mod.rs
        ├── vernal_context.rs
        ├── vernal_beans.rs
        └── vernal_tx.rs
```

---

## 一、org.springframework.test.context（核心接口）

| Java 类/接口 | Rust 文件 | 工作量 |
|-------------|----------|------|
| `TestContext` | `context/test_context.rs` | 0.5d |
| `TestContextManager` | `context/test_context_manager.rs` | 0.5d |
| `TestContextBootstrapper` | `context/test_context_bootstrapper.rs` | 0.5d |
| `BootstrapContext` | `context/bootstrap_context.rs` | 0.25d |
| `BootstrapUtils` | `context/bootstrap_utils.rs` | 0.25d |
| `BootstrapWith` | `context/bootstrap_with.rs` | 0.25d |
| `AttributeAccessor` | `context/attribute_accessor.rs` | 0.25d |
| `TestContextAnnotationUtils` | `context/test_context_annotation_utils.rs` | 0.5d |
| `MethodInvoker` | `context/method_invoker.rs` | 0.25d |
| `DefaultMethodInvoker` | `context/method_invoker.rs`（合并）| 0d |
| `TestConstructor` | `context/test_constructor.rs` | 0.25d |
| `NestedTestConfiguration` | `context/nested_test_configuration.rs` | 0.25d |
| `ApplicationContextFailureProcessor` | `context/application_context_failure_processor.rs` | 0.25d |
| `ContextLoadException` | `context/context_load_exception.rs` | 0.25d |
| `DynamicPropertySource` | `context/dynamic_property_source.rs` | 0.25d |
| `DynamicProperty` | `context/dynamic_property.rs` | 0.5d |
| `DynamicPropertyRegistry` | `context/dynamic_property.rs`（合并）| 0d |
| `DynamicPropertyRegistrar` | `context/dynamic_property.rs`（合并）| 0d |
| `MergedContextConfiguration` | `context/merged_context_configuration.rs` | 0.5d |
| `ContextConfigurationAttributes` | `context/context_configuration_attributes.rs` | 0.25d |
| `ContextHierarchy` | `context/context_hierarchy.rs` | 0.25d |
| `ContextCustomizer` | `context/context_customizer.rs` | 0.25d |
| `ContextCustomizerFactory` | `context/context_customizer_factory.rs` | 0.25d |
| `ContextCustomizerFactories` | `context/context_customizer_factories.rs` | 0.25d |
| `ContextLoader` | `context/loader/context_loader.rs` | 0.25d |
| `SmartContextLoader` | `context/loader/smart_context_loader.rs` | 0.25d |
| `AbstractContextLoader` | `context/loader/abstract_context_loader.rs` | 0.25d |
| `AbstractGenericContextLoader` | `context/loader/abstract_generic_context_loader.rs` | 0.5d |
| `AbstractGenericWebContextLoader` | `context/loader/abstract_generic_web_context_loader.rs` | 0.5d |
| `AbstractDelegatingSmartContextLoader` | `context/loader/abstract_delegating_smart_context_loader.rs` | 0.5d |
| `AnnotationConfigContextLoader` | `context/loader/annotation_config_context_loader.rs` | 0.5d |
| `AnnotationConfigContextLoaderUtils` | `context/loader/annotation_config_context_loader_utils.rs` | 0.25d |
| `AnnotationConfigWebContextLoader` | `context/loader/annotation_config_web_context_loader.rs` | 0.5d |
| `GenericContextLoader` | `context/loader/generic_context_loader.rs` | 0.5d |
| `GenericXmlContextLoader` | `context/loader/generic_xml_context_loader.rs` | 0.5d |
| `AotContextLoader` | `context/loader/aot_context_loader.rs` | 0.5d |
| `ContextLoaderUtils` | `context/loader/context_loader_utils.rs` | 0.25d |
| `ContextCache` | `context/cache/context_cache.rs` | 0.25d |
| `DefaultContextCache` | `context/cache/default_context_cache.rs` | 0.5d |
| `CacheAwareContextLoaderDelegate` | `context/cache/cache_aware_context_loader_delegate.rs` | 0.5d |
| `ContextCacheUtils` | `context/cache/context_cache_utils.rs` | 0.25d |
| `AotTestAttributes` | `context/aot/aot_test_attributes.rs` | 0.25d |
| `AotTestAttributesFactory` | `context/aot/aot_test_attributes_factory.rs` | 0.25d |
| `AotTestAttributesCodeGenerator` | `context/aot/aot_test_attributes_code_generator.rs` | 0.25d |
| `AotTestContextInitializers` | `context/aot/aot_test_context_initializers.rs` | 0.25d |
| `AotTestContextInitializersFactory` | `context/aot/aot_test_context_initializers_factory.rs` | 0.25d |
| `AotTestContextInitializersCodeGenerator` | `context/aot/aot_test_context_initializers_code_generator.rs` | 0.25d |
| `AotMergedContextConfiguration` | `context/aot/aot_merged_context_configuration.rs` | 0.25d |
| `BeanOverride` | `context/bean/bean_override.rs` | 0.25d |
| `BeanOverrideStrategy` | `context/bean/bean_override_strategy.rs` | 0.25d |
| `BeanOverrideHandler` | `context/bean/bean_override_handler.rs` | 0.25d |
| `BeanOverrideProcessor` | `context/bean/bean_override_processor.rs` | 0.25d |
| `BeanOverrideReflectiveProcessor` | `context/bean/bean_override_reflective_processor.rs` | 0.25d |
| `BeanOverrideContextCustomizer` | `context/bean/bean_override_context_customizer.rs` | 0.25d |
| `BeanOverrideContextCustomizerFactory` | `context/bean/bean_override_context_customizer_factory.rs` | 0.25d |
| `BeanOverrideBeanFactoryPostProcessor` | `context/bean/bean_override_bean_factory_post_processor.rs` | 0.25d |
| `BeanOverrideRegistry` | `context/bean/bean_override_registry.rs` | 0.25d |
| `BeanOverrideTestExecutionListener` | `listener/bean_overriding_test_execution_listener.rs` | 0.25d |
| `BeforeTestClassEvent` | `context/event/before_test_class_event.rs` | 0.1d |
| `AfterTestClassEvent` | `context/event/after_test_class_event.rs` | 0.1d |
| `BeforeTestMethodEvent` | `context/event/before_test_method_event.rs` | 0.1d |
| `AfterTestMethodEvent` | `context/event/after_test_method_event.rs` | 0.1d |
| `BeforeTestExecutionEvent` | `context/event/before_test_execution_event.rs` | 0.1d |
| `AfterTestExecutionEvent` | `context/event/after_test_execution_event.rs` | 0.1d |
| `TestContextHint` | `context/hint/test_context_hint.rs` | 0.25d |
| `Search` | `context/hint/search_hints.rs` | 0.25d |

---

## 二、TestExecutionListener 体系

| Java 类/接口 | Rust 文件 | 工作量 |
|-------------|----------|------|
| `TestExecutionListener` | `listener/test_execution_listener.rs` | 0.5d |
| `TestExecutionListeners` | `listener/test_execution_listeners.rs` | 0.5d |
| `TransactionalTestExecutionListener` | `listener/transactional_test_execution_listener.rs` | 0.5d |
| `SqlScriptsTestExecutionListener` | `listener/sql_scripts_test_execution_listener.rs` | 0.5d |
| `SqlScriptsRegistrar` | `listener/sql_scripts_registrar.rs` | 0.25d |
| `DependencyInjectionTestExecutionListener` | `listener/dependency_injection_test_execution_listener.rs` | 0.5d |
| `AbstractDirtiesContextTestExecutionListener` | `listener/dirties_context_test_execution_listener.rs` | 0.5d |
| `DirtiesContextTestExecutionListener` | `listener/dirties_context_test_execution_listener.rs`（合并） | 0d |
| `EventPublishingTestExecutionListener` | `listener/event_publishing_test_execution_listener.rs` | 0.25d |
| `ApplicationEventsTestExecutionListener` | `listener/application_events_test_execution_listener.rs` | 0.25d |
| `ApplicationEventsHolder` | `listener/event_publishing_test_execution_listener.rs`（合并） | 0d |
| `ApplicationEventsApplicationListener` | `listener/event_publishing_test_execution_listener.rs`（合并） | 0d |
| `BeanOverrideTestExecutionListener` | `listener/bean_overriding_test_execution_listener.rs` | 0.25d |
| `AotTestExecutionListener` | `listener/aot_test_execution_listener.rs` | 0.25d |
| `CommonCachesTestExecutionListener` | `listener/common_caches_test_execution_listener.rs` | 0.25d |

---

## 三、org.springframework.test.annotation（注解）

| Java 注解 | Rust 文件 | 工作量 |
|---------|----------|------|
| `@ContextConfiguration` | `annotation/context_configuration.rs` | 0.25d |
| `@ActiveProfiles` | `annotation/active_profiles.rs` | 0.5d |
| `ActiveProfilesResolver` | `annotation/active_profiles_resolver.rs` | 0.25d |
| `ActiveProfilesUtils` | `annotation/active_profiles_utils.rs` | 0.25d |
| `@TestPropertySource` | `annotation/test_property_source.rs` | 0.5d |
| `@TestPropertySources` | `annotation/test_property_sources.rs` | 0.25d |
| `@DirtiesContext` | `annotation/dirties_context.rs` | 0.5d |
| `@HierarchyMode` | `annotation/dirties_context.rs`（合并） | 0d |
| `@Commit` | `annotation/commit.rs` | 0.1d |
| `@Rollback` | `annotation/rollback.rs` | 0.1d |
| `@BeforeTransaction` | `annotation/before_transaction.rs` | 0.1d |
| `@AfterTransaction` | `annotation/after_transaction.rs` | 0.1d |
| `@Sql` | `annotation/sql.rs` | 0.5d |
| `@SqlGroup` | `annotation/sql_group.rs` | 0.25d |
| `@Timed` | `annotation/timed.rs` | 0.25d |
| `@Repeat` | `annotation/repeat.rs` | 0.25d |
| `@ProfileValueSourceConfiguration` | `annotation/profile_value_source.rs` | 0.25d |
| `@IfProfileValue` | `annotation/if_profile_value.rs` | 0.25d |
| `@ExpectedException` | `annotation/expected_exception.rs` | 0.1d |

---

## 四、org.springframework.test.context.transaction（事务测试）

| Java 类/接口 | Rust 文件 | 工作量 |
|-------------|----------|------|
| `TransactionalTestExecutionListener` | `transaction/transactional_test_execution_listener.rs` | 0.5d |
| `TestTransaction` | `transaction/test_transaction.rs` | 0.5d |
| `TransactionAssert` | `transaction/transaction_assert.rs` | 0.25d |
| `TransactionalTestUtil` | `transaction/transactional_test_util.rs` | 0.25d |

---

## 五、org.springframework.test.jdbc + org.springframework.test.context.jdbc（JDBC 测试）

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `JdbcTestUtils` | `jdbc/jdbc_test_utils.rs` | 0.5d |
| `SimpleJdbcTestUtils` | `jdbc/simple_jdbc_test_utils.rs` | 0.25d |
| `ExpectedException` (test version) | `jdbc/expected_exception.rs` | 0.25d |

---

## 六、org.springframework.test.web.*（Web 测试）

### 6.1 Web Servlet 测试

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockMvc` | `web/servlet/mock_mvc.rs` | 0.5d |
| `MockMvcBuilder` | `web/servlet/mock_mvc_builder.rs` | 0.5d |
| `ConfigurableMockMvcBuilder` | `web/servlet/mock_mvc_builder.rs`（合并） | 0d |
| `StandaloneMockMvcBuilder` | `web/servlet/mock_mvc_builders.rs` | 0.5d |
| `DefaultMockMvcBuilder` | `web/servlet/mock_mvc_builders.rs`（合并） | 0d |
| `MockMvcBuilders` | `web/servlet/mock_mvc_builders.rs`（合并） | 0d |
| `MockMvcResultHandlers` | `web/servlet/mock_mvc_result_handlers.rs` | 0.25d |
| `PrintMockMvcResultHandler` | `web/servlet/mock_mvc_result_handlers.rs`（合并） | 0d |
| `ResultActions` | `web/servlet/result_actions.rs` | 0.5d |
| `DefaultResultActions` | `web/servlet/result_actions.rs`（合并） | 0d |
| `MockHttpServletRequestBuilder` | `web/servlet/request_builder.rs` | 0.25d |
| `MockMultipartHttpServletRequestBuilder` | `web/servlet/request_builder.rs`（合并） | 0d |
| `MockCookie` | `web/servlet/cookie/mock_cookie.rs` | 0.25d |
| `CookieAssertions` | `web/servlet/cookie/cookie_assertions.rs` | 0.25d |
| `StatusResultMatchers` | `web/servlet/result/status_result_matchers.rs` | 0.25d |
| `HeaderResultMatchers` | `web/servlet/result/header_result_matchers.rs` | 0.25d |
| `CookieResultMatchers` | `web/servlet/result/cookie_result_matchers.rs` | 0.25d |
| `ContentResultMatchers` | `web/servlet/result/content_result_matchers.rs` | 0.25d |
| `JsonPathResultMatchers` | `web/servlet/result/json_path_result_matchers.rs` | 0.25d |
| `XpathResultMatchers` | `web/servlet/result/xpath_result_matchers.rs` | 0.25d |
| `ModelResultMatchers` | `web/servlet/result/model_result_matchers.rs` | 0.25d |
| `ViewResultMatchers` | `web/servlet/result/view_result_matchers.rs` | 0.25d |
| `FlashAttributeResultMatchers` | `web/servlet/result/flash_attribute_result_matchers.rs` | 0.25d |
| `RequestResultMatchers` | `web/servlet/result/request_result_matchers.rs` | 0.25d |

### 6.2 Web Reactive 测试

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `WebTestClient` | `web/reactive/web_test_client.rs` | 0.5d |
| `WebTestClientBuilder` | `web/reactive/web_test_client_builder.rs` | 0.5d |
| `WebTestClient.Builder` | `web/reactive/web_test_client_builder.rs`（合并） | 0d |

### 6.3 Web Client 测试

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `RestTestClient` | `web/client/rest_test_client.rs` | 0.5d |

### 6.4 Kotlin DSL（test/web/servlet/*.kt）

| Kotlin 类 | Rust 文件 | 工作量 |
|---------|----------|------|
| `MockMvcExtensions` | `web/servlet/dsl/mock_mvc_extensions.rs` | 0.25d |
| `MockMvcResultMatchersDsl` | `web/servlet/dsl/mock_mvc_result_matchers_dsl.rs` | 0.25d |
| `MockMvcResultHandlersDsl` | `web/servlet/dsl/mock_mvc_result_handlers_dsl.rs` | 0.25d |
| `StatusResultMatchersDsl` | `web/servlet/dsl/status_result_matchers_dsl.rs` | 0.25d |
| `HeaderResultMatchersDsl` | `web/servlet/dsl/header_result_matchers_dsl.rs` | 0.25d |
| `CookieResultMatchersDsl` | `web/servlet/dsl/cookie_result_matchers_dsl.rs` | 0.25d |
| `ContentResultMatchersDsl` | `web/servlet/dsl/content_result_matchers_dsl.rs` | 0.25d |
| `JsonPathResultMatchersDsl` | `web/servlet/dsl/json_path_result_matchers_dsl.rs` | 0.25d |
| `XpathResultMatchersDsl` | `web/servlet/dsl/xpath_result_matchers_dsl.rs` | 0.25d |
| `ModelResultMatchersDsl` | `web/servlet/dsl/model_result_matchers_dsl.rs` | 0.25d |
| `ViewResultMatchersDsl` | `web/servlet/dsl/view_result_matchers_dsl.rs` | 0.25d |
| `FlashAttributeResultMatchersDsl` | `web/servlet/dsl/flash_attribute_result_matchers_dsl.rs` | 0.25d |
| `RequestResultMatchersDsl` | `web/servlet/dsl/request_result_matchers_dsl.rs` | 0.25d |
| `ResultActionsDsl` | `web/servlet/dsl/result_actions_dsl.rs` | 0.25d |
| `MockHttpServletRequestDsl` | `web/servlet/dsl/mock_http_servlet_request_dsl.rs` | 0.25d |
| `MockMultipartHttpServletRequestDsl` | `web/servlet/dsl/mock_multipart_http_servlet_request_dsl.rs` | 0.25d |
| `WebTestClientExtensions` | `web/reactive/web_test_client_extensions.rs` | 0.25d |
| `RestTestClientExtensions` | `web/client/rest_test_client_extensions.rs` | 0.25d |

---

## 七、org.springframework.mock.*（Mock 对象 ~80 类）

### 7.1 mock.env

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockEnvironment` | `mock/env/mock_environment.rs` | 0.25d |
| `MockPropertySource` | `mock/env/mock_property_source.rs` | 0.25d |
| `MockPropertyResolver` | `mock/env/mock_property_resolver.rs` | 0.25d |
| `MockBeanFactory` | `mock/env/mock_bean_factory.rs` | 0.25d |
| `MockPrototypeTargetSource` | `mock/env/mock_prototype_target_source.rs` | 0.25d |

### 7.2 mock.http

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockHttpServletRequest` | `mock/http/mock_http_servlet_request.rs` | 0.5d |
| `MockHttpServletResponse` | `mock/http/mock_http_servlet_response.rs` | 0.5d |
| `MockHttpSession` | `mock/http/mock_http_session.rs` | 0.25d |
| `MockMultipartFile` | `mock/http/mock_multipart_file.rs` | 0.25d |
| `MockMultipartHttpServletRequest` | `mock/http/mock_multipart_http_servlet_request.rs` | 0.25d |
| `MockPart` | `mock/http/mock_part.rs` | 0.25d |
| `MockHttpOutputMessage` | `mock/http/mock_http_output_message.rs` | 0.25d |
| `MockHttpInputMessage` | `mock/http/mock_http_input_message.rs` | 0.25d |

### 7.3 mock.http.client

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockClientHttpRequest` | `mock/http/client/mock_client_http_request.rs` | 0.25d |
| `MockClientHttpResponse` | `mock/http/client/mock_client_http_response.rs` | 0.25d |
| `MockClientHttpRequestFactory` | `mock/http/client/mock_client_http_request_factory.rs` | 0.25d |
| `MockAsyncClientHttpRequest` | `mock/http/client/mock_async_client_http_request.rs` | 0.25d |

### 7.4 mock.http.server

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockServerHttpRequest` | `mock/http/server/mock_server_http_request.rs` | 0.25d |
| `MockServerHttpResponse` | `mock/http/server/mock_server_http_response.rs` | 0.25d |
| `MockReactiveServerHttpRequest` | `mock/http/server/mock_reactive_server_http_request.rs` | 0.25d |
| `MockServerWebExchange` | `mock/http/server/mock_server_web_exchange.rs` | 0.25d |

### 7.5 mock.web

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockServletContext` | `mock/web/mock_servlet_context.rs` | 0.5d |
| `MockRequestDispatcher` | `mock/web/mock_request_dispatcher.rs` | 0.25d |
| `MockServletConfig` | `mock/web/mock_servlet_config.rs` | 0.25d |
| `MockFilterChain` | `mock/web/mock_filter_chain.rs` | 0.25d |
| `MockFilterConfig` | `mock/web/mock_filter_config.rs` | 0.25d |
| `MockAsyncContext` | `mock/web/mock_async_context.rs` | 0.25d |
| `MockDispatcherType` | `mock/web/mock_dispatcher_type.rs` | 0.1d |
| `MockErrorPageRegister` | `mock/web/mock_error_page_register.rs` | 0.1d |
| `MockPageContext` | `mock/web/mock_page_context.rs` | 0.25d |
| `MockServletContextListener` | `mock/web/mock_servlet_context_listener.rs` | 0.25d |

### 7.6 mock.web.reactive

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockServerHttpRequest` | `mock/web/reactive/mock_server_http_request.rs` | 0.25d |
| `MockServerHttpResponse` | `mock/web/reactive/mock_server_http_response.rs` | 0.25d |
| `MockServerWebExchange` | `mock/web/reactive/mock_server_web_exchange.rs` | 0.25d |
| `MockServerCookie` | `mock/web/reactive/mock_server_cookie.rs` | 0.25d |

### 7.7 mock.web.server

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MockWebServiceConnection` | `mock/web/server/mock_web_service_connection.rs` | 0.25d |
| `MockWebServiceMessage` | `mock/web/server/mock_web_service_message.rs` | 0.25d |
| `MockWebServiceMessageFactory` | `mock/web/server/mock_web_service_message_factory.rs` | 0.25d |

---

## 八、org.springframework.test.http / json / validation

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `MediaTypeAssert` | `http/media_type_assertions.rs` | 0.25d |
| `HeaderAssertions` | `http/header_assertions.rs` | 0.25d |
| `CookieAssertions` (http pkg) | `http/cookie_assertions.rs` | 0.25d |
| `ContentRequestMatchers` | `http/request_matcher.rs` | 0.25d |
| `ContentResultMatchers` (http pkg) | `http/content_result_matchers.rs` | 0.25d |
| `JsonPathExpectationsHelper` | `json/json_path_assertions.rs` | 0.25d |
| `JsonPathAssert` | `json/json_path_assert.rs` | 0.25d |
| `JsonValueAssert` | `json/json_value_assert.rs` | 0.25d |
| `BindingResultAssert` | `validation/binding_result_assert.rs` | 0.25d |

---

## 九、org.springframework.test.util（工具类）

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `ReflectionTestUtils` | `util/reflection_test_utils.rs` | 0.5d |
| `AopTestUtils` | `util/aop_test_utils.rs` | 0.25d |
| `AssertionErrors` | `util/assertion_errors.rs` | 0.1d |
| `AnnotationTestUtils` | `util/annotation_test_utils.rs` | 0.25d |

---

## 十、过程宏（vernal-test-macros）

| 宏 | Rust 文件 | 工作量 |
|---|----------|------|
| `#[vernal_test]` | `macros/src/vernal_test.rs` | 1d |
| `#[with_context]` | `macros/src/with_context.rs` | 0.25d |
| `#[mock_bean]` | `macros/src/mock_bean.rs` | 0.25d |
| `#[spy_bean]` | `macros/src/spy_bean.rs` | 0.25d |
| `#[derive(SpringTestAttribute)]` | `macros/src/spring_test_attribute.rs` | 0.25d |

---

## 十一、JUnit 集成层

| Java 类 | Rust 文件 | 工作量 |
|--------|----------|------|
| `SpringExtension` | `junit/vernal_extension.rs` | 0.25d |
| `@SpringJUnitConfig` | `junit/spring_vernal_config.rs` | 0.25d |
| `SpringRunner` | `junit/vernal_runner.rs` | 0.25d |
| `@WebMvcTest` | `junit/web_vernal_config.rs` | 0.25d |

---

## 十二、统计

| 包 | Java 对象数 | Rust 文件数 | 总工作量 |
|---|---|---|---|
| `org.springframework.test.context` | ~80 | ~80（含子包） | 25d |
| `org.springframework.test.context.event` | 6 | 6 | 0.6d |
| `org.springframework.test.context.hint` | 2 | 2 | 0.5d |
| `org.springframework.test.context.support` | 2 | 2 | 0.5d |
| `org.springframework.test.context.util` | 1 | 1 | 0.25d |
| `TestExecutionListener 体系` | 15 | 11 | 4d |
| `org.springframework.test.annotation` | 19 | 19 | 3.5d |
| `org.springframework.test.context.transaction` | 4 | 4 | 1.5d |
| `org.springframework.test.jdbc` + `context.jdbc` | 3 | 3 | 1d |
| `org.springframework.test.web.servlet` | ~40 | ~40 | 6d |
| `org.springframework.test.web.reactive` | 3 | 3 | 1d |
| `org.springframework.test.web.client` | 1 | 1 | 0.5d |
| Kotlin DSL | 18 | 18 | 4.5d |
| `org.springframework.mock.*` | ~50 | ~50 | 8d |
| `org.springframework.test.http` / `json` / `validation` | ~10 | 9 | 2d |
| `org.springframework.test.util` | 4 | 4 | 1.1d |
| `vernal-test-macros` | — | 5 | 2d |
| `junit/ 集成` | 4 | 4 | 1d |
| **合计** | **~280** | **~262** | **~62d** |

> 注：Java 对象数剔除 package-info、内部类、相同语义的合并类（如 `Abstract*` 与其唯一
> 子类合并为同一 Rust 文件）。

---

## 十三、不迁移的对象（明确边界）

| Java 类 | 不迁移原因 |
|--------|----------|
| `org.springframework.test.context.junit4.*` 全部 | JUnit 4 已停更，仅保留 Jupiter |
| `org.springframework.test.context.testng.*` 全部 | TestNG 集成不在 vernal 范围内 |
| `org.springframework.test.context.junit.jupiter.EnabledIf*` | 改用 `#[cfg(...)]` / 自定义条件宏 |
| `org.springframework.test.context.junit.jupiter.SpringExtension`（Java 版）| 在 Rust 中实现等价 trait，类名 `VernalExtension` |
| `org.springframework.web.client.RestTemplate` 相关 Mock | 已被 `RestTestClient` 替代 |
| `org.springframework.test.web.servlet.setup.SecurityMockMvcConfigurers` | 安全相关，依赖 spring-security，超出 spring-test 范围 |
| `org.springframework.test.context.web.WebDelegatingSmartContextLoader` 等老 API | 7.0 已 deprecated |
| `org.springframework.test.context.event.EventPublishingTestExecutionListener` 中的 AWT 监听器 | Java AWT 特有 |
| `@Ignore`（JUnit 4 注解） | Rust 用 `#[ignore]` 属性（libtest 自带） |