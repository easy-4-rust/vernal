//! 类型化值测试（`TypedValue` / `ExpressionValue` / `TypeDescriptor`）。

use bigdecimal::BigDecimal;
use chrono::{Duration as ChronoDuration, TimeZone, Utc};
use num_bigint::BigInt;
use vernal_expression::{ExpressionValue, PrimitiveKind, TypeDescriptor, TypedValue};

// ═══════════════════════════════════════════════════════════════════
//  TypedValue::new() with all ExpressionValue variants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn typed_value_new_with_null() {
    let tv = TypedValue::new(ExpressionValue::Null, TypeDescriptor::NULL);
    assert!(tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Null));
}

#[test]
fn typed_value_new_with_boolean() {
    let tv = TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN);
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Boolean(true)));
}

#[test]
fn typed_value_new_with_int() {
    let tv = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Int(42)));
}

#[test]
fn typed_value_new_with_long() {
    let tv = TypedValue::new(ExpressionValue::Long(100), TypeDescriptor::LONG);
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Long(100)));
}

#[test]
fn typed_value_new_with_float() {
    let tv = TypedValue::new(ExpressionValue::Float(3.14), TypeDescriptor::FLOAT);
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Float(_)));
}

#[test]
fn typed_value_new_with_double() {
    let tv = TypedValue::new(ExpressionValue::Double(2.718), TypeDescriptor::DOUBLE);
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Double(_)));
}

#[test]
fn typed_value_new_with_bigint() {
    let tv = TypedValue::new(
        ExpressionValue::BigInt(BigInt::from(42)),
        TypeDescriptor::Primitive(PrimitiveKind::BigInt),
    );
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::BigInt(_)));
}

#[test]
fn typed_value_new_with_decimal() {
    let tv = TypedValue::new(
        ExpressionValue::Decimal(BigDecimal::from(42)),
        TypeDescriptor::Primitive(PrimitiveKind::BigDecimal),
    );
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Decimal(_)));
}

#[test]
fn typed_value_new_with_char() {
    let tv = TypedValue::new(
        ExpressionValue::Char('a'),
        TypeDescriptor::Primitive(PrimitiveKind::Char),
    );
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Char('a')));
}

#[test]
fn typed_value_new_with_string() {
    let tv = TypedValue::new(
        ExpressionValue::String("hello".to_string()),
        TypeDescriptor::STRING,
    );
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::String(s) if s == "hello"));
}

#[test]
fn typed_value_new_with_datetime() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    let tv = TypedValue::new(
        ExpressionValue::DateTime(dt),
        TypeDescriptor::Primitive(PrimitiveKind::DateTime),
    );
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::DateTime(_)));
}

#[test]
fn typed_value_new_with_duration() {
    let dur = ChronoDuration::seconds(42);
    let tv = TypedValue::new(
        ExpressionValue::Duration(dur),
        TypeDescriptor::Primitive(PrimitiveKind::Duration),
    );
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Duration(_)));
}

#[test]
fn typed_value_new_with_list() {
    let list = ExpressionValue::List(vec![
        TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
        TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT),
    ]);
    let tv = TypedValue::new(list, TypeDescriptor::from_type_name("java.util.List"));
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::List(l) if l.len() == 2));
}

#[test]
fn typed_value_new_with_map() {
    let map = ExpressionValue::Map(vec![(
        TypedValue::new(
            ExpressionValue::String("k".to_string()),
            TypeDescriptor::STRING,
        ),
        TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
    )]);
    let td = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let tv = TypedValue::new(map, td);
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Map(m) if m.len() == 1));
}

#[test]
fn typed_value_new_with_object() {
    let obj = ExpressionValue::object(42i32);
    let td = obj.type_descriptor();
    let tv = TypedValue::new(obj, td);
    assert!(!tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Object(_)));
}

// ═══════════════════════════════════════════════════════════════════
//  TypedValue factory / accessor tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn typed_value_null_factory() {
    let tv = TypedValue::null();
    assert!(tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Null));
}

#[test]
fn typed_value_null_constant() {
    let tv = TypedValue::NULL;
    assert!(tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Null));
    assert!(matches!(
        tv.type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Null)
    ));
}

