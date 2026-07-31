<!-- migration-doc: authority=historical canonical=迁移路线图.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [迁移路线图.md](迁移路线图.md) 为准。

> **历史文档，非当前验收依据。** 当前版本见[迁移路线图](迁移路线图.md)。

# spring-test → vernal-test 全量迁移路线图

> 版本：v1.0（2026-07-27）｜基线：Spring Framework **7.0.8**（spring-test）
> 仓库：easy-4-rust/vernal @ dev ｜ 本文档随代码同步维护，每阶段完成时更新

## 一、总目标

以 Spring Framework 7.0.8 的 spring-test 模块为蓝本，实现功能语义完全对齐的
vernal-test crate，目标 **150+ 个 Rust 文件**，覆盖：

- TestContext 体系（TestContext / TestContextManager / TestContextBootstrapper）
- 注解驱动测试上下文（@ContextConfiguration / @ActiveProfiles / @TestPropertySource / @DirtiesContext …）
- TestExecutionListener 体系（含 12+ 内置 Listener）
- ContextLoader / SmartContextLoader 体系 + AnnotationConfig / Generic / Aot 加载器
- ContextCache / MergedContextConfiguration / ContextHierarchy
- 事务测试（@Transactional + @Commit / @Rollback + BeforeTransaction / AfterTransaction）
- JUnit Jupiter 集成（SpringExtension + @SpringJUnitConfig + @SpringBootTest 语义）
- Web 测试（MockMvc / WebTestClient / RestTestClient）
- Mock 对象体系（MockHttpServletRequest / MockHttpSession / MockBeanFactory 等 ~80 类）
- JDBC 测试支持（JdbcTestUtils + SimpleJdbcTestUtils）
- 工具类（ReflectionTestUtils / AopTestUtils / AssertionErrors 等）

### 设计原则

1. **功能语义对齐，实现方式 Rust 化**：Java 反射 → trait 对象；注解 → 过程宏；
   JUnit 扩展回调 → trait 默认方法；ThreadLocal → `task_local!`/线程局部存储
2. **每个 .rs 文件只对应一个 Java 对象**：对齐 vernal-expression / liteflow-rust
   的一文件一对象规范
3. **所有公开类型必须有中文文档注释**：结构体、枚举、trait、方法、字段
4. **模块路径 snake_case**，类型 PascalCase，方法 snake_case
5. **复用 Rust 测试生态**：测试上下文（test-context 思想）、参数化（rstest）、
   Mock（mockall）、Web Mock（wiremock / mockito）、断言（pretty_assertions）、
   异步测试（tokio-test）、属性测试（proptest）作为底层能力，vernal-test 在此
   之上提供 Spring-style 的"测试上下文管理"语义
6. **不实现字节码 / 类路径扫描**：Rust 编译期已确定模块结构，无需运行时扫描
7. **与 vernal 框架深度集成**：`TestContext` 持有 `vernal-context::ApplicationContext`
   引用；`@Autowired` 字段注入通过 `vernal-beans` 实现

### 与 vernal-test 当前状态的差距

| 维度 | 当前 | 目标 |
|------|------|------|
| 文件数 | 2 | 150+ |
| 代码行数 | 25 | 8000+ |
| TestContext 接口 | 骨架（仅 name） | 完整对齐 Spring 7.x 接口 |
| TestExecutionListener | 0 | 12+ 内置实现 |
| ContextLoader | 0 | 5 种（Generic/AnnotationConfig/Aot/WebGeneric/Delegating） |
| 注解 | 0 | 20+（过程宏）|
| Mock 对象 | 0 | ~80（Http/Web/Env/Jdbc）|
| Web 测试 | 0 | MockMvc / WebTestClient（依赖 vernal-web-testkit）|
| 事务测试 | 0 | 完整（依赖 vernal-tx）|
| 与 Spring 7.x 对标度 | 2% | 85%+ |

---

## 二、进度总览

