//! 类型化值封装（对标 Spring `TypedValue`）。

use std::fmt;

use crate::expression_value::ExpressionValue;
use crate::type_descriptor::{PrimitiveKind, TypeDescriptor};

/// 类型化值（值 + 类型描述符）。
///
/// 对标 Spring `org.springframework.expression.TypedValue`。
#[derive(Debug, Clone)]
pub struct TypedValue {
    /// 当前值。
    value: ExpressionValue,
    /// 类型描述符。
    type_descriptor: TypeDescriptor,
}

impl TypedValue {
    /// 空值单例。
    pub const NULL: Self = Self {
        value: ExpressionValue::Null,
        type_descriptor: TypeDescriptor::NULL,
    };

    /// 创建类型化值。
    #[must_use]
    pub fn new(value: ExpressionValue, type_descriptor: TypeDescriptor) -> Self {
        Self {
            value,
            type_descriptor,
        }
    }

    /// 通过值构造，类型描述符自动从值推导。
    #[must_use]
    pub fn of(value: ExpressionValue) -> Self {
        let td = value.type_descriptor();
        Self::new(value, td)
    }

    /// 创建空值。
    #[must_use]
    pub fn null() -> Self {
        Self::NULL
    }

    /// 获取值引用。
    #[must_use]
    pub fn value(&self) -> &ExpressionValue {
        &self.value
    }

    /// 获取值所有权。
    #[must_use]
    pub fn into_value(self) -> ExpressionValue {
        self.value
    }

    /// 获取类型描述符。
    #[must_use]
    pub fn type_descriptor(&self) -> &TypeDescriptor {
        &self.type_descriptor
    }

    /// 是否为空值。
    #[must_use]
    pub fn is_null(&self) -> bool {
        self.value.is_null()
    }

    /// 创建 boolean `true`。
    #[must_use]
    pub fn bool_true() -> Self {
        Self::new(
            ExpressionValue::Boolean(true),
            TypeDescriptor::Primitive(PrimitiveKind::Boolean),
        )
    }

    /// 创建 boolean `false`。
    #[must_use]
    pub fn bool_false() -> Self {
        Self::new(
            ExpressionValue::Boolean(false),
            TypeDescriptor::Primitive(PrimitiveKind::Boolean),
        )
    }

    /// 转换为布尔值（Spring `ExpressionUtils.toBoolean`）。
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match &self.value {
            ExpressionValue::Boolean(b) => Some(*b),
            ExpressionValue::Int(i) => Some(*i != 0),
            ExpressionValue::Long(l) => Some(*l != 0),
            ExpressionValue::Double(d) => Some(*d != 0.0),
            ExpressionValue::Float(f) => Some(*f != 0.0),
            _ => None,
        }
    }
}

impl Default for TypedValue {
    fn default() -> Self {
        Self::NULL
    }
}

impl PartialEq for TypedValue {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl fmt::Display for TypedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            ExpressionValue::Null => f.write_str("null"),
            ExpressionValue::String(s) => write!(f, "{s}"),
            ExpressionValue::Int(i) => write!(f, "{i}"),
            ExpressionValue::Long(l) => write!(f, "{l}"),
            ExpressionValue::Float(x) => write!(f, "{x}"),
            ExpressionValue::Double(x) => write!(f, "{x}"),
            ExpressionValue::Boolean(b) => write!(f, "{b}"),
            ExpressionValue::Char(c) => write!(f, "'{c}'"),
            ExpressionValue::BigInt(b) => write!(f, "{b}"),
            ExpressionValue::Decimal(d) => write!(f, "{d}"),
            ExpressionValue::List(l) => {
                f.write_str("[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{v}")?;
                }
                f.write_str("]")
            }
            ExpressionValue::Map(m) => {
                f.write_str("{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                f.write_str("}")
            }
            ExpressionValue::DateTime(dt) => write!(f, "{dt}"),
            ExpressionValue::Duration(d) => write!(f, "{d}"),
            ExpressionValue::Object(_) => f.write_str("<object>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_singleton() {
        assert_eq!(TypedValue::NULL, TypedValue::null());
        assert!(TypedValue::NULL.is_null());
    }

    #[test]
    fn auto_descriptor() {
        let v = TypedValue::of(ExpressionValue::Int(42));
        assert_eq!(v.type_descriptor().primitive_kind(), Some(PrimitiveKind::Int));
    }
}
