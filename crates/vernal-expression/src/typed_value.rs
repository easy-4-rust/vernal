//! 类型化值（对标 Spring `TypedValue`）。
//!
//! 此文件只承载 `TypedValue` 结构体；`ExpressionValue` 与 `TypeDescriptor`
//! 在 `crate::expression_value` 与 `crate::type_descriptor` 里定义，并通过 `pub use` 重导出
//! 以保持现有 AST 文件的 `use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue}`
//! 路径可用。

use std::fmt;

// `ExpressionValue` 与 `TypeDescriptor` 真正的定义在 expression_value.rs / type_descriptor.rs
// （13 变体 + 富描述符，Phase A 引入）。这里重导出以兼容。
pub use crate::expression_value::ExpressionValue;
pub use crate::type_descriptor::{PrimitiveKind, TypeDescriptor};

/// 类型化值（对标 Spring `org.springframework.expression.TypedValue`）。
#[derive(Debug, Clone)]
pub struct TypedValue {
    value: ExpressionValue,
    type_descriptor: TypeDescriptor,
}

impl TypedValue {
    /// 空值常量。
    #[allow(dead_code)]
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

    /// 创建空值（推荐工厂，避免 const 兼容问题）。
    #[must_use]
    pub fn null() -> Self {
        Self::new(ExpressionValue::Null, TypeDescriptor::NULL)
    }

    /// 获取值引用。
    #[must_use]
    pub fn value(&self) -> &ExpressionValue {
        &self.value
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
}

impl Default for TypedValue {
    fn default() -> Self {
        Self::null()
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
