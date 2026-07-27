//! 错误码 trait。
//!
//! 对标 `tx_di` 的 `CodeMsg` 模式：每个错误枚举变体携带域、码、消息三元组，
//! 支持程序化匹配和跨 crate 边界的结构化错误传播。
//!
//! 通过 `#[derive(ErrorCode)]` 宏（在 `vernal-macros` 中实现）自动生成此 trait 的实现。

use super::VernalError;

/// 错误码 trait。
///
/// 实现此 trait 的枚举可以与 [`VernalError`] 无缝互转。
/// 每个变体通过 `#[error("DOMAIN")]` 和 `#[error(code, "message")]` 属性标注。
///
/// # 设计来源
///
/// 对标 `tx_di` 的 `CodeMsg` trait，适配 vernal 的类型体系。
///
/// # 实现方式
///
/// 通常通过 `#[derive(ErrorCode)]` 宏自动生成，无需手动实现。
///
/// # 示例
///
/// ```rust
/// use vernal_core::error::ErrorCode;
///
/// // 手动实现示例（实际使用 #[derive(ErrorCode)] 自动生成）
/// enum IoCError {
///     NotFound,
///     Ambiguous,
/// }
///
/// impl ErrorCode for IoCError {
///     fn domain(&self) -> &'static str { "ioc" }
///     fn code(&self) -> i32 {
///         match self {
///             Self::NotFound => -1,
///             Self::Ambiguous => -2,
///         }
///     }
///     fn message(&self) -> &'static str {
///         match self {
///             Self::NotFound => "组件未找到",
///             Self::Ambiguous => "依赖歧义：找到多个候选",
///         }
///     }
/// }
/// ```
pub trait ErrorCode: Send + Sync + 'static {
    /// 错误所属的子系统域。
    ///
    /// 必须是 [`ErrorDomain`] 中定义的常量之一。
    fn domain(&self) -> &'static str;

    /// 子系统内的错误码。
    ///
    /// 负数表示框架错误，正数表示业务错误，零保留。
    fn code(&self) -> i32;

    /// 人类可读的静态错误描述。
    ///
    /// 必须是编译期常量字符串，不包含动态数据。
    fn message(&self) -> &'static str;

    /// 转换为 [`VernalError::Business`] 变体。
    ///
    /// 消费自身，将错误码三元组打包为 `VernalError`。
    fn into_vernal_error(self) -> VernalError
    where
        Self: Sized,
    {
        VernalError::Business {
            domain: self.domain(),
            code: self.code(),
            message: self.message(),
        }
    }

    /// 判断两个错误码是否属于同一类错误（忽略上下文）。
    ///
    /// 通过比较 domain + code 实现，用于重试逻辑和错误聚合。
    fn is_same_kind(&self, other: &dyn ErrorCode) -> bool {
        self.domain() == other.domain() && self.code() == other.code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 手动实现 ErrorCode 的测试枚举
    #[derive(Debug)]
    enum TestError {
        NotFound,
        Ambiguous,
    }

    impl ErrorCode for TestError {
        fn domain(&self) -> &'static str { "test" }
        fn code(&self) -> i32 {
            match self {
                Self::NotFound => -1,
                Self::Ambiguous => -2,
            }
        }
        fn message(&self) -> &'static str {
            match self {
                Self::NotFound => "not found",
                Self::Ambiguous => "ambiguous",
            }
        }
    }

    #[test]
    fn domain_code_message() {
        let err = TestError::NotFound;
        assert_eq!(err.domain(), "test");
        assert_eq!(err.code(), -1);
        assert_eq!(err.message(), "not found");
    }

    #[test]
    fn into_vernal_error() {
        let err = TestError::Ambiguous.into_vernal_error();
        assert_eq!(err.domain(), Some("test"));
        assert_eq!(err.code(), Some(-2));
    }

    #[test]
    fn is_same_kind_true() {
        let a = TestError::NotFound;
        let b = TestError::NotFound;
        assert!(a.is_same_kind(&b));
    }

    #[test]
    fn is_same_kind_false_different_code() {
        let a = TestError::NotFound;
        let b = TestError::Ambiguous;
        assert!(!a.is_same_kind(&b));
    }
}
