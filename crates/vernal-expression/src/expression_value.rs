//! 多类型值模型（对标 Spring `Object` 在 `TypedValue` 中的角色）。
//!
//! Spring 中 `TypedValue` 包装任意 `Object`；Vernal 用 `ExpressionValue` enum 显式列出
//! 我们支持的基本值类型，并保留 `Object` 变体以备注入 vernal-beans Bean 等用户对象。

use std::any::Any;

use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use num_bigint::BigInt;

use crate::typed_value::TypedValue;
use crate::type_descriptor::{PrimitiveKind, TypeDescriptor};

/// 表达式值枚举（13 变体）。
///
/// 与 Spring `TypedValue` 1:1 对齐语义：
/// - `Null` ↔ Java `null`
/// - `Boolean/Int/Long/Float` ↔ Java primitive + boxed
/// - `BigInt` ↔ `java.math.BigInteger`
/// - `Decimal` ↔ `java.math.BigDecimal`
/// - `Char/String` ↔ `char/String`
/// - `DateTime/Duration` ↔ `java.util.Date` / `java.time.Duration`
/// - `List/Map` ↔ Java `List/Map`
/// - `Object` ↔ 任意用户对象（用于 vernal-beans 等 IoC 集成）
#[derive(Debug, Clone)]
pub enum ExpressionValue {
    /// 空值（Java `null`）。
    Null,
    /// 布尔值。
    Boolean(bool),
    /// 32 位整数（Spring 默认 int）。
    Int(i32),
    /// 64 位长整数。
    Long(i64),
    /// 单精度浮点。
    Float(f32),
    /// 双精度浮点。
    Double(f64),
    /// 任意精度整数（对标 `java.math.BigInteger`）。
    BigInt(BigInt),
    /// 任意精度小数（对标 `java.math.BigDecimal`）。
    Decimal(BigDecimal),
    /// 单个字符。
    Char(char),
    /// 字符串。
    String(String),
    /// UTC 日期时间（对标 `java.util.Date`）。
    DateTime(DateTime<Utc>),
    /// 时间间隔（对标 `java.time.Duration`）。
    Duration(ChronoDuration),
    /// 列表（对应任意索引集合）。
    List(Vec<TypedValue>),
    /// 映射（`Vec<(K, V)>` 形式以保留顺序并允许任意类型键）。
    Map(Vec<(TypedValue, TypedValue)>),
    /// 任意用户对象（用于 vernal-beans Bean 等）。
    Object(Box<dyn Any + Send + Sync>),
}

impl ExpressionValue {
    /// 获取该值的运行时类型描述符（对标 Spring `TypeDescriptor.forObject`）。
    #[must_use]
    pub fn type_descriptor(&self) -> TypeDescriptor {
        match self {
            Self::Null => TypeDescriptor::Primitive(PrimitiveKind::Null),
            Self::Boolean(_) => TypeDescriptor::Primitive(PrimitiveKind::Boolean),
            Self::Int(_) => TypeDescriptor::Primitive(PrimitiveKind::Int),
            Self::Long(_) => TypeDescriptor::Primitive(PrimitiveKind::Long),
            Self::Float(_) => TypeDescriptor::Primitive(PrimitiveKind::Float),
            Self::Double(_) => TypeDescriptor::Primitive(PrimitiveKind::Double),
            Self::BigInt(_) => TypeDescriptor::Primitive(PrimitiveKind::BigInt),
            Self::Decimal(_) => TypeDescriptor::Primitive(PrimitiveKind::BigDecimal),
            Self::Char(_) => TypeDescriptor::Primitive(PrimitiveKind::Char),
            Self::String(_) => TypeDescriptor::Primitive(PrimitiveKind::String),
            Self::DateTime(_) => TypeDescriptor::Primitive(PrimitiveKind::DateTime),
            Self::Duration(_) => TypeDescriptor::Primitive(PrimitiveKind::Duration),
            Self::List(_) => TypeDescriptor::from_type_name("java.util.List"),
            Self::Map(_) => TypeDescriptor::Map(
                Box::new(TypeDescriptor::OBJECT),
                Box::new(TypeDescriptor::OBJECT),
            ),
            Self::Object(obj) => TypeDescriptor::from_type_id_dyn(obj.as_any()),
        }
    }

    /// 是否为空值。
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Spring 风格的"真值"判定（null → false；Boolean → 原值；其他 → true）。
    /// 用于条件表达式（如 `condition ? a : b`、`matches`/`between`）。
    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Boolean(b) => *b,
            // 数字零值视为假（非零为真）— 与 Java `Boolean.valueOf` 行为一致
            Self::Int(i) => *i != 0,
            Self::Long(l) => *l != 0,
            Self::Float(f) => *f != 0.0,
            Self::Double(d) => *d != 0.0,
            Self::BigInt(b) => !b.is_zero(),
            Self::Decimal(d) => !d.is_zero(),
            Self::Char(c) => *c != '\0',
            Self::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    /// 将值降级为 `Any`（用于反射调用）。
    #[must_use]
    pub fn as_any(&self) -> &dyn Any {
        match self {
            Self::Object(o) => o.as_any(),
            Self::String(s) => s as &dyn Any,
            Self::Int(i) => i as &dyn Any,
            Self::Long(l) => l as &dyn Any,
            Self::Boolean(b) => b as &dyn Any,
            Self::Double(d) => d as &dyn Any,
            Self::Float(f) => f as &dyn Any,
            Self::Char(c) => c as &dyn Any,
            _ => self as &dyn Any,
        }
    }
}

impl PartialEq for ExpressionValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Long(a), Self::Long(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => (a - b).abs() < f32::EPSILON,
            (Self::Double(a), Self::Double(b)) => (a - b).abs() < f64::EPSILON,
            (Self::BigInt(a), Self::BigInt(b)) => a == b,
            (Self::Decimal(a), Self::Decimal(b)) => a == b,
            (Self::Char(a), Self::Char(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::List(a), Self::List(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,
            // DateTime/Duration/Object 使用 Eq 比较（DateTime 实现 PartialEq）
            (Self::DateTime(a), Self::DateTime(b)) => a == b,
            (Self::Duration(a), Self::Duration(b)) => a == b,
            // Object 比较：比较 TypeId（实际对象本身不可比较）
            (Self::Object(a), Self::Object(b)) => {
                a.as_any().type_id() == b.as_any().type_id()
            }
            _ => false,
        }
    }
}

impl TypeDescriptor {
    /// 从 `dyn Any` 推导出类型描述符。
    #[must_use]
    pub fn from_type_id_dyn(any: &dyn Any) -> Self {
        Self::Named {
            type_id: Some(any.type_id()),
            name: std::any::type_name_of_val(any).to_string(),
            generics: Vec::new(),
            annotations: Vec::new(),
        }
    }
}