| 阶段 | 内容 | 状态 | 验收 |
|---|---|---|---|
| S0 | 对象级对照表 + 语义迁移对照表 + 路线图 + 一致性检查 | ✅ | docs/vernal-test/ 四文档 |
| S1 | 核心 trait/struct 接口层（TestContext / TestContextManager / BootstrapContext 等 20 个） | ⬜ | 编译通过 + 最小测试 |
| S2 | 注解 + 宏（@ContextConfiguration / @ActiveProfiles / @TestPropertySource / @DirtiesContext / @Commit / @Rollback / @Sql / @Timed 等 22 个） | ⬜ | 宏展开测试 + e2e 集成测试 |
| S3 | TestExecutionListener 体系（trait + 12 个内置实现） | ⬜ | 监听器注册 + 回调顺序测试 |
| S4 | ContextLoader 体系（5 个加载器 + SmartContextLoader trait） | ⬜ | Annotation / Generic / Aot 加载 e2e |
| S5 | ContextCache + MergedContextConfiguration + ContextHierarchy | ⬜ | 缓存命中/失效/层级测试 |
| S6 | 事务测试集成（依赖 vernal-tx，提供 TransactionalTestExecutionListener + @Transactional 测试） | ⬜ | before/after/commit/rollback 全场景 |
| S7 | Web 测试（MockMvc / WebTestClient / RestTestClient，通过 vernal-web-testkit 桥接） | ⬜ | HTTP 请求/响应断言 e2e |
| S8 | Mock 对象体系（Http/Web/Env/Jdbc 共 ~80 类） | ⬜ | 每个 Mock 至少 1 单元测试 |
| S9 | JUnit 风格扩展 trait（类 JUnit Jupiter Extension 接入点） | ⬜ | 与 `#[test]` 协同工作 |
| S10 | 工具类（ReflectionTestUtils / AopTestUtils / AssertionErrors / TestContextAnnotationUtils） | ⬜ | 工具方法覆盖 |
| S11 | Vernal 集成（与 vernal-context / vernal-beans / vernal-tx 深度集成） | ⬜ | 真实容器集成测试 |
| S12 | 文档 + 中文注释收尾 | ⬜ | 所有文件中文注释覆盖 |

---

## 三、阶段详细计划

### S1 — 核心接口层（预估 2 天）

**范围**：`org.springframework.test.context` 包核心接口

**目标文件**：

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `context/test_context.rs` | TestContext | 0.5 天（充实现有骨架） |
| `context/test_context_manager.rs` | TestContextManager | 0.5 天 |
| `context/test_context_bootstrapper.rs` | TestContextBootstrapper | 0.5 天 |
| `context/bootstrap_context.rs` | BootstrapContext | 0.25 天 |
| `context/bootstrap_utils.rs` | BootstrapUtils | 0.25 天 |
| `context/bootstrap_with.rs` | BootstrapWith | 0.25 天 |
| `context/attribute_accessor.rs` | AttributeAccessor（来自 spring-core，重写精简版）| 0.25 天 |
| `context/test_context_annotation_utils.rs` | TestContextAnnotationUtils | 0.5 天 |
| `context/method_invoker.rs` | MethodInvoker + DefaultMethodInvoker | 0.25 天 |
| `context/test_constructor.rs` | TestConstructor | 0.25 天 |
| `context/nested_test_configuration.rs` | NestedTestConfiguration | 0.25 天 |
| `context/application_context_failure_processor.rs` | ApplicationContextFailureProcessor | 0.25 天 |
| `context/context_load_exception.rs` | ContextLoadException | 0.25 天 |
| `context/dynamic_property_source.rs` | DynamicPropertySource | 0.25 天 |
| `context/dynamic_property.rs` | DynamicProperty + DynamicPropertyRegistry + DynamicPropertyRegistrar | 0.5 天 |

**验收**：编译通过 + TestContext trait 至少 1 个 mock 实现测试

---

### S2 — 注解 + 过程宏（预估 3 天）

**范围**：`org.springframework.test.context` 包注解 + `vernal-macros` 过程宏

**目标文件**：

| 文件 | Java 注解 | 工作量 |
|------|---------|--------|
| `annotation/context_configuration.rs` | @ContextConfiguration | 0.25 天 |
| `annotation/active_profiles.rs` | @ActiveProfiles + ActiveProfilesResolver | 0.5 天 |
| `annotation/test_property_source.rs` | @TestPropertySource + @TestPropertySources | 0.5 天 |
| `annotation/dirties_context.rs` | @DirtiesContext + HierarchyMode | 0.5 天 |
| `annotation/commit.rs` | @Commit | 0.1 天 |
| `annotation/rollback.rs` | @Rollback | 0.1 天 |
| `annotation/before_transaction.rs` | @BeforeTransaction | 0.1 天 |
| `annotation/after_transaction.rs` | @AfterTransaction | 0.1 天 |
| `annotation/sql.rs` | @Sql + @SqlGroup + SqlScriptsTestExecutionListener | 0.5 天 |
| `annotation/timed.rs` | @Timed | 0.25 天 |
| `annotation/repeat.rs` | @Repeat | 0.25 天 |
| `annotation/profile_value_source.rs` | @ProfileValueSourceConfiguration | 0.25 天 |
| `annotation/if_profile_value.rs` | @IfProfileValue | 0.25 天 |
| `annotation/expected_exception.rs` | @ExpectedException（7.0 已 deprecated，可选） | 0.1 天 |
| `annotation/test_annotation.rs` | @TestAnnotation 标记 trait | 0.25 天 |
| `annotation/test_managed_resource.rs` | 资源管理标记 | 0.25 天 |
| `macros/src/vernal_test.rs` | vernal-test 专用宏（`#[vernal_test]` / `#[with_context]` / `#[mock_bean]`） | 1 天 |

