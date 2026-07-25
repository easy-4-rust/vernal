//! 切点组合扩展契约。

use crate::{AndPointcut, NotPointcut, OrPointcut, Pointcut};

/// 为全部具体切点提供类型安全的逻辑组合方法。
///
/// 该扩展 trait 不改变 [`Pointcut`] 的对象安全核心合同；只有在建造切点表达式时
/// 要求 `Self: Sized`。因此现有闭包、自定义切点和 Vernal 内建切点都可以使用
/// `.and(...)`、`.or(...)` 与 `.not()`，最终仍由 Advisor 统一类型擦除。
pub trait PointcutExt: Pointcut + Sized {
    /// 创建同时匹配当前切点与 `other` 的逻辑与表达式。
    #[must_use]
    fn and<P>(self, other: P) -> AndPointcut<Self, P>
    where
        P: Pointcut,
    {
        AndPointcut::new(self, other)
    }

    /// 创建匹配当前切点或 `other` 的逻辑或表达式。
    #[must_use]
    fn or<P>(self, other: P) -> OrPointcut<Self, P>
    where
        P: Pointcut,
    {
        OrPointcut::new(self, other)
    }

    /// 创建对当前切点结果取反的逻辑非表达式。
    #[must_use]
    fn not(self) -> NotPointcut<Self> {
        NotPointcut::new(self)
    }
}

impl<P> PointcutExt for P where P: Pointcut {}