#[test]
fn typed_value_default_is_null() {
    let tv = TypedValue::default();
    assert!(tv.is_null());
    assert!(matches!(tv.value(), ExpressionValue::Null));
}

#[test]
fn typed_value_value_accessor() {
    let tv = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    assert!(matches!(tv.value(), ExpressionValue::Int(42)));
}

#[test]
fn typed_value_type_descriptor_accessor() {
    let tv = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    assert!(tv.type_descriptor().is_primitive());
    assert!(matches!(
        tv.type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Int)
    ));
}

#[test]
fn typed_value_is_null_for_null_value() {
    let tv = TypedValue::null();
    assert!(tv.is_null());
}

#[test]
fn typed_value_is_null_for_non_null_value() {
    let tv = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    assert!(!tv.is_null());
}

// ═══════════════════════════════════════════════════════════════════
//  TypedValue Display for all ExpressionValue variants
// ═══════════════════════════════════════════════════════════════════

#[test]
fn typed_value_display_null() {
    let tv = TypedValue::null();
    assert_eq!(tv.to_string(), "null");
}

#[test]
fn typed_value_display_boolean_true() {
    let tv = TypedValue::new(ExpressionValue::Boolean(true), TypeDescriptor::BOOLEAN);
    assert_eq!(tv.to_string(), "true");
}

#[test]
fn typed_value_display_boolean_false() {
    let tv = TypedValue::new(ExpressionValue::Boolean(false), TypeDescriptor::BOOLEAN);
    assert_eq!(tv.to_string(), "false");
}

#[test]
fn typed_value_display_int() {
    let tv = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
    assert_eq!(tv.to_string(), "42");

    let tv_neg = TypedValue::new(ExpressionValue::Int(-7), TypeDescriptor::INT);
    assert_eq!(tv_neg.to_string(), "-7");
}

#[test]
fn typed_value_display_long() {
    let tv = TypedValue::new(ExpressionValue::Long(999), TypeDescriptor::LONG);
    assert_eq!(tv.to_string(), "999");
}

#[test]
fn typed_value_display_float() {
    let tv = TypedValue::new(ExpressionValue::Float(1.0), TypeDescriptor::FLOAT);
    assert_eq!(tv.to_string(), "1");
}

#[test]
fn typed_value_display_double() {
    let tv = TypedValue::new(ExpressionValue::Double(1.0), TypeDescriptor::DOUBLE);
    assert_eq!(tv.to_string(), "1");
}

#[test]
fn typed_value_display_bigint() {
    let tv = TypedValue::new(
        ExpressionValue::BigInt(BigInt::from(42)),
        TypeDescriptor::Primitive(PrimitiveKind::BigInt),
    );
    assert_eq!(tv.to_string(), "42");
}

#[test]
fn typed_value_display_decimal() {
    let tv = TypedValue::new(
        ExpressionValue::Decimal(BigDecimal::from(42)),
        TypeDescriptor::Primitive(PrimitiveKind::BigDecimal),
    );
    assert_eq!(tv.to_string(), "42");
}

#[test]
fn typed_value_display_char() {
    let tv = TypedValue::new(
        ExpressionValue::Char('a'),
        TypeDescriptor::Primitive(PrimitiveKind::Char),
    );
    assert_eq!(tv.to_string(), "'a'");
}

#[test]
fn typed_value_display_string() {
    let tv = TypedValue::new(
        ExpressionValue::String("hello".to_string()),
        TypeDescriptor::STRING,
    );
    assert_eq!(tv.to_string(), "hello");
}

#[test]
fn typed_value_display_datetime() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
    let tv = TypedValue::new(
        ExpressionValue::DateTime(dt),
        TypeDescriptor::Primitive(PrimitiveKind::DateTime),
    );
    let display = tv.to_string();
    assert!(
        display.contains("2024"),
        "DateTime display should contain year, got: {display}"
    );
}

#[test]
fn typed_value_display_duration() {
    let dur = ChronoDuration::seconds(3600);
    let tv = TypedValue::new(
        ExpressionValue::Duration(dur),
        TypeDescriptor::Primitive(PrimitiveKind::Duration),
    );
    let display = tv.to_string();
    assert!(!display.is_empty(), "Duration display should not be empty");
}