**验收**：每个宏可展开为完整 trait impl；与 S1 集成测试通过

---

### S3 — TestExecutionListener 体系（预估 2 天）

**范围**：trait + 内置实现

| 文件 | Java 接口/类 | 工作量 |
|------|-------------|--------|
| `listener/test_execution_listener.rs` | TestExecutionListener trait + 7 个回调事件 | 0.5 天 |
| `listener/test_execution_listeners.rs` | @TestExecutionListeners + MergeMode + default 列表 | 0.5 天 |
| `listener/transactional_test_execution_listener.rs` | TransactionalTestExecutionListener | 0.5 天 |
| `listener/sql_scripts_test_execution_listener.rs` | SqlScriptsTestExecutionListener | 0.5 天 |
| `listener/sql_scripts_registrar.rs` | SqlScriptsTestExecutionListener 配套 | 0.25 天 |
| `listener/dependency_injection_test_execution_listener.rs` | DependencyInjectionTestExecutionListener | 0.5 天 |
| `listener/dirties_context_test_execution_listener.rs` | DirtiesContextTestExecutionListener | 0.5 天 |
| `listener/event_publishing_test_execution_listener.rs` | EventPublishingTestExecutionListener | 0.25 天 |
| `listener/application_events_test_execution_listener.rs` | ApplicationEventsTestExecutionListener | 0.25 天 |
| `listener/bean_overriding_test_execution_listener.rs` | BeanOverrideTestExecutionListener | 0.25 天 |
| `listener/aot_test_execution_listener.rs` | AotTestExecutionListener | 0.25 天 |
| `listener/common_caches_test_execution_listener.rs` | CommonCachesTestExecutionListener | 0.25 天 |
| `event/before_test_class_event.rs` | BeforeTestClassEvent | 0.1 天 |
| `event/after_test_class_event.rs` | AfterTestClassEvent | 0.1 天 |
| `event/before_test_method_event.rs` | BeforeTestMethodEvent | 0.1 天 |
| `event/after_test_method_event.rs` | AfterTestMethodEvent | 0.1 天 |
| `event/before_test_execution_event.rs` | BeforeTestExecutionEvent | 0.1 天 |
| `event/after_test_execution_event.rs` | AfterTestExecutionEvent | 0.1 天 |

**验收**：12 个 Listener 可注册；回调顺序测试；事件类型对得上

---

### S4 — ContextLoader 体系（预估 2 天）

**范围**：ContextLoader / SmartContextLoader trait + 实现

| 文件 | Java 接口/类 | 工作量 |
|------|-------------|--------|
| `context/loader/context_loader.rs` | ContextLoader trait | 0.25 天 |
| `context/loader/smart_context_loader.rs` | SmartContextLoader trait | 0.25 天 |
| `context/loader/abstract_context_loader.rs` | AbstractContextLoader（骨架） | 0.25 天 |
| `context/loader/abstract_generic_context_loader.rs` | AbstractGenericContextLoader | 0.5 天 |
| `context/loader/abstract_generice_web_context_loader.rs` | AbstractGenericWebContextLoader | 0.5 天 |
| `context/loader/abstract_delegating_smart_context_loader.rs` | AbstractDelegatingSmartContextLoader | 0.5 天 |
| `context/loader/annotation_config_context_loader.rs` | AnnotationConfigContextLoader | 0.5 天 |
| `context/loader/annotation_config_context_loader_utils.rs` | AnnotationConfigContextLoaderUtils | 0.25 天 |
| `context/loader/annotation_config_web_context_loader.rs` | AnnotationConfigWebContextLoader | 0.5 天 |
| `context/loader/generic_context_loader.rs` | GenericContextLoader（XML → JSON/YAML）| 0.5 天 |
| `context/loader/generic_xml_context_loader.rs` | GenericXmlContextLoader | 0.5 天 |
| `context/loader/aot_context_loader.rs` | AotContextLoader | 0.5 天 |
| `context/loader/context_loader_utils.rs` | ContextLoaderUtils | 0.25 天 |

