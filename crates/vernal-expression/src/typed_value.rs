//! 类型化值封装。
//!
//! 对标 Spring 的 `TypedValue`：封装一个值及其类型描述符。

use std::fmt;

/// 类型化值。
///
/// 封装一个值及其 [`TypeDescriptor`]，用于在表达式求值过程中
/// 传递类型安全的值。
///
/// 对标 Spring 的 `org.springframework.expression.TypedValue`。
#[derive(Debug, Clone, PartialEq)]
pub struct TypedValue {
    /// 值（类型擦除）
    value: ExpressionValue,
    /// 类型描述符
    type_descriptor: TypeDescriptor,
}

/// 表达式值枚举。
#[derive(Debug, Clone)]
pub enum ExpressionValue {
    /// 空值
    Null,
    /// 布尔值
    Boolean(bool),
    /// 整数值
    Int(i64),
    /// 浮点值
    Float(f64),
    /// 字符串值
    String(String),
    /// 列表值
    List(Vec<TypedValue>),
    /// 映射值
    Map(Vec<(TypedValue, TypedValue)>),
}

impl PartialEq for ExpressionValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ExpressionValue {}

impl TypedValue {
    /// 空值单例。
    pub const NULL: TypedValue = TypedValue {
        value: ExpressionValue::Null,
        type_descriptor: TypeDescriptor::OBJECT,
    };

    /// 创建类型化值。
    #[must_use]
    pub fn new(value: ExpressionValue, type_descriptor: TypeDescriptor) -> Self {
        Self {
            value,
            type_descriptor,
        }
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

    /// 获取类型描述符。
    #[must_use]
    pub fn type_descriptor(&self) -> &TypeDescriptor {
        &self.type_descriptor
    }

    /// 是否为空值。
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self.value, ExpressionValue::Null)
    }
}

/// 类型描述符。
///
/// 对标 Spring 的 `TypeDescriptor`。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeDescriptor {
    /// 类型名称
    name: &'static str,
}

impl TypeDescriptor {
    /// Object 类型
    pub const OBJECT: TypeDescriptor = TypeDescriptor { name: "object" };
    /// Boolean 类型
    pub const BOOLEAN: TypeDescriptor = TypeDescriptor { name: "boolean" };
    /// Int 类型
    pub const INT: TypeDescriptor = TypeDescriptor { name: "int" };
    /// Float 类型
    pub const FLOAT: TypeDescriptor = TypeDescriptor { name: "float" };
    /// String 类型
    pub const STRING: TypeDescriptor = TypeDescriptor { name: "string" };

    /// 创建类型描述符。
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    /// 获取类型名称。
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl fmt::Display for TypeDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
