//! 借用型 AOP 调用参数合同。

/// 标记可以进入当前调用期 Send-AOP Future 的 borrowed 参数。
///
/// `&self` 与 `&mut self` 方法允许参数借用调用方数据，生命周期不必是
/// `'static`；但完整业务 Future 仍可能在 Tokio worker 间移动，所以按值参数与
/// `&mut T` 的目标类型必须满足 `Send`。顶层共享 `&T` 则由
/// [`crate::SharedInvocationArgument`] 表达 `T: Sync`。
///
/// 该 trait 具有 blanket implementation，业务代码只需声明真实的 `Send`/`Sync`
/// 泛型事实，不应手工实现框架标记。
pub trait BorrowedInvocationArgument: Send {}

impl<T> BorrowedInvocationArgument for T where T: ?Sized + Send {}
