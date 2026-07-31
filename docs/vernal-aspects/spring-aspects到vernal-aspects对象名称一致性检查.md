<!-- migration-doc: authority=historical canonical=对象名称一致性检查.md -->

> 迁移文档治理：本文级别为 **historical**，历史基线提交 `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`。正文不得作为当前验收结论；以 [对象名称一致性检查.md](对象名称一致性检查.md) 为准。

<!-- restored-detail-from-head: dd20300d16a09200bd8a379ff14db1e2da99b67c -->
# spring-aspects → vernal-aspects 对象名称一致性检查

> 检查时间：2026-07-27
> 基线：Spring Framework 7.0.8 spring-aspects（21 个 Java 类 + 1 资源）
> vernal-aspects：5 文件（lib.rs + 4 占位 aspect.rs）/ ~250 行 / 0 测试
> 底盘：aspect-rs（编译期织入） + vernal-aop（运行期拦截）
> **目录命名 100% 镜像 Spring 5 个 aspectj 子包路径**

## 命名对齐原则（强制要求）

1. **目录命名 100% 镜像 Spring 包路径**：Spring `transaction.aspectj` → vernal `transaction/aspectj`，依此类推
2. **类型名 100% 与 Spring 一致**：保留 `AspectJTransactionManagementConfiguration` 全名（**不**简化为 `Config`）
3. **文件名去 `.aj` 后缀**：Spring `AbstractTransactionAspect.aj` → vernal `abstract_transaction_aspect.rs`
4. **包路径多层级镜像**：`org.springframework.beans.factory.aspectj` → `beans/factory/aspectj/`，保留中间 `factory` 层

## 统计汇总

| 维度 | Spring | vernal | 说明 |
|------|--------|--------|------|
| Java 类总数 | 21 | — | 排除 package-info + aop.xml |
| Rust 文件总数 | — | 5（占位）→ 目标 46+ | 排除 mod.rs / lib.rs |
| **类型名完全匹配** | **4** | **4** | 4 个核心切面已对齐（仅骨架） |
| **目录镜像** | — | **0/5** | 现状 4 个 aspect.rs 都在 `src/` 根下，未镜像包路径 |
| Spring 有 vernal 没有 | 17 | — | 见下文分类 |
| vernal 有 Spring 没有 | — | 5 | Vernal 特有（weaver + support）|

---

## 一、目录路径镜像（强制规则）

### 1.1 5 个 Spring 包路径 → 5 个 vernal 模块路径

| Spring 包 | vernal-aspects 目录 | 现状 | 计划 |
|---|---|---|---|
| `org.springframework.transaction.aspectj` | `src/transaction/aspectj/` | ⬜ 当前 `src/transactional_aspect.rs` 在 `src/` 根下 | ⬜ 待迁移到 `transaction/aspectj/` |
| `org.springframework.cache.aspectj` | `src/cache/aspectj/` | ⬜ 当前 `src/cacheable_aspect.rs` 在 `src/` 根下 | ⬜ 待迁移到 `cache/aspectj/` |
| `org.springframework.scheduling.aspectj` | `src/scheduling/aspectj/` | ⬜ 当前 `src/async_aspect.rs` 在 `src/` 根下 | ⬜ 待迁移到 `scheduling/aspectj/` |
| `org.springframework.beans.factory.aspectj` | `src/beans/factory/aspectj/` | ⬜ 当前无对应 | ⬜ 待创建 |
| `org.springframework.context.annotation.aspectj` | `src/context/annotation/aspectj/` | ⬜ 当前无对应 | ⬜ 待创建 |

### 1.2 镜像规则要点

- **包路径的最后两层目录**直接作为 vernal 模块目录：Spring `X.aspectj` → vernal `X/aspectj/`
- **3 层深度的包**保留全部层级：Spring `X.Y.aspectj` → vernal `X/Y/aspectj/`（如 `beans/factory/aspectj/` 3 层）
- **不抽取业务词根**：保留 Spring 的全小写路径名，不做"语义化"改写
- **同包内所有类放同一目录**：事务切面 5 类 + 6 个支撑类全部在 `transaction/aspectj/` 下

---

## 二、已对齐的 4 个核心切面（仅类型名层面 ✅）

以下 4 个对象的**类型名**与 Spring 一致，但**目录路径**尚未镜像。

### 2.1 现状对照

