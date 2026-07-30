//! BooleanTypedValue（对标 Spring `BooleanTypedValue`）。
//!
//! 对标 Java `org.springframework.expression.spel.support.BooleanTypedValue`。
//! 用于在 SpEL 中表示布尔类型的类型化值。
//!
//! # Spring 行为
//!
//! - `BooleanTypedValue.TRUE` — 表示 true 的类型化值
//! - `BooleanTypedValue.FALSE` — 表示 false 的类型化值
//! - 继承自 `TypedValue`，但固定类型为 `boolean`

use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue};

/// BooleanTypedValue（对标 Spring `BooleanTypedValue`）。
///
/// 用于在 SpEL 中表示布尔类型的类型化值。
/// 提供 `TRUE` 和 `FALSE` 两个常量实例。
pub struct BooleanTypedValue;

impl BooleanTypedValue {
    /// 表示 true 的类型化值。
    pub const TRUE: TypedValue = TypedValue::new(
        ExpressionValue::Boolean(true),
        TypeDescriptor::BOOLEAN,
    );

    /// 表示 false 的类型化值。
    pub const FALSE: TypedValue = TypedValue::new(
        ExpressionValue::Boolean(false),
        TypeDescriptor::BOOLEAN,
    );

    /// 根据布尔值获取对应的类型化值。
    #[must_use]
    pub fn for_value(value: bool) -> TypedValue {
        if value {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }

    /// 从 TypedValue 中提取布尔值。
    #[must_use]
    pub fn typed_value_to_boolean(tv: &TypedValue) -> Option<bool> {
        match tv.value() {
            ExpressionValue::Boolean(b) => Some(*b),
            ExpressionValue::Int(i) => Some(*i != 0),
            ExpressionValue::Long(l) => Some(*l != 0),
            ExpressionValue::Float(f) => Some(*f != 0.0),
            ExpressionValue::Double(d) => Some(*d != 0.0),
            ExpressionValue::String(s) => match s.as_str() {
                "true" | "TRUE" | "1" => Some(true),
                "false" | "FALSE" | "0" => Some(false),
                _ => None,
            },
            ExpressionValue::Null => Some(false),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_value_true() {
        let tv = BooleanTypedValue::for_value(true);
        assert_eq!(*tv.value(), ExpressionValue::Boolean(true));
    }

    #[test]
    fn for_value_false() {
        let tv = BooleanTypedValue::for_value(false);
        assert_eq!(*tv.value(), ExpressionValue::Boolean(false));
    }

    #[test]
    fn typed_value_to_boolean_from_bool() {
        let tv = TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN);
        assert_eq!(BooleanTypedValue::typed_value_to_boolean(&tv), Some(true));
    }

    #[test]
    fn typed_value_to_boolean_from_int() {
        let tv = TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT);
        assert_eq!(BooleanTypedValue::typed_value_to_boolean(&tv), Some(true));
    }

    #[test]
    fn typed_value_to_boolean_from_null() {
        let tv = TypedValue::null();
        assert_eq!(BooleanTypedValue::typed_value_to_boolean(&tv), Some(false));
    }
}
