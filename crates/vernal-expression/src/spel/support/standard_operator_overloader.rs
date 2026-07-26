//! 标准运算符重载器。
//!
//! 对标 Spring 的 `StandardOperatorOverloader`：默认不重载任何运算符。

use crate::operation::Operation;
use crate::operator_overloader::OperatorOverloader;
use crate::typed_value::TypedValue;

/// 标准运算符重载器。
///
/// 默认实现不重载任何运算符。
/// 对标 Spring 的 `org.springframework.expression.spel.support.StandardOperatorOverloader`。
pub struct StandardOperatorOverloader;

impl StandardOperatorOverloader {
    /// 单例实例。
    pub const INSTANCE: Self = Self;
}

impl OperatorOverloader for StandardOperatorOverloader {
    fn overrides_operation(
        &self,
        _operation: Operation,
        _left_operand: &TypedValue,
        _right_operand: &TypedValue,
    ) -> bool {
        false
    }

    fn operate(
        &self,
        _operation: Operation,
        _left_operand: &TypedValue,
        _right_operand: &TypedValue,
    ) -> Result<TypedValue, String> {
        Err("标准运算符重载器不支持自定义运算".to_string())
    }
}