| Spring Java 类 | vernal 当前文件（错误） | vernal 目标文件（正确） | 类型名一致性 | 路径一致性 |
|---|---|---|---|---|
| `AbstractTransactionAspect` | `src/transactional_aspect.rs` | `src/transaction/aspectj/abstract_transaction_aspect.rs` | ✅ `AbstractTransactionAspect` | ⬜ 未镜像 |
| `AbstractCacheAspect` | `src/cacheable_aspect.rs` | `src/cache/aspectj/abstract_cache_aspect.rs` | ✅ `AbstractCacheAspect` | ⬜ 未镜像 |
| `AbstractAsyncExecutionAspect` | `src/async_aspect.rs` | `src/scheduling/aspectj/abstract_async_execution_aspect.rs` | ✅ `AbstractAsyncExecutionAspect` | ⬜ 未镜像 |
| （无 Spring 对偶）| `src/scheduled_aspect.rs`（`ScheduledAspect`）| （无对应目录，保留在 `src/` 根）| 🆕 Rust 特有 | 🆕 Rust 特有 |

### 2.2 现状（vernal-aspects 当前 4 个 aspect.rs）

```rust
// src/transactional_aspect.rs (❌ 路径未镜像 / ✅ 类型名一致)
pub struct TransactionalAspect { config: TransactionConfig }
//                       ^^^^^^ 注意：当前用的是 TransactionalAspect，
//                              Spring 是 AbstractTransactionAspect

// src/cacheable_aspect.rs (❌ 路径未镜像 / ✅ 类型名一致)
pub struct CacheableAspect { config: CacheConfig }
//                       ^^^^^^ Spring 是 AbstractCacheAspect

// src/async_aspect.rs (❌ 路径未镜像 / ✅ 类型名一致)
pub struct AsyncAspect { config: AsyncConfig }
//                   ^^^^^^ Spring 是 AbstractAsyncExecutionAspect

// src/scheduled_aspect.rs (🆕 无 Spring 对偶)
pub struct ScheduledAspect { config: ScheduleConfig }
```

**关键问题**：
1. **路径全错**：4 个 aspect.rs 都在 `src/` 根下，应分别在 `transaction/aspectj/`、`cache/aspectj/`、`scheduling/aspectj/` 下
2. **类型名不一致**：当前用的是 `TransactionalAspect` / `CacheableAspect` / `AsyncAspect`（无 `Abstract` 前缀），应改为 `AbstractTransactionAspect` / `AbstractCacheAspect` / `AbstractAsyncExecutionAspect`

---

## 三、Spring 有但 vernal 还没有的 17 个类

### 3.1 事务切面（5 个类，`transaction/aspectj/` 模块）

| Spring 类 | 目标文件 | 一致性 |
|---|---|---|
| `AbstractTransactionAspect` | `transaction/aspectj/abstract_transaction_aspect.rs` | ⬜ 类型名待改（当前 `TransactionalAspect` → `AbstractTransactionAspect`） |
| `AnnotationTransactionAspect` | `transaction/aspectj/annotation_transaction_aspect.rs` | ⬜ 待实现 |
| `JtaAnnotationTransactionAspect` | `transaction/aspectj/jta_annotation_transaction_aspect.rs` | ⬜ 待实现 |
| `AspectJTransactionManagementConfiguration` | `transaction/aspectj/aspectj_transaction_management_configuration.rs` | ⬜ 待实现 |
| `AspectJJtaTransactionManagementConfiguration` | `transaction/aspectj/aspectj_jta_transaction_management_configuration.rs` | ⬜ 待实现 |

### 3.2 缓存切面（5 个类，`cache/aspectj/` 模块）

| Spring 类 | 目标文件 | 一致性 |
|---|---|---|
| `AbstractCacheAspect` | `cache/aspectj/abstract_cache_aspect.rs` | ⬜ 类型名待改 |
| `AnnotationCacheAspect` | `cache/aspectj/annotation_cache_aspect.rs` | ⬜ 待实现 |
| `JCacheCacheAspect` | `cache/aspectj/jcache_cache_aspect.rs` | ⬜ 待实现 |
| `AspectJCachingConfiguration` | `cache/aspectj/aspectj_caching_configuration.rs` | ⬜ 待实现 |
| `AspectJJCacheConfiguration` | `cache/aspectj/aspectj_jcache_configuration.rs` | ⬜ 待实现 |

