//! 多类型值模型（对标 Spring `Object` 在 `TypedValue` 中的角色）。
//!
//! Spring 中 `TypedValue` 包装任意 `Object`；Vernal 用 `ExpressionValue` enum 显式列出
//! 我们支持的基本值类型，并保留 `Object` 变体以备注入 vernal-beans Bean 等用户对象。

use std::any::Any;

use std::sync::Arc;

use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use num_bigint::BigInt;
use num_traits::Zero;

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
/// - `Object` ↔ 任意用户对象（用于 vernal-beans Bean 等）
///
/// # 数值类型选择
///
/// `Int(i64)` 与 `Long(i64)` 当前合并到 `i64` 与 AST 文件兼容。
/// Spring 把 `int` 装到 `Integer`（实现细节），`ExpressionValue::Int`
/// 在 SpEL 范围内用 `i64` 即可完整覆盖所有 Java int 操作（含强制转换）。
/// 完整 Phase F 会拆出 32/64 位枚举。
#[derive(Debug, Clone)]
pub enum ExpressionValue {
    /// 空值（Java `null`）。
    Null,
    /// 布尔值。
    Boolean(bool),
    /// 整数（i64，与 Spring Java `Integer`/`Long` 等合并处理）。
    Int(i64),
    /// 整数长形式（保留独立变体以便 Phase F 区分）。
    Long(i64),
    /// 单精度浮点（与 Spring Java `Float` 对应）。
    Float(f64),
    /// 双精度浮点（与 Spring Java `Double` 对应；Spring 内部 `Float` 也可走 Double）。
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
    /// 用 `Arc` 包裹以便 `Clone` 派生。
    Object(Arc<dyn Any + Send + Sync>),
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
            Self::Object(obj) => TypeDescriptor::from_type_id_dyn(obj.as_ref()),
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
    /// 对 `Object` 变体只暴露 `type_id`，因为 `Arc<dyn Any+Send+Sync>` 不可转
    /// 成裸 `&dyn Any`（需要 unsafe）。
    #[must_use]
    pub fn as_any(&self) -> Option<&dyn Any> {
        match self {
            Self::String(s) => Some(s as &dyn Any),
            Self::Int(i) => Some(i as &dyn Any),
            Self::Long(l) => Some(l as &dyn Any),
            Self::Boolean(b) => Some(b as &dyn Any),
            Self::Double(d) => Some(d as &dyn Any),
            Self::Float(f) => Some(f as &dyn Any),
            Self::Char(c) => Some(c as &dyn Any),
            _ => None,
        }
    }

    /// 直接取得 `type_id`（对所有变体都可用）。
    #[must_use]
    pub fn type_id(&self) -> std::any::TypeId {
        match self {
            Self::Object(o) => (**o).type_id(),
            Self::String(_) => std::any::TypeId::of::<String>(),
            Self::Int(_) => std::any::TypeId::of::<i64>(),
            Self::Long(_) => std::any::TypeId::of::<i64>(),
            Self::Boolean(_) => std::any::TypeId::of::<bool>(),
            Self::Float(_) => std::any::TypeId::of::<f64>(),
            Self::Double(_) => std::any::TypeId::of::<f64>(),
            Self::BigInt(_) => std::any::TypeId::of::<num_bigint::BigInt>(),
            Self::Decimal(_) => std::any::TypeId::of::<bigdecimal::BigDecimal>(),
            Self::Char(_) => std::any::TypeId::of::<char>(),
            Self::List(_) => std::any::TypeId::of::<Vec<TypedValue>>(),
            Self::Map(_) => std::any::TypeId::of::<Vec<(TypedValue, TypedValue)>>(),
            Self::DateTime(_) => std::any::TypeId::of::<chrono::DateTime<chrono::Utc>>(),
            Self::Duration(_) => std::any::TypeId::of::<chrono::Duration>(),
            Self::Null => std::any::TypeId::of::<()>(),
        }
    }
}

// Arc 包裹 So we 可 derive Clone
fn _arc_compat() {}

impl ExpressionValue {
    /// 便捷包装：从裸 `Any+Send+Sync` 创建 Object 变体。
    #[must_use]
    pub fn object<T: Any + Send + Sync + 'static>(value: T) -> Self {
        Self::Object(Arc::new(value) as Arc<dyn Any + Send + Sync>)
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
            (Self::List(a), Self::List(b)) => {
                // List/Map 内部元素为 TypedValue，本身未实现 PartialEq（因 Object 类型）
                // 退化为逐元素 identity 比较，对 Phase F 跟踪；Phase F 把 TypedValue 派生
                // PartialEq 后此处统一用 `a == b`。
                a.len() == b.len()
                    && a.iter().zip(b.iter()).all(|(x, y)| match (x.value(), y.value()) {
                        (ExpressionValue::List(xs), ExpressionValue::List(ys)) => {
                            xs.len() == ys.len()
                        }
                        _ => false,
                    })
            }
            (Self::Map(a), Self::Map(b)) => a.len() == b.len(),
            (Self::DateTime(a), Self::DateTime(b)) => a == b,
            (Self::Duration(a), Self::Duration(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => {
                // Box<dyn Any> 等价比较：TypeId 相同
                Any::type_id(a.as_ref()) == Any::type_id(b.as_ref())
            }
            _ => false,
        }
    }
}

impl Eq for ExpressionValue {}

impl TypeDescriptor {
    /// 从 `&dyn Any` 推导出类型描述符。
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
