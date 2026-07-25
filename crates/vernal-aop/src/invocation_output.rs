//! AOP 调用成功返回值合同。

use std::any::Any;

/// 标记可以穿过动态拦截器链并恢复静态类型的成功返回值。
///
/// Vernal 把成功值暂时擦除为 [`crate::InvocationValue`]，随后在宏生成的方法边界
/// 恢复原始类型。因此返回值必须满足 `Any + Send + Sync + 'static`，既能跨 Tokio
/// task 安全传递，也不会让借用逃出一次调用。
///
/// `#[vernal_macros::intercept]` 将本合同显式投影到方法签名。关联类型、嵌套泛型
/// 或具体非线程安全类型不满足要求时，编译器会直接报告 `InvocationOutput` 边界。
pub trait InvocationOutput: Any + Send + Sync + 'static {}

impl<T> InvocationOutput for T where T: Any + Send + Sync + 'static {}
