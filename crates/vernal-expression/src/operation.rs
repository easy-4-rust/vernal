//! 操作枚举。
//!
//! 对标 Spring 的 `Operation` 枚举：支持的数学运算操作。

/// 支持的操作枚举。
///
/// 对标 Spring 的 `org.springframework.expression.Operation`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    /// 加法
    Add,
    /// 减法
    Subtract,
    /// 乘法
    Multiply,
    /// 除法
    Divide,
    /// 取模
    Modulus,
    /// 幂运算
    Power,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Subtract => write!(f, "-"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Modulus => write!(f, "%"),
            Self::Power => write!(f, "^"),
        }
    }
}
