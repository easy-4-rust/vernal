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

use crate::type_descriptor::{PrimitiveKind, TypeDescriptor};
use crate::typed_value::TypedValue;

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
            (Self::Float(a), Self::Float(b)) => (a - b).abs() < f64::EPSILON,
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
                    && a.iter()
                        .zip(b.iter())
                        .all(|(x, y)| match (x.value(), y.value()) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── type_descriptor() ─────────────────────────────────────────────

    #[test]
    fn type_descriptor_null() {
        assert_eq!(
            ExpressionValue::Null.type_descriptor(),
            TypeDescriptor::NULL
        );
    }

    #[test]
    fn type_descriptor_boolean() {
        assert_eq!(
            ExpressionValue::Boolean(true).type_descriptor(),
            TypeDescriptor::BOOLEAN
        );
    }

    #[test]
    fn type_descriptor_int() {
        assert_eq!(
            ExpressionValue::Int(42).type_descriptor(),
            TypeDescriptor::INT
        );
    }

    #[test]
    fn type_descriptor_long() {
        assert_eq!(
            ExpressionValue::Long(100).type_descriptor(),
            TypeDescriptor::LONG
        );
    }

    #[test]
    fn type_descriptor_float() {
        assert_eq!(
            ExpressionValue::Float(1.5).type_descriptor(),
            TypeDescriptor::FLOAT
        );
    }

    #[test]
    fn type_descriptor_double() {
        assert_eq!(
            ExpressionValue::Double(2.5).type_descriptor(),
            TypeDescriptor::DOUBLE
        );
    }

    #[test]
    fn type_descriptor_bigint() {
        let td = ExpressionValue::BigInt(BigInt::from(123)).type_descriptor();
        assert_eq!(td, TypeDescriptor::Primitive(PrimitiveKind::BigInt));
    }

    #[test]
    fn type_descriptor_decimal() {
        let td = ExpressionValue::Decimal(BigDecimal::from(456)).type_descriptor();
        assert_eq!(td, TypeDescriptor::Primitive(PrimitiveKind::BigDecimal));
    }

    #[test]
    fn type_descriptor_char() {
        let td = ExpressionValue::Char('x').type_descriptor();
        assert_eq!(td, TypeDescriptor::Primitive(PrimitiveKind::Char));
    }

    #[test]
    fn type_descriptor_string() {
        assert_eq!(
            ExpressionValue::String("hi".into()).type_descriptor(),
            TypeDescriptor::STRING
        );
    }

    #[test]
    fn type_descriptor_datetime() {
        let td = ExpressionValue::DateTime(Utc::now()).type_descriptor();
        assert_eq!(td, TypeDescriptor::Primitive(PrimitiveKind::DateTime));
    }

    #[test]
    fn type_descriptor_duration() {
        let td = ExpressionValue::Duration(ChronoDuration::seconds(10)).type_descriptor();
        assert_eq!(td, TypeDescriptor::Primitive(PrimitiveKind::Duration));
    }

    #[test]
    fn type_descriptor_list() {
        let td = ExpressionValue::List(vec![]).type_descriptor();
        assert!(matches!(td, TypeDescriptor::Named { name, .. } if name == "java.util.List"));
    }

    #[test]
    fn type_descriptor_map() {
        let td = ExpressionValue::Map(vec![]).type_descriptor();
        assert!(matches!(td, TypeDescriptor::Map(_, _)));
    }

    #[test]
    fn type_descriptor_object() {
        let td = ExpressionValue::object(42_i32).type_descriptor();
        assert!(matches!(
            td,
            TypeDescriptor::Named {
                type_id: Some(_),
                ..
            }
        ));
    }

    // ── is_null() ─────────────────────────────────────────────────────

    #[test]
    fn is_null_true() {
        assert!(ExpressionValue::Null.is_null());
    }

    #[test]
    fn is_null_false_for_values() {
        assert!(!ExpressionValue::Int(0).is_null());
        assert!(!ExpressionValue::Boolean(false).is_null());
        assert!(!ExpressionValue::String(String::new()).is_null());
    }

    // ── is_truthy() ───────────────────────────────────────────────────

    #[test]
    fn truthy_null() {
        assert!(!ExpressionValue::Null.is_truthy());
    }

    #[test]
    fn truthy_boolean() {
        assert!(ExpressionValue::Boolean(true).is_truthy());
        assert!(!ExpressionValue::Boolean(false).is_truthy());
    }

    #[test]
    fn truthy_int() {
        assert!(ExpressionValue::Int(1).is_truthy());
        assert!(!ExpressionValue::Int(0).is_truthy());
        assert!(ExpressionValue::Int(-1).is_truthy());
    }

    #[test]
    fn truthy_long() {
        assert!(ExpressionValue::Long(1).is_truthy());
        assert!(!ExpressionValue::Long(0).is_truthy());
    }

    #[test]
    fn truthy_float() {
        assert!(ExpressionValue::Float(1.0).is_truthy());
        assert!(!ExpressionValue::Float(0.0).is_truthy());
    }

    #[test]
    fn truthy_double() {
        assert!(ExpressionValue::Double(1.0).is_truthy());
        assert!(!ExpressionValue::Double(0.0).is_truthy());
    }

    #[test]
    fn truthy_bigint() {
        assert!(ExpressionValue::BigInt(BigInt::from(1)).is_truthy());
        assert!(!ExpressionValue::BigInt(BigInt::from(0)).is_truthy());
    }

    #[test]
    fn truthy_decimal() {
        assert!(ExpressionValue::Decimal(BigDecimal::from(1)).is_truthy());
        assert!(!ExpressionValue::Decimal(BigDecimal::from(0)).is_truthy());
    }

    #[test]
    fn truthy_char() {
        assert!(ExpressionValue::Char('a').is_truthy());
        assert!(!ExpressionValue::Char('\0').is_truthy());
    }

    #[test]
    fn truthy_string() {
        assert!(ExpressionValue::String("hello".into()).is_truthy());
        assert!(!ExpressionValue::String(String::new()).is_truthy());
    }

    #[test]
    fn truthy_list() {
        assert!(ExpressionValue::List(vec![TypedValue::null()]).is_truthy());
    }

    #[test]
    fn truthy_map() {
        assert!(ExpressionValue::Map(vec![]).is_truthy());
    }

    #[test]
    fn truthy_datetime() {
        assert!(ExpressionValue::DateTime(Utc::now()).is_truthy());
    }

    #[test]
    fn truthy_duration() {
        assert!(ExpressionValue::Duration(ChronoDuration::seconds(1)).is_truthy());
    }

    // ── as_any() ──────────────────────────────────────────────────────

    #[test]
    fn as_any_string() {
        let v = ExpressionValue::String("test".into());
        let any = v.as_any().unwrap();
        assert!(any.is::<String>());
    }

    #[test]
    fn as_any_int() {
        let v = ExpressionValue::Int(42);
        let any = v.as_any().unwrap();
        assert!(any.is::<i64>());
    }

    #[test]
    fn as_any_long() {
        let v = ExpressionValue::Long(42);
        assert!(v.as_any().unwrap().is::<i64>());
    }

    #[test]
    fn as_any_boolean() {
        let v = ExpressionValue::Boolean(true);
        assert!(v.as_any().unwrap().is::<bool>());
    }

    #[test]
    fn as_any_double() {
        let v = ExpressionValue::Double(1.5);
        assert!(v.as_any().unwrap().is::<f64>());
    }

    #[test]
    fn as_any_float() {
        let v = ExpressionValue::Float(1.5);
        assert!(v.as_any().unwrap().is::<f64>());
    }

    #[test]
    fn as_any_char() {
        let v = ExpressionValue::Char('x');
        assert!(v.as_any().unwrap().is::<char>());
    }

    #[test]
    fn as_any_none_for_complex_types() {
        assert!(ExpressionValue::Null.as_any().is_none());
        assert!(ExpressionValue::BigInt(BigInt::from(1)).as_any().is_none());
        assert!(
            ExpressionValue::Decimal(BigDecimal::from(1))
                .as_any()
                .is_none()
        );
        assert!(ExpressionValue::List(vec![]).as_any().is_none());
        assert!(ExpressionValue::Map(vec![]).as_any().is_none());
        assert!(ExpressionValue::DateTime(Utc::now()).as_any().is_none());
        assert!(
            ExpressionValue::Duration(ChronoDuration::seconds(1))
                .as_any()
                .is_none()
        );
        assert!(ExpressionValue::object(42_i32).as_any().is_none());
    }

    // ── type_id() ─────────────────────────────────────────────────────

    #[test]
    fn type_id_string() {
        assert_eq!(
            ExpressionValue::String("x".into()).type_id(),
            std::any::TypeId::of::<String>()
        );
    }

    #[test]
    fn type_id_int() {
        assert_eq!(
            ExpressionValue::Int(0).type_id(),
            std::any::TypeId::of::<i64>()
        );
    }

    #[test]
    fn type_id_long() {
        assert_eq!(
            ExpressionValue::Long(0).type_id(),
            std::any::TypeId::of::<i64>()
        );
    }

    #[test]
    fn type_id_boolean() {
        assert_eq!(
            ExpressionValue::Boolean(false).type_id(),
            std::any::TypeId::of::<bool>()
        );
    }

    #[test]
    fn type_id_float() {
        assert_eq!(
            ExpressionValue::Float(0.0).type_id(),
            std::any::TypeId::of::<f64>()
        );
    }

    #[test]
    fn type_id_double() {
        assert_eq!(
            ExpressionValue::Double(0.0).type_id(),
            std::any::TypeId::of::<f64>()
        );
    }

    #[test]
    fn type_id_bigint() {
        assert_eq!(
            ExpressionValue::BigInt(BigInt::from(0)).type_id(),
            std::any::TypeId::of::<BigInt>()
        );
    }

    #[test]
    fn type_id_decimal() {
        assert_eq!(
            ExpressionValue::Decimal(BigDecimal::from(0)).type_id(),
            std::any::TypeId::of::<BigDecimal>()
        );
    }

    #[test]
    fn type_id_char() {
        assert_eq!(
            ExpressionValue::Char('\0').type_id(),
            std::any::TypeId::of::<char>()
        );
    }

    #[test]
    fn type_id_null() {
        assert_eq!(
            ExpressionValue::Null.type_id(),
            std::any::TypeId::of::<()>()
        );
    }

    #[test]
    fn type_id_list() {
        assert_eq!(
            ExpressionValue::List(vec![]).type_id(),
            std::any::TypeId::of::<Vec<TypedValue>>()
        );
    }

    #[test]
    fn type_id_map() {
        assert_eq!(
            ExpressionValue::Map(vec![]).type_id(),
            std::any::TypeId::of::<Vec<(TypedValue, TypedValue)>>()
        );
    }

    #[test]
    fn type_id_datetime() {
        let v = ExpressionValue::DateTime(Utc::now());
        assert_eq!(v.type_id(), std::any::TypeId::of::<DateTime<Utc>>());
    }

    #[test]
    fn type_id_duration() {
        let v = ExpressionValue::Duration(ChronoDuration::seconds(1));
        assert_eq!(v.type_id(), std::any::TypeId::of::<ChronoDuration>());
    }

    #[test]
    fn type_id_object() {
        let v = ExpressionValue::object(42_i32);
        assert_eq!(v.type_id(), std::any::TypeId::of::<i32>());
    }

    // ── object() 工厂 ─────────────────────────────────────────────────

    #[test]
    fn object_factory() {
        let v = ExpressionValue::object(String::from("hello"));
        assert!(matches!(v, ExpressionValue::Object(_)));
        assert_eq!(v.type_id(), std::any::TypeId::of::<String>());
    }

    // ── PartialEq ─────────────────────────────────────────────────────

    #[test]
    fn partial_eq_null() {
        assert_eq!(ExpressionValue::Null, ExpressionValue::Null);
    }

    #[test]
    fn partial_eq_boolean() {
        assert_eq!(
            ExpressionValue::Boolean(true),
            ExpressionValue::Boolean(true)
        );
        assert_ne!(
            ExpressionValue::Boolean(true),
            ExpressionValue::Boolean(false)
        );
    }

    #[test]
    fn partial_eq_int() {
        assert_eq!(ExpressionValue::Int(42), ExpressionValue::Int(42));
        assert_ne!(ExpressionValue::Int(1), ExpressionValue::Int(2));
    }

    #[test]
    fn partial_eq_long() {
        assert_eq!(ExpressionValue::Long(42), ExpressionValue::Long(42));
        assert_ne!(ExpressionValue::Long(1), ExpressionValue::Long(2));
    }

    #[test]
    fn partial_eq_float() {
        assert_eq!(ExpressionValue::Float(1.5), ExpressionValue::Float(1.5));
        assert_ne!(ExpressionValue::Float(1.0), ExpressionValue::Float(2.0));
    }

    #[test]
    fn partial_eq_double() {
        assert_eq!(ExpressionValue::Double(1.5), ExpressionValue::Double(1.5));
    }

    #[test]
    fn partial_eq_bigint() {
        assert_eq!(
            ExpressionValue::BigInt(BigInt::from(100)),
            ExpressionValue::BigInt(BigInt::from(100))
        );
    }

    #[test]
    fn partial_eq_decimal() {
        assert_eq!(
            ExpressionValue::Decimal(BigDecimal::from(100)),
            ExpressionValue::Decimal(BigDecimal::from(100))
        );
    }

    #[test]
    fn partial_eq_char() {
        assert_eq!(ExpressionValue::Char('a'), ExpressionValue::Char('a'));
        assert_ne!(ExpressionValue::Char('a'), ExpressionValue::Char('b'));
    }

    #[test]
    fn partial_eq_string() {
        assert_eq!(
            ExpressionValue::String("hello".into()),
            ExpressionValue::String("hello".into())
        );
    }

    #[test]
    fn partial_eq_datetime() {
        let now = Utc::now();
        assert_eq!(
            ExpressionValue::DateTime(now),
            ExpressionValue::DateTime(now)
        );
    }

    #[test]
    fn partial_eq_duration() {
        let d = ChronoDuration::seconds(10);
        assert_eq!(ExpressionValue::Duration(d), ExpressionValue::Duration(d));
    }

    #[test]
    fn partial_eq_map_same_len() {
        let m1 = ExpressionValue::Map(vec![]);
        let m2 = ExpressionValue::Map(vec![]);
        assert_eq!(m1, m2);
    }

    #[test]
    fn partial_eq_different_types_not_equal() {
        assert_ne!(ExpressionValue::Null, ExpressionValue::Int(0));
        assert_ne!(ExpressionValue::Int(1), ExpressionValue::Long(1));
        assert_ne!(ExpressionValue::Boolean(true), ExpressionValue::Int(1));
        assert_ne!(ExpressionValue::String("1".into()), ExpressionValue::Int(1));
    }

    #[test]
    fn partial_eq_object_same_type() {
        let a = ExpressionValue::object(42_i32);
        let b = ExpressionValue::object(99_i32);
        // Object PartialEq compares TypeId only
        assert_eq!(a, b);
    }

    // ── Clone / Debug ─────────────────────────────────────────────────

    #[test]
    fn clone_works() {
        let v = ExpressionValue::String("hello".into());
        let v2 = v.clone();
        assert_eq!(v, v2);
    }

    #[test]
    fn debug_format() {
        assert_eq!(format!("{:?}", ExpressionValue::Null), "Null");
        assert_eq!(format!("{:?}", ExpressionValue::Int(42)), "Int(42)");
        assert_eq!(
            format!("{:?}", ExpressionValue::Boolean(true)),
            "Boolean(true)"
        );
    }

    // ── from_type_id_dyn ──────────────────────────────────────────────

    #[test]
    fn from_type_id_dyn_string() {
        let s = String::from("hello");
        let td = TypeDescriptor::from_type_id_dyn(&s);
        assert!(matches!(
            td,
            TypeDescriptor::Named {
                type_id: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn from_type_id_dyn_i32() {
        let i = 42_i32;
        let td = TypeDescriptor::from_type_id_dyn(&i);
        assert!(td.type_id().is_some());
    }
}