**验收**：AnnotationConfig / Generic / Aot 三种加载路径 e2e 通过

---

### S5 — ContextCache + 配置体系（预估 2 天）

**范围**：ContextCache / MergedContextConfiguration / ContextHierarchy / ContextConfigurationAttributes

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `context/cache/context_cache.rs` | ContextCache trait | 0.25 天 |
| `context/cache/default_context_cache.rs` | DefaultContextCache（基于 `moka`） | 0.5 天 |
| `context/cache/cache_aware_context_loader_delegate.rs` | CacheAwareContextLoaderDelegate | 0.5 天 |
| `context/cache/context_cache_utils.rs` | ContextCacheUtils | 0.25 天 |
| `context/merged_context_configuration.rs` | MergedContextConfiguration | 0.5 天 |
| `context/context_configuration_attributes.rs` | ContextConfigurationAttributes | 0.25 天 |
| `context/context_hierarchy.rs` | @ContextHierarchy | 0.25 天 |
| `context/context_customizer.rs` | ContextCustomizer trait | 0.25 天 |
| `context/context_customizer_factory.rs` | ContextCustomizerFactory trait | 0.25 天 |
| `context/context_customizer_factories.rs` | ContextCustomizerFactories | 0.25 天 |

**验收**：缓存命中/失效；层级上下文父子关系；自定义器串联

---

### S6 — 事务测试集成（预估 1.5 天）

**范围**：依赖 `vernal-tx`，提供完整事务测试能力

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `transaction/transactional_test_execution_listener.rs` | TransactionalTestExecutionListener（已在 S3 占位，正式实现）| 0.5 天 |
| `transaction/transaction_assert.rs` | TransactionAssert | 0.25 天 |
| `transaction/test_transaction.rs` | TestTransaction | 0.5 天 |
| `transaction/transactional_test_util.rs` | TransactionalTestUtil | 0.25 天 |

**验收**：`@Commit` / `@Rollback` / `@BeforeTransaction` / `@AfterTransaction` 全场景

---

### S7 — Web 测试（预估 2.5 天）

**范围**：依赖 `vernal-web-testkit`，提供 MockMvc / WebTestClient / RestTestClient

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `web/model_and_view_assert.rs` | ModelAndViewAssert | 0.25 天 |
| `web/model_and_view_assertions.rs` | ModelAndViewAssertions | 0.25 天 |
| `web/servlet/mock_mvc.rs` | MockMvc | 0.5 天 |
| `web/servlet/mock_mvc_builder.rs` | MockMvcBuilder + StandaloneMockMvcBuilder | 0.5 天 |
| `web/servlet/mock_mvc_result.rs` | DefaultMockMvcBuilder + MockMvcBuilders | 0.5 天 |
| `web/servlet/request_builder.rs` | MockHttpServletRequestBuilder | 0.25 天 |
| `web/servlet/result_actions.rs` | ResultActions + DefaultResultActions | 0.5 天 |
| `web/servlet/result/matchers.rs` | StatusResultMatchers / HeaderResultMatchers / ContentResultMatchers / …（7 个 Matcher） | 1 天 |
| `web/servlet/result/print_handler.rs` | MockMvcResultHandlers | 0.25 天 |
| `web/reactive/web_test_client.rs` | WebTestClient | 0.5 天 |
| `web/client/rest_test_client.rs` | RestTestClient | 0.5 天 |

**验收**：发送 GET / POST 请求并断言状态/头/体；与 vernal-actix-web / vernal-axum 集成

---

### S8 — Mock 对象体系（预估 5 天）

**范围**：`org.springframework.mock.*` 包 ~80 类

| 子包 | 类数 | 工作量 |
|------|------|--------|
| `mock/env/` | MockEnvironment / MockPropertySource / MockBeanFactory 等 5 类 | 0.5 天 |
| `mock/http/` | MockHttpServletRequest / MockHttpServletResponse / MockHttpSession 等 8 类 | 1 天 |
| `mock/http/client/` | MockClientHttpRequest / MockClientHttpResponse 等 4 类 | 0.5 天 |
| `mock/http/server/` | MockServerHttpRequest / MockServerHttpResponse 等 4 类 | 0.5 天 |
| `mock/web/` | MockHttpSession + MockRequestDispatcher + MockServletContext 等 10 类 | 1.5 天 |
| `mock/web/reactive/` | MockServerHttpRequest + MockServerWebExchange 等 4 类 | 0.5 天 |
| `mock/web/server/` | MockWebServiceConnection 等 3 类 | 0.5 天 |
| `mock/jndi/` | MockInitialContextFactory 等 3 类（可选） | 0.25 天 |

