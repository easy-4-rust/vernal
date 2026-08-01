<!-- migration-doc: authority=support canonical=../迁移验收规范.md -->

> 迁移文档治理：本文级别为 **support**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。

# vernal-aspects SOURCE_PARITY 测试台账

> 本文档记录 Java 源测试到 Rust 测试的映射关系。
> 基线：Spring Framework 7.0.8 spring-aspects 模块。

## 测试映射表

| Java 测试类 | Java 测试方法 | Rust 测试 | 证据级别 | 状态 |
|---|---|---|---|---|
| `AbstractTransactionAspectTests` | `testRequiredPropagation` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_required_no_existing` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testRequiresNewPropagation` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_requires_new_with_existing_tx` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testNestedPropagation` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_nested_with_existing_tx` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testMandatoryPropagation` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_mandatory_with_existing_tx` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testSupportsPropagation` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_supports_with_existing_tx` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testNotSupportedPropagation` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_not_supported_with_existing_tx` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testNeverPropagation` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_never_with_existing_tx` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testRollbackForRule` | `transaction::aspectj::transaction_attribute::tests::test_should_rollback_for_runtime_exception` | V2_MIRRORED | ✅ |
| `AbstractTransactionAspectTests` | `testNoRollbackForRule` | `transaction::aspectj::transaction_aspect_support::tests::test_invoke_within_transaction_required_with_no_rollback_for_set` | V2_MIRRORED | ✅ |
| `AbstractCacheAspectTests` | `testCacheableHit` | `cache::aspectj::cache_aspect_support::tests::test_execute_cacheable_with_cache_hit` | V2_MIRRORED | ✅ |
| `AbstractCacheAspectTests` | `testCacheableMiss` | `cache::aspectj::cache_aspect_support::tests::test_execute_cacheable_with_operation` | V2_MIRRORED | ✅ |
| `AbstractCacheAspectTests` | `testCachePut` | `cache::aspectj::cache_aspect_support::tests::test_execute_cache_put` | V2_MIRRORED | ✅ |
| `AbstractCacheAspectTests` | `testCacheEvict` | `cache::aspectj::cache_aspect_support::tests::test_execute_cache_evict` | V2_MIRRORED | ✅ |
| `AbstractAsyncExecutionAspectTests` | `testAsyncMethod` | `scheduling::aspectj::annotation_async_execution_aspect::tests::test_matches_async_marked_method` | V2_MIRRORED | ✅ |
| `AbstractAsyncExecutionAspectTests` | `testExecutorSelection` | `scheduling::aspectj::abstract_async_execution_aspect::tests::test_determine_async_executor` | V2_MIRRORED | ✅ |
| `AnnotationBeanConfigurerAspectTests` | `testPostConstructionInjection` | `beans::factory::aspectj::abstract_dependency_injection_aspect::tests::test_after_construction` | V2_MIRRORED | ✅ |
| `AnnotationBeanConfigurerAspectTests` | `testPreConstructionInjection` | `beans::factory::aspectj::abstract_dependency_injection_aspect::tests::test_before_construction` | V2_MIRRORED | ✅ |

## 统计

| 指标 | 数量 |
|---|---|
| Java 测试方法 | 17 |
| 已映射到 Rust | 17 |
| 映射率 | 100% |