### 3.3 异步切面（3 个类，`scheduling/aspectj/` 模块）

| Spring 类 | 目标文件 | 一致性 |
|---|---|---|
| `AbstractAsyncExecutionAspect` | `scheduling/aspectj/abstract_async_execution_aspect.rs` | ⬜ 类型名待改（当前 `AsyncAspect` → `AbstractAsyncExecutionAspect`） |
| `AnnotationAsyncExecutionAspect` | `scheduling/aspectj/annotation_async_execution_aspect.rs` | ⬜ 待实现 |
| `AspectJAsyncConfiguration` | `scheduling/aspectj/aspectj_async_configuration.rs` | ⬜ 待实现 |

### 3.4 可配置对象切面（5 个类 + 1 嵌套接口，`beans/factory/aspectj/` 模块）

| Spring 类 | 目标文件 | 一致性 |
|---|---|---|
| `AbstractDependencyInjectionAspect` | `beans/factory/aspectj/abstract_dependency_injection_aspect.rs` | ⬜ 待实现 |
| `AbstractInterfaceDrivenDependencyInjectionAspect` | `beans/factory/aspectj/abstract_interface_driven_dependency_injection_aspect.rs` | ⬜ 待实现 |
| `AnnotationBeanConfigurerAspect` | `beans/factory/aspectj/annotation_bean_configurer_aspect.rs` | ⬜ 待实现 |
| `GenericInterfaceDrivenDependencyInjectionAspect<I>` | `beans/factory/aspectj/generic_interface_driven_dependency_injection_aspect.rs` | ⬜ 待实现 |
| `ConfigurableObject` | `beans/factory/aspectj/configurable_object.rs` | ⬜ 待实现 |
| `ConfigurableDeserializationSupport`（嵌套 static interface） | `beans/factory/aspectj/configurable_deserialization_support.rs` | ⬜ 待实现（独立文件，归属 AbstractInterfaceDrivenDependencyInjectionAspect） |

### 3.5 Spring Configured 启用（2 个类，`context/annotation/aspectj/` 模块）

| Spring 类 | 目标文件 | 一致性 |
|---|---|---|
| `EnableSpringConfigured` | `context/annotation/aspectj/enable_spring_configured.rs` | ⬜ 待实现 |
| `SpringConfiguredConfiguration` | `context/annotation/aspectj/spring_configured_configuration.rs` | ⬜ 待实现 |

### 3.6 Java 生态特有，不迁移（1 个资源）

| Spring 资源 | 说明 | vernal 处置 |
|---|---|---|
| `META-INF/aop.xml` | AspectJ LTW 配置清单 | 🆕 用 `weaver/aop_xml.rs` + `build.rs` 替代 |

---

## 四、命名一致性规则（强制）

### 4.1 类型名（PascalCase，**100% 与 Spring 一致**，不简化）

| Java 类 | Rust 类型 | 一致性 |
|---|---|---|
| `AbstractTransactionAspect` | `AbstractTransactionAspect`（struct）| ⬜ 当前错用 `TransactionalAspect` |
| `AbstractCacheAspect` | `AbstractCacheAspect`（struct）| ⬜ 当前错用 `CacheableAspect` |
| `AbstractAsyncExecutionAspect` | `AbstractAsyncExecutionAspect`（struct）| ⬜ 当前错用 `AsyncAspect` |
| `AnnotationTransactionAspect` | `AnnotationTransactionAspect`（struct）| ⬜ 待实现 |
| `JtaAnnotationTransactionAspect` | `JtaAnnotationTransactionAspect`（struct）| ⬜ 待实现 |
| `AnnotationCacheAspect` | `AnnotationCacheAspect`（struct）| ⬜ 待实现 |
| `JCacheCacheAspect` | `JCacheCacheAspect`（struct）| ⬜ 待实现 |
| `AnnotationAsyncExecutionAspect` | `AnnotationAsyncExecutionAspect`（struct）| ⬜ 待实现 |
| `AnnotationBeanConfigurerAspect` | `AnnotationBeanConfigurerAspect`（struct）| ⬜ 待实现 |
| `GenericInterfaceDrivenDependencyInjectionAspect<I>` | `GenericInterfaceDrivenDependencyInjectionAspect<I>`（struct）| ⬜ 待实现 |
| `AbstractDependencyInjectionAspect` | `AbstractDependencyInjectionAspect`（trait）| ⬜ 待实现 |
| `AbstractInterfaceDrivenDependencyInjectionAspect` | `AbstractInterfaceDrivenDependencyInjectionAspect`（trait）| ⬜ 待实现 |
| `ConfigurableObject` | `ConfigurableObject`（trait）| ⬜ 待实现 |
| `ConfigurableDeserializationSupport` | `ConfigurableDeserializationSupport`（trait，嵌套接口独立文件）| ⬜ 待实现 |
| `Propagation` | `Propagation`（enum）| ⬜ 待实现 |
| `Isolation` | `Isolation`（enum）| ⬜ 待实现 |
| `AnyThrow` | `AnyThrow`（utility struct）| ⬜ 待实现 |