**验收**：每个 Mock 类至少 1 单元测试；MockMvc 使用 Mock 请求/响应跑通

---

### S9 — JUnit 风格扩展（预估 1 天）

**范围**：Rust 测试函数到 TestContext 的桥接（不依赖 JUnit，参考 `test-context` crate 思想）

| 文件 | 说明 | 工作量 |
|------|------|--------|
| `junit/vernal_extension.rs` | `VernalExtension` 类似 JUnit Jupiter Extension | 0.25 天 |
| `junit/spring_vernal_config.rs` | @SpringJUnitConfig 等价宏 | 0.25 天 |
| `junit/vernal_runner.rs` | 类 JUnit Runner 的 trait | 0.25 天 |
| `junit/web_vernal_config.rs` | @WebMvcTest 等价宏 | 0.25 天 |

**验收**：`#[vernal_test]` 函数能自动加载上下文 + 注入依赖 + 跑事务回滚

---

### S10 — 工具类（预估 1 天）

| 文件 | Java 类 | 工作量 |
|------|---------|--------|
| `util/reflection_test_utils.rs` | ReflectionTestUtils | 0.5 天 |
| `util/aop_test_utils.rs` | AopTestUtils | 0.25 天 |
| `util/assertion_errors.rs` | AssertionErrors | 0.1 天 |
| `util/annotation_test_utils.rs` | AnnotationTestUtils | 0.25 天 |
| `util/test_bean.rs` | TestBean | 0.1 天 |
| `util/test_retention.rs` | @Retention 辅助 | 0.1 天 |

**验收**：通过反射工具访问私有字段；AOP 目标对象解包

---

### S11 — Vernal 集成（预估 1.5 天）

**范围**：与 vernal-context / vernal-beans / vernal-tx 集成

| 文件 | 说明 | 工作量 |
|------|------|--------|
| `context/vernal_test_context.rs` | 持有 vernal-context::ApplicationContext | 0.5 天 |
| `listener/vernal_di_listener.rs` | 依赖注入监听器（读 vernal-beans 容器） | 0.5 天 |
| `transaction/vernal_tx_adapter.rs` | 桥接 vernal-tx | 0.25 天 |
| `macros/src/with_context.rs` | `#[with_context]` 过程宏 | 0.25 天 |

**验收**：在真实 vernal 容器上跑 `@VernalTest` 测试，Bean 注入 + 事务回滚正常

---

### S12 — 文档 + 中文注释收尾（预估 1 天）

- 所有文件头部中文 doc 注释（对应 Java 类全限定名 + 核心职责）
- 方法级中文注释从 Java 源码同步翻译
- lib.rs 中文模块文档
- README 更新
- 示例：`examples/integration_basic.rs` / `examples/web_mock_mvc.rs`

---

## 四、工程规范

### 命名规则

| 项目 | 规范 |
|------|------|
| 目录/文件名 | snake_case（`test_context.rs`、`context_loader.rs`）|
| 类型名 | PascalCase（`TestContext`、`ContextLoader`）|
| 方法名 | snake_case（`get_test_class`、`mark_dirty`）|
| 注解 | PascalCase 过程宏属性（`#[VernalTest]` / `#[WithContext]`）|
| 枚举值 | SCREAMING_SNAKE（`MergeMode::All` 对应 `MERGE_MODE_ALL`）|

### 文件结构规范

```rust
//! 对应 Java 类：org.springframework.test.context.TestContext
//!
//! 测试上下文：封装单个测试方法执行所需的所有上下文信息。
//! 是 Spring TestContext 框架的核心接口，对所有测试框架中立。

use std::any::Any;

/// 测试上下文 trait。
///
/// 对应 Spring 的 `TestContext` 接口。提供：
/// - 当前测试类/方法/实例的访问
/// - ApplicationContext 的获取与缓存
/// - 测试属性的 AttributeAccessor 能力
pub trait TestContext: AttributeAccessor {
    /// 获取当前测试类。
    fn test_class(&self) -> &dyn Any;
    
    /// 获取当前测试方法。
    fn test_method(&self) -> Option<&dyn Any>;
    
    /// 获取（或懒加载）ApplicationContext。
    fn application_context(&self) -> &dyn Any;
    
    /// 标记上下文为脏，触发缓存清除。
    fn mark_dirty(&self, mode: Option<HierarchyMode>);
}
```