#[test]
fn typed_value_display_list() {
    let list = ExpressionValue::List(vec![
        TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
        TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT),
    ]);
    let tv = TypedValue::new(list, TypeDescriptor::from_type_name("java.util.List"));
    assert_eq!(tv.to_string(), "[1, 2]");
}

#[test]
fn typed_value_display_empty_list() {
    let list = ExpressionValue::List(vec![]);
    let tv = TypedValue::new(list, TypeDescriptor::from_type_name("java.util.List"));
    assert_eq!(tv.to_string(), "[]");
}

#[test]
fn typed_value_display_map() {
    let map = ExpressionValue::Map(vec![(
        TypedValue::new(
            ExpressionValue::String("k".to_string()),
            TypeDescriptor::STRING,
        ),
        TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
    )]);
    let td = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let tv = TypedValue::new(map, td);
    assert_eq!(tv.to_string(), "{k: 1}");
}

#[test]
fn typed_value_display_empty_map() {
    let map = ExpressionValue::Map(vec![]);
    let td = TypeDescriptor::Map(
        Box::new(TypeDescriptor::OBJECT),
        Box::new(TypeDescriptor::OBJECT),
    );
    let tv = TypedValue::new(map, td);
    assert_eq!(tv.to_string(), "{}");
}

#[test]
fn typed_value_display_object() {
    let obj = ExpressionValue::object(42i32);
    let td = obj.type_descriptor();
    let tv = TypedValue::new(obj, td);
    assert_eq!(tv.to_string(), "<object>");
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue::is_null()
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_is_null_for_null_variant() {
    assert!(ExpressionValue::Null.is_null());
}

#[test]
fn expression_value_is_null_for_non_null_variants() {
    assert!(!ExpressionValue::Boolean(true).is_null());
    assert!(!ExpressionValue::Int(0).is_null());
    assert!(!ExpressionValue::String("".to_string()).is_null());
    assert!(!ExpressionValue::List(vec![]).is_null());
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue::is_truthy()
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_is_truthy_null_is_false() {
    assert!(!ExpressionValue::Null.is_truthy());
}

#[test]
fn expression_value_is_truthy_boolean_variants() {
    assert!(ExpressionValue::Boolean(true).is_truthy());
    assert!(!ExpressionValue::Boolean(false).is_truthy());
}

#[test]
fn expression_value_is_truthy_numeric_zero_is_false() {
    assert!(!ExpressionValue::Int(0).is_truthy());
    assert!(!ExpressionValue::Long(0).is_truthy());
    assert!(!ExpressionValue::Float(0.0).is_truthy());
    assert!(!ExpressionValue::Double(0.0).is_truthy());
    assert!(!ExpressionValue::BigInt(BigInt::from(0)).is_truthy());
    assert!(!ExpressionValue::Decimal(BigDecimal::from(0)).is_truthy());
}

#[test]
fn expression_value_is_truthy_numeric_nonzero_is_true() {
    assert!(ExpressionValue::Int(1).is_truthy());
    assert!(ExpressionValue::Long(-1).is_truthy());
    assert!(ExpressionValue::Float(1.0).is_truthy());
    assert!(ExpressionValue::Double(-1.0).is_truthy());
    assert!(ExpressionValue::BigInt(BigInt::from(1)).is_truthy());
    assert!(ExpressionValue::Decimal(BigDecimal::from(1)).is_truthy());
}

#[test]
fn expression_value_is_truthy_string_empty_vs_nonempty() {
    assert!(!ExpressionValue::String("".to_string()).is_truthy());
    assert!(ExpressionValue::String("hello".to_string()).is_truthy());
}

#[test]
fn expression_value_is_truthy_char_null_char() {
    assert!(!ExpressionValue::Char('\0').is_truthy());
    assert!(ExpressionValue::Char('a').is_truthy());
    assert!(ExpressionValue::Char('0').is_truthy());
}

#[test]
fn expression_value_is_truthy_list_and_map_are_true() {
    assert!(ExpressionValue::List(vec![]).is_truthy());
    assert!(ExpressionValue::Map(vec![]).is_truthy());
    let dt = Utc::now();
    assert!(ExpressionValue::DateTime(dt).is_truthy());
    assert!(ExpressionValue::Duration(ChronoDuration::zero()).is_truthy());
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue::type_descriptor()
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_type_descriptor_null() {
    assert!(matches!(
        ExpressionValue::Null.type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Null)
    ));
}

#[test]
fn expression_value_type_descriptor_boolean() {
    assert!(matches!(
        ExpressionValue::Boolean(true).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Boolean)
    ));
}

#[test]
fn expression_value_type_descriptor_int() {
    assert!(matches!(
        ExpressionValue::Int(42).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Int)
    ));
}