### 4.2 配置类命名（**保留 `Configuration` 全名，不简化为 `Config`**）

| Java 类 | Rust 类型 | 规则 |
|---|---|---|
| `AspectJTransactionManagementConfiguration` | `AspectJTransactionManagementConfiguration` | 保留全名 |
| `AspectJJtaTransactionManagementConfiguration` | `AspectJJtaTransactionManagementConfiguration` | 保留全名 |
| `AspectJCachingConfiguration` | `AspectJCachingConfiguration` | 保留全名 |
| `AspectJJCacheConfiguration` | `AspectJJCacheConfiguration` | 保留全名 |
| `AspectJAsyncConfiguration` | `AspectJAsyncConfiguration` | 保留全名 |
| `SpringConfiguredConfiguration` | `SpringConfiguredConfiguration` | 保留全名 |

### 4.3 文件名（snake_case，去 `.aj` 后缀）

| Spring 文件 | vernal 文件 | 规则 |
|---|---|---|
| `AbstractTransactionAspect.aj` | `abstract_transaction_aspect.rs` | 去 `.aj`，snake_case |
| `AnnotationTransactionAspect.aj` | `annotation_transaction_aspect.rs` | 同上 |
| `JtaAnnotationTransactionAspect.aj` | `jta_annotation_transaction_aspect.rs` | 同上 |
| `AspectJTransactionManagementConfiguration.java` | `aspectj_transaction_management_configuration.rs` | snake_case 转换 |
| `AspectJJtaTransactionManagementConfiguration.java` | `aspectj_jta_transaction_management_configuration.rs` | 同上 |
| `AbstractCacheAspect.aj` | `abstract_cache_aspect.rs` | 同上 |
| `AnnotationCacheAspect.aj` | `annotation_cache_aspect.rs` | 同上 |
| `JCacheCacheAspect.aj` | `jcache_cache_aspect.rs` | 同上 |
| `AspectJCachingConfiguration.java` | `aspectj_caching_configuration.rs` | 同上 |
| `AspectJJCacheConfiguration.java` | `aspectj_jcache_configuration.rs` | 同上 |
| `AnyThrow.java` | `any_throw.rs` | 全小写 snake_case |
| `AbstractAsyncExecutionAspect.aj` | `abstract_async_execution_aspect.rs` | 同上 |
| `AnnotationAsyncExecutionAspect.aj` | `annotation_async_execution_aspect.rs` | 同上 |
| `AspectJAsyncConfiguration.java` | `aspectj_async_configuration.rs` | 同上 |
| `AbstractDependencyInjectionAspect.aj` | `abstract_dependency_injection_aspect.rs` | 同上 |
| `AbstractInterfaceDrivenDependencyInjectionAspect.aj` | `abstract_interface_driven_dependency_injection_aspect.rs` | 同上 |
| `AnnotationBeanConfigurerAspect.aj` | `annotation_bean_configurer_aspect.rs` | 同上 |
| `GenericInterfaceDrivenDependencyInjectionAspect.aj` | `generic_interface_driven_dependency_injection_aspect.rs` | 同上 |
| `ConfigurableObject.java` | `configurable_object.rs` | 同上 |
| `ConfigurableDeserializationSupport.java`（嵌套接口） | `configurable_deserialization_support.rs` | 同上 |
| `EnableSpringConfigured.java` | `enable_spring_configured.rs` | 同上 |
| `SpringConfiguredConfiguration.java` | `spring_configured_configuration.rs` | 同上 |

### 4.4 方法名（snake_case）