### 测试规范

- 每个公开 trait 至少 1 个 mock 实现测试
- TestExecutionListener 注册/回调顺序测试
- 完整 e2e：`#[vernal_test] fn my_test() { ... }` 加载容器 + 注入 Bean + 事务回滚
- Web 测试：MockMvc + WebTestClient 各 5+ 测试

---

## 五、复用 Rust 测试生态（vernal-test 不重新造轮子）

| 语义 | Rust crate | 用途 |
|------|------------|------|
| **TestContext（测试上下文管理）** | [`test-context`](https://crates.io/crates/test-context) | 提供测试间共享的 setup/teardown 上下文，对应 Spring 的 TestContext |
| **Mock 对象** | [`mockall`](https://crates.io/crates/mockall) | 自动 Mock 生成，对应 Spring 的 Mockito 集成 |
| **参数化测试** | [`rstest`](https://crates.io/crates/rstest) | 夹具 + 参数化，对应 Spring 的 @ParameterizedTest |
| **用例表驱动** | [`test-case`](https://crates.io/crates/test-case) | 单测用例表，对应 Spring 的 @ValueSource / @CsvSource |
| **HTTP Mock** | [`wiremock`](https://crates.io/crates/wiremock) / [`mockito`](https://crates.io/crates/mockito) | HTTP 服务端 Mock，对应 Spring 的 MockRestServiceServer |
| **HTTP 断言** | [`axum-test`](https://crates.io/crates/axum-test) | axum 应用 e2e 测试，对应 Spring 的 WebTestClient |
| **异步测试** | [`tokio-test`](https://crates.io/crates/tokio-test) | tokio runtime 工具，对应 Spring 的 @Async 测试 |
| **属性测试** | [`proptest`](https://crates.io/crates/proptest) | 属性化测试，对应 Spring 的 jqwik 集成（可选） |
| **断言增强** | [`pretty_assertions`](https://crates.io/crates/pretty_assertions) | 彩色 diff 断言，对应 Spring 的 AssertJ |
| **Mock 校验** | [`mockall`](https://crates.io/crates/mockall) 的 `expectation` | 调用次数/参数验证，对应 Mockito 的 verify |

> vernal-test 的定位：**Spring-style 测试上下文管理** + **注解驱动测试元数据** +
> **与 vernal 框架的集成**。底层机制（Mock、参数化、属性测试、HTTP Mock）复用
> Rust 生态，不重新实现。

---

## 六、风险评估

| 风险 | 影响 | 对策 |
|------|------|------|
| 注解 → 过程宏映射不完整 | 元数据丢失 | 优先实现 `@ContextConfiguration` / `@TestPropertySource` / `@DirtiesContext` 三大件 |
| TestContext 生命周期复杂 | 多 Listener 串联困难 | 用 trait + 显式注册表，避免 ThreadLocal 黑魔法 |
| JUnit 集成缺位 | 用户需自行桥接 | 提供 `VernalExtension` trait + `#[vernal_test]` 宏，对 JUnit Jupiter 语义对齐 |
| Mock 对象数量大 | 工作量超预期 | 分批实现，S8 单独预留 5 天 |
| 与 vernal-context 集成紧耦合 | 早期无法独立测试 | 先用 trait 抽象，集成测试放最后 |

---

## 七、预期成果

完成全部 12 个阶段后，vernal-test 将成为 **Rust 生态最完整的 Spring-style 测试框架**：

| 维度 | 当前 | 完成后 |
|------|------|--------|
| 文件数 | 2 | **150+** |
| 代码行数 | 25 | **8000+** |
| TestContext 体系 | 骨架 | **完整对齐 Spring 7.x** |
| 注解 | 0 | **20+ 过程宏** |
| TestExecutionListener | 0 | **12+ 内置实现** |
| ContextLoader | 0 | **5 种加载策略** |
| Web 测试 | 0 | **MockMvc + WebTestClient + RestTestClient** |
| Mock 对象 | 0 | **~80 类** |
| 与 Spring 7.x 对标度 | 2% | **85%+** |
| 与 vernal 框架集成 | 无 | **完整（context + beans + tx）** |