#[test]
fn expression_value_type_descriptor_long() {
    assert!(matches!(
        ExpressionValue::Long(42).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Long)
    ));
}

#[test]
fn expression_value_type_descriptor_float_double() {
    assert!(matches!(
        ExpressionValue::Float(1.0).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Float)
    ));
    assert!(matches!(
        ExpressionValue::Double(1.0).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Double)
    ));
}

#[test]
fn expression_value_type_descriptor_bigint_decimal() {
    assert!(matches!(
        ExpressionValue::BigInt(BigInt::from(42)).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::BigInt)
    ));
    assert!(matches!(
        ExpressionValue::Decimal(BigDecimal::from(42)).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::BigDecimal)
    ));
}

#[test]
fn expression_value_type_descriptor_char_string() {
    assert!(matches!(
        ExpressionValue::Char('a').type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Char)
    ));
    assert!(matches!(
        ExpressionValue::String("hello".to_string()).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::String)
    ));
}

#[test]
fn expression_value_type_descriptor_datetime_duration() {
    assert!(matches!(
        ExpressionValue::DateTime(Utc::now()).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::DateTime)
    ));
    assert!(matches!(
        ExpressionValue::Duration(ChronoDuration::seconds(42)).type_descriptor(),
        TypeDescriptor::Primitive(PrimitiveKind::Duration)
    ));
}

#[test]
fn expression_value_type_descriptor_list_returns_named() {
    let list_td = ExpressionValue::List(vec![]).type_descriptor();
    assert!(matches!(list_td, TypeDescriptor::Named { .. }));
}

