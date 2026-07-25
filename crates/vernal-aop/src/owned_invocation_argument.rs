//! Owned AOP 调用参数合同。

/// 标记可以进入 `'static` Send-AOP 最终目标的 owned 参数。
///
/// `self: Arc<Self>` 方法会把参数移动到可跨 Tokio worker 调度的最终目标中，因此
/// 参数必须满足 `Send + 'static`。该 trait 由 Vernal 为所有满足条件的类型自动
/// 实现；业务代码不应手工实现它，而应修正参数所有权或补充真实泛型边界。
///
/// `#[vernal_macros::intercept]` 会把这个可命名合同写入方法的 `where` 子句，使
/// 不兼容类型在方法声明或调用处直接指出 owned 参数边界，而不是泄漏 Box Future
/// 内部类型错误。
pub trait OwnedInvocationArgument: Send + 'static {}

impl<T> OwnedInvocationArgument for T where T: Send + 'static {}
