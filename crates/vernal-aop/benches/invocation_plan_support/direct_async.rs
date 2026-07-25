//! 直接异步调用基线。

/// 返回与 AOP 目标相同的固定业务值。
///
/// 该基线仍通过 Criterion 的 Tokio executor 执行，因此不会把 Runtime 驱动方式
/// 的差异误计为 Vernal AOP 开销。
pub async fn direct_async() -> u64 {
    42
}