#[test]
fn expression_value_type_descriptor_map_returns_map_type() {
    let map_td = ExpressionValue::Map(vec![]).type_descriptor();
    assert!(map_td.is_map());
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue::as_any()
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_as_any_string() {
    let val = ExpressionValue::String("hello".to_string());
    let any_ref = val.as_any().unwrap();
    assert_eq!(any_ref.downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn expression_value_as_any_int() {
    let val = ExpressionValue::Int(42);
    let any_ref = val.as_any().unwrap();
    assert_eq!(*any_ref.downcast_ref::<i64>().unwrap(), 42);
}

#[test]
fn expression_value_as_any_long() {
    let val = ExpressionValue::Long(42);
    let any_ref = val.as_any().unwrap();
    assert_eq!(*any_ref.downcast_ref::<i64>().unwrap(), 42);
}

#[test]
fn expression_value_as_any_boolean() {
    let val = ExpressionValue::Boolean(true);
    let any_ref = val.as_any().unwrap();
    assert!(*any_ref.downcast_ref::<bool>().unwrap());
}

#[test]
fn expression_value_as_any_double() {
    let val = ExpressionValue::Double(3.14);
    let any_ref = val.as_any().unwrap();
    assert!((any_ref.downcast_ref::<f64>().unwrap() - 3.14).abs() < f64::EPSILON);
}

#[test]
fn expression_value_as_any_float() {
    let val = ExpressionValue::Float(2.5);
    let any_ref = val.as_any().unwrap();
    assert!((any_ref.downcast_ref::<f64>().unwrap() - 2.5).abs() < f64::EPSILON);
}

#[test]
fn expression_value_as_any_char() {
    let val = ExpressionValue::Char('x');
    let any_ref = val.as_any().unwrap();
    assert_eq!(*any_ref.downcast_ref::<char>().unwrap(), 'x');
}

#[test]
fn expression_value_as_any_null_returns_none() {
    assert!(ExpressionValue::Null.as_any().is_none());
}

#[test]
fn expression_value_as_any_unsupported_types_return_none() {
    assert!(ExpressionValue::BigInt(BigInt::from(42)).as_any().is_none());
    assert!(
        ExpressionValue::Decimal(BigDecimal::from(42))
            .as_any()
            .is_none()
    );
    assert!(ExpressionValue::List(vec![]).as_any().is_none());
    assert!(ExpressionValue::Map(vec![]).as_any().is_none());
    assert!(ExpressionValue::DateTime(Utc::now()).as_any().is_none());
    assert!(
        ExpressionValue::Duration(ChronoDuration::seconds(42))
            .as_any()
            .is_none()
    );
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue::type_id()
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_type_id_null() {
    assert_eq!(
        ExpressionValue::Null.type_id(),
        std::any::TypeId::of::<()>()
    );
}

#[test]
fn expression_value_type_id_boolean() {
    assert_eq!(
        ExpressionValue::Boolean(true).type_id(),
        std::any::TypeId::of::<bool>()
    );
}

#[test]
fn expression_value_type_id_int_and_long_share_i64() {
    assert_eq!(
        ExpressionValue::Int(0).type_id(),
        std::any::TypeId::of::<i64>()
    );
    assert_eq!(
        ExpressionValue::Long(0).type_id(),
        std::any::TypeId::of::<i64>()
    );
}

#[test]
fn expression_value_type_id_float_and_double_share_f64() {
    assert_eq!(
        ExpressionValue::Float(0.0).type_id(),
        std::any::TypeId::of::<f64>()
    );
    assert_eq!(
        ExpressionValue::Double(0.0).type_id(),
        std::any::TypeId::of::<f64>()
    );
}

#[test]
fn expression_value_type_id_string() {
    assert_eq!(
        ExpressionValue::String("".to_string()).type_id(),
        std::any::TypeId::of::<String>()
    );
}

#[test]
fn expression_value_type_id_char() {
    assert_eq!(
        ExpressionValue::Char('\0').type_id(),
        std::any::TypeId::of::<char>()
    );
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue PartialEq
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_partial_eq_null() {
    assert_eq!(ExpressionValue::Null, ExpressionValue::Null);
}

#[test]
fn expression_value_partial_eq_boolean_equal() {
    assert_eq!(
        ExpressionValue::Boolean(true),
        ExpressionValue::Boolean(true)
    );
    assert_eq!(
        ExpressionValue::Boolean(false),
        ExpressionValue::Boolean(false)
    );
}

#[test]
fn expression_value_partial_eq_boolean_not_equal() {
    assert_ne!(
        ExpressionValue::Boolean(true),
        ExpressionValue::Boolean(false)
    );
}

#[test]
fn expression_value_partial_eq_int_equal() {
    assert_eq!(ExpressionValue::Int(42), ExpressionValue::Int(42));
}

#[test]
fn expression_value_partial_eq_int_not_equal() {
    assert_ne!(ExpressionValue::Int(42), ExpressionValue::Int(43));
}

#[test]
fn expression_value_partial_eq_long() {
    assert_eq!(ExpressionValue::Long(100), ExpressionValue::Long(100));
    assert_ne!(ExpressionValue::Long(100), ExpressionValue::Long(200));
}

#[test]
fn expression_value_partial_eq_float_within_epsilon() {
    assert_eq!(ExpressionValue::Float(1.0), ExpressionValue::Float(1.0));
    // 0.1 + 0.2 is within epsilon of 0.3
    assert_eq!(
        ExpressionValue::Double(0.1 + 0.2),
        ExpressionValue::Double(0.3)
    );
}

#[test]
fn expression_value_partial_eq_float_outside_epsilon() {
    assert_ne!(ExpressionValue::Float(1.0), ExpressionValue::Float(2.0));
    assert_ne!(ExpressionValue::Double(1.0), ExpressionValue::Double(2.0));
}

#[test]
fn expression_value_partial_eq_bigint() {
    assert_eq!(
        ExpressionValue::BigInt(BigInt::from(42)),
        ExpressionValue::BigInt(BigInt::from(42))
    );
    assert_ne!(
        ExpressionValue::BigInt(BigInt::from(42)),
        ExpressionValue::BigInt(BigInt::from(43))
    );
}

#[test]
fn expression_value_partial_eq_decimal() {
    assert_eq!(
        ExpressionValue::Decimal(BigDecimal::from(42)),
        ExpressionValue::Decimal(BigDecimal::from(42))
    );
}

#[test]
fn expression_value_partial_eq_char() {
    assert_eq!(ExpressionValue::Char('a'), ExpressionValue::Char('a'));
    assert_ne!(ExpressionValue::Char('a'), ExpressionValue::Char('b'));
}

#[test]
fn expression_value_partial_eq_string() {
    assert_eq!(
        ExpressionValue::String("hello".to_string()),
        ExpressionValue::String("hello".to_string())
    );
    assert_ne!(
        ExpressionValue::String("hello".to_string()),
        ExpressionValue::String("world".to_string())
    );
}

#[test]
fn expression_value_partial_eq_different_types_not_equal() {
    assert_ne!(ExpressionValue::Int(42), ExpressionValue::Long(42));
    assert_ne!(ExpressionValue::Int(42), ExpressionValue::Float(42.0));
    assert_ne!(ExpressionValue::Null, ExpressionValue::Int(0));
    assert_ne!(ExpressionValue::Null, ExpressionValue::Boolean(false));
    assert_ne!(ExpressionValue::Boolean(true), ExpressionValue::Int(1));
    assert_ne!(
        ExpressionValue::String("42".to_string()),
        ExpressionValue::Int(42)
    );
}

#[test]
fn expression_value_partial_eq_datetime() {
    let dt1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let dt2 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let dt3 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
    assert_eq!(
        ExpressionValue::DateTime(dt1),
        ExpressionValue::DateTime(dt2)
    );
    assert_ne!(
        ExpressionValue::DateTime(dt1),
        ExpressionValue::DateTime(dt3)
    );
}

#[test]
fn expression_value_partial_eq_duration() {
    assert_eq!(
        ExpressionValue::Duration(ChronoDuration::seconds(42)),
        ExpressionValue::Duration(ChronoDuration::seconds(42))
    );
    assert_ne!(
        ExpressionValue::Duration(ChronoDuration::seconds(42)),
        ExpressionValue::Duration(ChronoDuration::seconds(43))
    );
}

#[test]
fn expression_value_partial_eq_list_by_length() {
    // Current impl: List compares lengths and nested list structure only
    let list_a = ExpressionValue::List(vec![TypedValue::new(
        ExpressionValue::Int(1),
        TypeDescriptor::INT,
    )]);
    let list_b = ExpressionValue::List(vec![TypedValue::new(
        ExpressionValue::Int(2),
        TypeDescriptor::INT,
    )]);
    let list_c = ExpressionValue::List(vec![
        TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
        TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT),
    ]);
    // Same length but non-list inner values -> not equal (current impl)
    assert_ne!(list_a, list_b);
    // Different length -> not equal
    assert_ne!(list_a, list_c);
}

#[test]
fn expression_value_partial_eq_map_by_length() {
    let map_a = ExpressionValue::Map(vec![(
        TypedValue::new(
            ExpressionValue::String("a".to_string()),
            TypeDescriptor::STRING,
        ),
        TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT),
    )]);
    let map_b = ExpressionValue::Map(vec![(
        TypedValue::new(
            ExpressionValue::String("b".to_string()),
            TypeDescriptor::STRING,
        ),
        TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT),
    )]);
    let map_c = ExpressionValue::Map(vec![]);
    // Same length -> equal (current impl compares length only)
    assert_eq!(map_a, map_b);
    // Different length -> not equal
    assert_ne!(map_a, map_c);
}

#[test]
fn expression_value_partial_eq_object_same_type_id() {
    let obj_a = ExpressionValue::object(42i32);
    let obj_b = ExpressionValue::object(99i32);
    // Same underlying type -> equal (current impl compares TypeId)
    assert_eq!(obj_a, obj_b);
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue Clone
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_clone_null() {
    let v = ExpressionValue::Null;
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_boolean() {
    let v = ExpressionValue::Boolean(true);
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_int() {
    let v = ExpressionValue::Int(42);
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_long() {
    let v = ExpressionValue::Long(42);
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_float() {
    let v = ExpressionValue::Float(3.14);
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_double() {
    let v = ExpressionValue::Double(3.14);
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_bigint() {
    let v = ExpressionValue::BigInt(BigInt::from(42));
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_decimal() {
    let v = ExpressionValue::Decimal(BigDecimal::from(42));
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_char() {
    let v = ExpressionValue::Char('z');
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_string() {
    let v = ExpressionValue::String("hello".to_string());
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_datetime() {
    let dt = Utc::now();
    let v = ExpressionValue::DateTime(dt);
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_duration() {
    let v = ExpressionValue::Duration(ChronoDuration::seconds(42));
    assert_eq!(v, v.clone());
}

#[test]
fn expression_value_clone_list() {
    let v = ExpressionValue::List(vec![TypedValue::new(
        ExpressionValue::Int(1),
        TypeDescriptor::INT,
    )]);
    let cloned = v.clone();
    assert!(matches!(cloned, ExpressionValue::List(l) if l.len() == 1));
}

#[test]
fn expression_value_clone_map() {
    let v = ExpressionValue::Map(vec![]);
    let cloned = v.clone();
    assert!(matches!(cloned, ExpressionValue::Map(m) if m.is_empty()));
}

#[test]
fn expression_value_clone_object() {
    let v = ExpressionValue::object(42i32);
    let cloned = v.clone();
    assert!(matches!(cloned, ExpressionValue::Object(_)));
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue Debug
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_debug_null() {
    let debug = format!("{:?}", ExpressionValue::Null);
    assert!(debug.contains("Null"));
}

#[test]
fn expression_value_debug_boolean() {
    let debug = format!("{:?}", ExpressionValue::Boolean(true));
    assert!(debug.contains("Boolean"));
    assert!(debug.contains("true"));
}

#[test]
fn expression_value_debug_int() {
    let debug = format!("{:?}", ExpressionValue::Int(42));
    assert!(debug.contains("Int"));
    assert!(debug.contains("42"));
}

#[test]
fn expression_value_debug_long() {
    let debug = format!("{:?}", ExpressionValue::Long(100));
    assert!(debug.contains("Long"));
    assert!(debug.contains("100"));
}

#[test]
fn expression_value_debug_float() {
    let debug = format!("{:?}", ExpressionValue::Float(1.5));
    assert!(debug.contains("Float"));
}

#[test]
fn expression_value_debug_double() {
    let debug = format!("{:?}", ExpressionValue::Double(2.5));
    assert!(debug.contains("Double"));
}

#[test]
fn expression_value_debug_bigint() {
    let debug = format!("{:?}", ExpressionValue::BigInt(BigInt::from(42)));
    assert!(debug.contains("BigInt"));
}

#[test]
fn expression_value_debug_decimal() {
    let debug = format!("{:?}", ExpressionValue::Decimal(BigDecimal::from(42)));
    assert!(debug.contains("Decimal"));
}

#[test]
fn expression_value_debug_char() {
    let debug = format!("{:?}", ExpressionValue::Char('a'));
    assert!(debug.contains("Char"));
    assert!(debug.contains('a'));
}

#[test]
fn expression_value_debug_string() {
    let debug = format!("{:?}", ExpressionValue::String("hello".to_string()));
    assert!(debug.contains("String"));
    assert!(debug.contains("hello"));
}

#[test]
fn expression_value_debug_list() {
    let debug = format!("{:?}", ExpressionValue::List(vec![]));
    assert!(debug.contains("List"));
}

#[test]
fn expression_value_debug_map() {
    let debug = format!("{:?}", ExpressionValue::Map(vec![]));
    assert!(debug.contains("Map"));
}

#[test]
fn expression_value_debug_object() {
    let obj = ExpressionValue::object(42i32);
    let debug = format!("{:?}", obj);
    assert!(debug.contains("Object"));
}

// ═══════════════════════════════════════════════════════════════════
//  ExpressionValue::object() factory
// ═══════════════════════════════════════════════════════════════════

#[test]
fn expression_value_object_factory() {
    let obj = ExpressionValue::object(42i32);
    assert!(matches!(obj, ExpressionValue::Object(_)));
    assert!(!obj.is_null());
    assert!(obj.is_truthy());
    // Object does not support as_any
    assert!(obj.as_any().is_none());
}

#[test]
fn expression_value_object_with_string() {
    let obj = ExpressionValue::object("hello".to_string());
    assert!(matches!(obj, ExpressionValue::Object(_)));
    assert!(!obj.is_null());
}
