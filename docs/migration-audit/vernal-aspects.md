<!-- migration-doc: authority=authoritative canonical=../迁移验收规范.md -->
# vernal-aspects 迁移事实审计

> 本文由 `scripts/audit_migration_docs.py` 根据源码生成；禁止手工修改统计和对象行。
> Spring 基线提交：`9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82`；路径规则：保留末 `2` 层包目录。

<!-- current-migration-contract-start -->
## 当前迁移规范执行口径

| 规范项 | 本模块强制要求 |
|---|---|
| 来源基线 | `9e8cea3ef8ae02efb7956b071cd7bbef7c22cb82` |
| Java 对象边界 | 9 个 class/interface/enum/record；`package-info.java` 不计入 |
| 目录算法 | 去掉组织和模块根包，保留末 2 层包目录 |
| 文件边界 | 一个 Java 对象对应一个 snake_case `.rs` 文件；内部类/Builder 可随主对象 |
| 模块文件 | `lib.rs`/`mod.rs` 只允许模块文档、声明和显式重导出 |
| 完成状态 | 仅 `IMPLEMENTED`、`DEPENDENCY_REUSED`、`PLATFORM_NA` 计入完成 |
| 未完成状态 | `MISSING`、`MISPLACED`、`STUB`、`PARTIAL`、`UNVERIFIED` |
| 注释与测试 | 中文 Java 来源注释；正常、失败、边界和生命周期语义测试 |

本文件顶部事实区始终按当前源码重新生成；下方历史设计附录不得覆盖这里的对象数量、路径、状态或证据。
<!-- current-migration-contract-end -->

## 汇总

| 指标 | 数量 |
|---|---:|
| Java 业务对象 | 9 |
| 已处理（严格三类） | 0 |
| `DEPENDENCY_REUSED` | 0 |
| `IMPLEMENTED` | 0 |
| `MISPLACED` | 2 |
| `MISSING` | 6 |
| `PARTIAL` | 0 |
| `PLATFORM_NA` | 0 |
| `STUB` | 0 |
| `UNVERIFIED` | 1 |

## 结构红线

> 下列既存问题属于未完成证据。本报告只登记，不在文档治理任务中修改源码。

- 单文件多个公开对象位于 `transactional_aspect.rs`：`Propagation`、`Isolation`、`TransactionConfig`、`TransactionalAspect`
- 单文件多个公开对象位于 `async_aspect.rs`：`AsyncConfig`、`AsyncAspect`
- 单文件多个公开对象位于 `cacheable_aspect.rs`：`CacheOperation`、`CacheConfig`、`CacheableAspect`
- 单文件多个公开对象位于 `scheduled_aspect.rs`：`ScheduleConfig`、`ScheduledAspect`
- 单文件多个公开对象位于 `weaver/pointcut_matcher.rs`：`MethodMetadata`、`PointcutMatcher`
- 类型定义位于 `support/mod.rs`
- 单文件多个公开对象位于 `beans/factory/aspectj/configurable_object.rs`：`ConfigurableObject`、`ConfigurableObjectWithLifetime`
- 单文件多个公开对象位于 `beans/factory/aspectj/abstract_dependency_injection_aspect.rs`：`MethodMetadata`、`AbstractDependencyInjectionAspect`
- 单文件多个公开对象位于 `scheduling/aspectj/async_task_executor.rs`：`AsyncTaskResult`、`AsyncTaskExecutor`、`DefaultAsyncTaskExecutor`
- 单文件多个公开对象位于 `scheduling/aspectj/abstract_async_execution_aspect.rs`：`MethodMetadata`、`AsyncExecutionResult`、`AbstractAsyncExecutionAspect`
- 单文件多个公开对象位于 `scheduling/aspectj/async_uncaught_exception_handler.rs`：`AsyncUncaughtExceptionHandler`、`DefaultAsyncUncaughtExceptionHandler`
- 单文件多个公开对象位于 `cache/aspectj/cache_operation.rs`：`CacheOperation`、`CacheOperationMetadata`
- 单文件多个公开对象位于 `cache/aspectj/cache_aspect_support.rs`：`CacheResult`、`CacheManager`、`Cache`、`CacheOperationInvoker`、`CacheAspectSupport`
- 单文件多个公开对象位于 `cache/aspectj/cache_operation_source.rs`：`MethodMetadata`、`CacheOperationSource`、`AnnotationCacheOperationSource`
- 单文件多个公开对象位于 `transaction/aspectj/transaction_aspect_support.rs`：`TransactionResult`、`TransactionError`、`TransactionManager`、`NoOpTransactionManager`、`SuspendedTransactionInfo`、`TransactionAspectSupport`
- 单文件多个公开对象位于 `transaction/aspectj/transaction_attribute_source.rs`：`MethodMetadata`、`TransactionAttributeSource`、`AnnotationTransactionAttributeSource`

## 逐对象台账

| Java FQN | Java 相对路径 | 预期 Rust 路径 | 当前 Rust 路径 | 状态 | 证据 |
|---|---|---|---|---|---|
| `org.springframework.beans.factory.aspectj.ConfigurableObject` | `beans/factory/aspectj/ConfigurableObject.java` | `factory/aspectj/configurable_object.rs` | `beans/factory/aspectj/configurable_object.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.cache.aspectj.AnyThrow` | `cache/aspectj/AnyThrow.java` | `cache/aspectj/any_throw.rs` | `cache/aspectj/any_throw.rs` | `UNVERIFIED` | 缺少中文 Java 来源注释 |
| `org.springframework.cache.aspectj.AspectJCachingConfiguration` | `cache/aspectj/AspectJCachingConfiguration.java` | `cache/aspectj/aspect_j_caching_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.cache.aspectj.AspectJJCacheConfiguration` | `cache/aspectj/AspectJJCacheConfiguration.java` | `cache/aspectj/aspect_jj_cache_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.aspectj.EnableSpringConfigured` | `context/annotation/aspectj/EnableSpringConfigured.java` | `annotation/aspectj/enable_spring_configured.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.context.annotation.aspectj.SpringConfiguredConfiguration` | `context/annotation/aspectj/SpringConfiguredConfiguration.java` | `annotation/aspectj/spring_configured_configuration.rs` | `context/annotation/aspectj/spring_configured_configuration.rs` | `MISPLACED` | 文件名存在，但未位于保留末两层包目录计算出的路径 |
| `org.springframework.scheduling.aspectj.AspectJAsyncConfiguration` | `scheduling/aspectj/AspectJAsyncConfiguration.java` | `scheduling/aspectj/aspect_j_async_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.aspectj.AspectJJtaTransactionManagementConfiguration` | `transaction/aspectj/AspectJJtaTransactionManagementConfiguration.java` | `transaction/aspectj/aspect_j_jta_transaction_management_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
| `org.springframework.transaction.aspectj.AspectJTransactionManagementConfiguration` | `transaction/aspectj/AspectJTransactionManagementConfiguration.java` | `transaction/aspectj/aspect_j_transaction_management_configuration.rs` | `—` | `MISSING` | 未找到同名 Rust 对象文件 |
