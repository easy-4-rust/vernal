//! 标准运算符重载器（对标 Spring `StandardOperatorOverloader`）。
//!
//! 对标 Java `org.springframework.expression.spel.support.StandardOperatorOverloader`。
//! 默认不重载任何运算符（所有运算使用 SpEL 内置语义）。
//! 用户可通过 `Inventory` 注册自定义运算符覆盖。

use crate::operation::Operation;
use crate::operator_overloader::OperatorOverloader;
use crate::typed_value::TypedValue;

/// 标准运算符重载器（对标 Spring `StandardOperatorOverloader`）。
///
/// 默认实现不重载任何运算符。
/// 用户可通过 `inventory::submit!` 注册自定义运算符覆盖。
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_value::ExpressionValue;
    use crate::type_descriptor::TypeDescriptor;

    #[test]
    fn does_not_override_any_operation() {
        let overloader = StandardOperatorOverloader;
        let left = TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT);
        let right = TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT);
        assert!(!overloader.overrides_operation(Operation::Add, &left, &right));
        assert!(!overloader.overrides_operation(Operation::Subtract, &left, &right));
    }

    #[test]
    fn operate_returns_error() {
        let overloader = StandardOperatorOverloader;
        let left = TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT);
        let right = TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT);
        assert!(overloader.operate(Operation::Add, &left, &right).is_err());
    }
}
