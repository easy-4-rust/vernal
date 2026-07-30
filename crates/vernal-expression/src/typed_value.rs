//! 类型化值（对标 Spring `TypedValue`）。
//!
//! 此文件只承载 `TypedValue` 结构体；`ExpressionValue` 与 `TypeDescriptor`
//! 在 `crate::expression_value` 与 `crate::type_descriptor` 里定义，并通过 `pub use` 重导出
//! 以保持现有 AST 文件的 `use crate::typed_value::{ExpressionValue, TypeDescriptor, TypedValue}`
//! 路径可用。
//!
//! 对应 Java 类：`org.springframework.expression.TypedValue`。

use std::fmt;

// `ExpressionValue` 与 `TypeDescriptor` 真正的定义在 expression_value.rs / type_descriptor.rs
// （13 变体 + 富描述符，Phase A 引入）。这里重导出以兼容。
pub use crate::expression_value::ExpressionValue;
pub use crate::type_descriptor::{PrimitiveKind, TypeDescriptor};

/// 类型化值（对标 Spring `TypedValue`）。
///
/// 封装一个值及其 [`TypeDescriptor`]，用于在表达式求值过程中
/// 传递类型安全的值。
///
/// # 与 Spring 的关系
///
/// Spring `TypedValue` 包装任意 `Object` 和 `TypeDescriptor`。
/// Rust 中 `ExpressionValue` 枚举显式列出支持的值类型（13 变体），
/// `TypeDescriptor` 描述值的类型信息（包括泛型参数、注解等）。
///
/// Spring `TypeDescriptor` 的"延迟初始化"语义（`initialized when/if requested`）
/// 在 Rust 中不适用，因为类型描述符在构造时即确定。
#[derive(Debug, Clone)]
pub struct TypedValue {
    /// 当前值。
    value: ExpressionValue,
    /// 类型描述符。
    type_descriptor: TypeDescriptor,
}

impl TypedValue {
    /// 空值常量。
    ///
    /// 对标 Java `TypedValue.NULL`。
    /// 用于表示无值状态（如未解析的根对象）。
    #[allow(dead_code)]
    pub const NULL: Self = Self {
        value: ExpressionValue::Null,
        type_descriptor: TypeDescriptor::NULL,
    };

    /// 创建类型化值。
    ///
    /// 对标 Java `TypedValue(Object value)` 构造器。
    /// 类型描述符通过 `TypeDescriptor::forObject(value)` 自动推导。
    ///
    /// # 参数
    ///
    /// - `value` — 要封装的值
    /// - `type_descriptor` — 值的类型描述符
    #[must_use]
    pub const fn new(value: ExpressionValue, type_descriptor: TypeDescriptor) -> Self {
        Self {
            value,
            type_descriptor,
        }
    }

    /// 创建空值（推荐工厂，避免 const 兼容问题）。
    ///
    /// 对标 Java `new TypedValue(null)` 或 `TypedValue.NULL`。
    #[must_use]
    pub fn null() -> Self {
        Self::new(ExpressionValue::Null, TypeDescriptor::NULL)
    }

    /// 获取值引用。
    ///
    /// 对标 Java `TypedValue.getValue()`。
    #[must_use]
    pub fn value(&self) -> &ExpressionValue {
        &self.value
    }

    /// 获取类型描述符。
    ///
    /// 对标 Java `TypedValue.getTypeDescriptor()`。
    #[must_use]
    pub fn type_descriptor(&self) -> &TypeDescriptor {
        &self.type_descriptor
    }

    /// 是否为空值。
    ///
    /// 对标 Java 中 `value == null` 的判断。
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
