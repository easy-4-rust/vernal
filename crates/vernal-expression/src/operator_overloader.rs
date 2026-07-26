//! 运算符重载器 trait。
//!
//! 对标 Spring 的 `OperatorOverloader`。

use super::operation::Operation;
use super::typed_value::TypedValue;

/// 运算符重载器 trait。
///
/// 支持非标准类型的自定义数学运算。
/// 对标 Spring 的 `org.springframework.expression.OperatorOverloader`。
pub trait OperatorOverloader: Send + Sync {
    /// 是否重载了指定操作。
    fn overrides_operation(
        &self,
        operation: Operation,
        left: &TypedValue,
        right: &TypedValue,
    ) -> bool;

    /// 执行重载的操作。
    fn operate(
        &self,
        operation: Operation,
        left: &TypedValue,
        right: &TypedValue,
    ) -> Result<TypedValue, String>;
}