| Java 方法 | Rust 方法 | 规则 |
|---|---|---|
| `invokeWithinTransaction` | `invoke_within_transaction` | camelCase → snake_case |
| `transactionalMethodExecution(Object)` | `transactional_method_execution(&self)` | 同上 |
| `setTransactionManager` | `set_transaction_manager` | 同上 |
| `getDefaultExecutor` | `get_default_executor` | 同上 |
| `determineAsyncExecutor` | `determine_async_executor` | 同上 |
| `doSubmit` | `do_submit` | 同上 |
| `getExecutorQualifier` | `get_executor_qualifier` | 同上 |
| `clearTransactionManagerCache` | `clear_transaction_manager_cache` | 同上 |
| `clearMetadataCache` | `clear_metadata_cache` | 同上 |
| `configureBean(Object)` | `configure_bean(&self, bean: &mut dyn Any)` | 同上 |
| `setBeanFactory(BeanFactory)` | `set_bean_factory(&mut self, bean_factory: Arc<dyn BeanFactory>)` | 同上 |
| `afterPropertiesSet()` | `after_properties_set(&mut self)` | 同上 |
| `aspectOf()` | `aspect_of() -> &Self` | 同上 |
| `pointcut executionOfAnyPublicMethodInAtTransactionalType()` | `fn execution_of_any_public_method_in_at_transactional_type(&self) -> Pointcut` | 同上 |

### 4.5 字段名（snake_case）

| Java 字段 | Rust 字段 |
|---|---|
| `propagation` | `propagation`（已一致） |
| `isolation` | `isolation`（已一致） |
| `readOnly` | `read_only` |
| `timeout` | `timeout_secs`（带单位后缀） |
| `rollbackFor` | `rollback_for` |
| `noRollbackFor` | `no_rollback_for` |
| `transactionManager` | `transaction_manager` |
| `cacheManager` | `cache_manager` |
| `cacheResolver` | `cache_resolver` |
| `keyGenerator` | `key_generator` |
| `errorHandler` | `error_handler` |
| `beforeInvocation` | `before_invocation` |
| `allEntries` | `all_entries` |
| `cacheName` | `cache_name` |
| `executorName` | `executor_name` |
| `fixedDelay` | `fixed_delay_ms`（带单位后缀） |
| `fixedRate` | `fixed_rate_ms` |
| `initialDelay` | `initial_delay_ms` |

---

## 五、目录结构一致性（强制 100% 镜像）

### 5.1 包路径 → 模块路径（最终强制映射）

| Spring 包 | vernal 模块 | 中间层保留 |
|---|---|---|
| `org.springframework.transaction.aspectj` | `transaction/aspectj/` | 1 层 |
| `org.springframework.cache.aspectj` | `cache/aspectj/` | 1 层 |
| `org.springframework.scheduling.aspectj` | `scheduling/aspectj/` | 1 层 |
| `org.springframework.beans.factory.aspectj` | `beans/factory/aspectj/` | **2 层**（保留 `factory` 中间包） |
| `org.springframework.context.annotation.aspectj` | `context/annotation/aspectj/` | **2 层**（保留 `annotation` 中间包） |
| （META-INF）| `weaver/` | vernal 特有 |
| （runtime 桥接）| `support/` | vernal 特有 |

**关键约束**：
- 5 个 aspectj 包路径**不做任何业务词根改写**（不再用 `transactional/`、`cacheable/`、`async_execution/`、`configurable/`、`spring_configured/`）
- 模块名严格保留 Spring 包名（包括 `aspectj` 子包作为单独目录）
- 接受 `beans/factory/aspectj/` 3 层深度的路径长度，换取 100% 镜像

---

## 六、业务逻辑一致性检查

### ✅ 已对齐（仅类型名层面）
- 4 个核心切面 struct 名一致（`TransactionalAspect` 对 Spring `AbstractTransactionAspect`）
- 4 个核心切面 `*Config` 字段名一致
- `Interceptor::intercept` 签名一致（已实现 `Interceptor` trait）

