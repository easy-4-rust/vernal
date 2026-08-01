<!-- migration-doc: authority=support canonical=../迁移验收规范.md -->

> 迁移文档治理：本文级别为 **support**。正文中的历史统计或完成标记不得单独作为验收结论；以 [../迁移验收规范.md](../迁移验收规范.md) 和自动审计报告为准。

# vernal-aspects RUST_OBLIGATION 测试台账

> 本文档记录 Rust 实现引入的特有测试义务。
> 这些测试在 Java 中不存在，但在 Rust 中必须验证。

## 测试矩阵

| Rust 机制 | 测试场景 | 测试位置 | 状态 |
|---|---|---|---|
| **Ownership / Drop** | 事务资源在 panic 时正确释放 | `transaction::aspectj::transaction_aspect_support::tests` | ✅ |
| **Ownership / Drop** | 缓存锁在异常路径正确释放 | `cache::aspectj::cache_aspect_support::tests` | ✅ |
| **Send + Sync** | 所有核心 trait 满足 Send + Sync 约束 | 各模块 `tests::test_*_is_send_sync` | ✅ |
| **Send + Sync** | 切面可在多线程运行时使用 | `scheduling::aspectj::abstract_async_execution_aspect::tests` | ✅ |
| **Typed Errors** | TransactionError 包含完整错误信息 | `transaction::aspectj::transaction_aspect_support::tests::test_transaction_error_new` | ✅ |
| **Typed Errors** | CacheResult 错误变体正确传播 | `cache::aspectj::cache_aspect_support::tests::test_cache_result_error` | ✅ |
| **Arc / RwLock** | 事务管理器缓存并发安全 | `transaction::aspectj::transaction_aspect_support::tests::test_clear_transaction_manager_cache` | ✅ |
| **Arc / RwLock** | 缓存管理器并发访问安全 | `cache::aspectj::cache_aspect_support::tests::test_cache_aspect_support_set_cache_manager` | ✅ |
| **Trait Bounds** | TransactionAttributeSource 泛型约束正确 | `transaction::aspectj::transaction_attribute_source::tests` | ✅ |
| **Trait Bounds** | CacheOperationSource 泛型约束正确 | `cache::aspectj::cache_operation_source::tests` | ✅ |
| **Enum Safety** | Propagation 枚举所有变体可匹配 | `transaction::aspectj::propagation::tests::test_propagation_variants` | ✅ |
| **Enum Safety** | Isolation 枚举所有变体可匹配 | `transaction::aspectj::isolation::tests::test_isolation_variants` | ✅ |
| **panic::catch_unwind** | AnyThrow 正确捕获 panic | `cache::aspectj::any_throw::tests::test_any_throw_try_execute_panic` | ✅ |
| **panic::catch_unwind** | Rethrower 正确捕获 panic | `transaction::aspectj::rethrower::tests::test_rethrower_try_execute_panic` | ✅ |
| **Cow<'static, str>** | 事务属性中字符串字段零拷贝 | `transaction::aspectj::transaction_attribute::tests` | ✅ |
| **Default trait** | 所有配置结构体有合理默认值 | 各模块 `tests::test_*_default` | ✅ |
| **Debug trait** | 所有公开类型可调试输出 | 各模块 `tests::test_*_debug` | ✅ |
| **Clone trait** | 需要复制的类型正确实现 Clone | 各模块 `tests::test_*_clone` | ✅ |

## 统计

| 指标 | 数量 |
|---|---|
| Rust 特有测试场景 | 18 |
| 已实现 | 18 |
| 覆盖率 | 100% |
