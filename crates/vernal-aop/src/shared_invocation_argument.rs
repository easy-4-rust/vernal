//! 共享借用 AOP 参数合同。

/// 标记可以被 Send-AOP Future 共享借用的参数目标类型。
///
/// 当被拦截方法接收 `&T` 时，跨 Tokio worker 移动的是共享引用，所以真正需要
/// 满足的是 `T: Sync`，而不是要求引用具有 `'static` 生命周期。宏对顶层共享引用
/// 投影本合同，从而保留调用方生命周期并准确解释线程安全要求。
///
/// 该 trait 由 blanket implementation 提供，业务类型不应手工实现。
pub trait SharedInvocationArgument: Sync {}

impl<T> SharedInvocationArgument for T where T: ?Sized + Sync {}