### 🔶 形态不同（待迁移时统一）
- Spring 的 `Interceptor` 是 Java interface；vernal 用 `vernal_aop::Interceptor` trait
- Spring 的 `TransactionAttribute` 是 interface；vernal 用 `TransactionConfig` struct（**待改名为 `TransactionAttribute`**）
- Spring 的 `Propagation` 是 enum；vernal 已有同名 enum，但未与 `TransactionConfig` 关联（**待迁移到 `transaction/aspectj/propagation.rs`**）
- Spring 的 `AspectJTransactionManagementConfiguration` 是 `@Configuration` 注解类；vernal 当前 `TransactionalAspect` 是 struct（**类型名错误**）

### ⬜ 未实现的语义
- 7 种 Propagation 行为分支
- 5 种 Isolation 行为分支
- `@Transactional` / `@Cacheable` / `@Async` 注解属性读取
- JTA 1.2 / JSR-107 双注解支持
- `@Configurable` 非托管对象 DI 注入
- 反序列化 `readResolve` reattach
- pre/post-construction 注入顺序
- 泛型 `configure(I)` 类型安全派发

---

## 七、结论

| 维度 | 结果 |
|------|------|
| 类型名称一致性 | **0/21 = 0%**（当前 4 个核心切面类型名错用，缺少 `Abstract` 前缀） |
| 文件名一致性 | **0/21 = 0%**（路径全错，未镜像 Spring 包路径） |
| **目录镜像** | **0/5 = 0%**（所有 aspect.rs 都在 `src/` 根下）|
| 业务逻辑一致性 | **0/21**（骨架层面，无语义实现） |

### 当前命名错误清单（必须修正）

| 序号 | 当前错误 | 应改为 |
|---|---|---|
| 1 | `src/transactional_aspect.rs` 中的 `pub struct TransactionalAspect` | `src/transaction/aspectj/abstract_transaction_aspect.rs` 中的 `pub struct AbstractTransactionAspect` |
| 2 | `src/cacheable_aspect.rs` 中的 `pub struct CacheableAspect` | `src/cache/aspectj/abstract_cache_aspect.rs` 中的 `pub struct AbstractCacheAspect` |
| 3 | `src/async_aspect.rs` 中的 `pub struct AsyncAspect` | `src/scheduling/aspectj/abstract_async_execution_aspect.rs` 中的 `pub struct AbstractAsyncExecutionAspect` |
| 4 | `src/transactional_aspect.rs` 中的 `pub struct TransactionConfig` | `src/transaction/aspectj/transaction_attribute.rs` 中的 `pub struct TransactionAttribute` |
| 5 | `src/scheduled_aspect.rs`（无 Spring 对偶）| 应迁移到 `src/scheduling/aspectj/` 下（与 `AbstractAsyncExecutionAspect` 同包）|
| 6 | `src/cacheable_aspect.rs` 中的 `pub struct CacheConfig` | 保留独立文件 `src/cache/aspectj/cache_config.rs`（Spring `Cache` 是接口，此处独立）|

### 与 `vernal-expression` 一致性检查的对比

| 维度 | vernal-expression | vernal-aspects |
|---|---|---|
| 完全匹配数 | 79/117（67.5%）| **0/21（0%）**（类型名 + 路径全错）|
| 业务逻辑一致性 | 核心 AST 节点 100% | **0%**（骨架阶段）|
| 差距原因 | 12 个 Support + 5 个运算符 + 7 个异常 | **目录未镜像 + 类型名错用 + 17 个 Java 切面 + 配置类 + 元数据源 + 织入机制** |

### 下一步行动

1. **P0（紧急）：重命名现有 4 个 aspect.rs**：类型名加 `Abstract` 前缀，迁移到对应 aspectj 子包目录
2. **P0（路线图 S1）**：实现织入机制层（7 文件），先建立目录骨架
3. **P0（路线图 S2）**：实现事务切面（11 文件），把 4 个核心切面中的 `AbstractTransactionAspect` 升级为完整实现
4. **P0（路线图 S3）**：实现缓存切面（14 文件），把 `AbstractCacheAspect` 升级为完整实现
5. **P0（路线图 S4）**：实现异步切面（7 文件），把 `AbstractAsyncExecutionAspect` 升级为完整实现
6. **P1（路线图 S5）**：实现可配置对象切面（11 文件）
7. **P1（路线图 S6）**：实现 Spring Configured 启用（2 文件 + proc-macro）

完成 P0 + P1 任务后：
- **类型名称一致性**：21/21 = **100%**
- **目录路径镜像**：5/5 = **100%**
- **业务逻辑一致性**：从 0% 提升到 **90%+**