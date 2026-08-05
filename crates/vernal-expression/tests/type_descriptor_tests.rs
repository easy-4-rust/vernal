//! Comprehensive tests for `TypeDescriptor` and `PrimitiveKind`.
//!
//! Covers all 14 PrimitiveKind variants, every public method on both types,
//! PartialEq/Eq/Hash, Display, and `is_assignable_from` with all combinations.

use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use vernal_expression::type_descriptor::{PrimitiveKind, TypeDescriptor};

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn hash_of<T: Hash>(v: &T) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

// ===========================================================================
// 1. PrimitiveKind::name() for all 14 variants
// ===========================================================================

#[test]
fn primitive_kind_name_null() {
    assert_eq!(PrimitiveKind::Null.name(), "null");
}

#[test]
fn primitive_kind_name_boolean() {
    assert_eq!(PrimitiveKind::Boolean.name(), "boolean");
}

#[test]
fn primitive_kind_name_byte() {
    assert_eq!(PrimitiveKind::Byte.name(), "byte");
}

#[test]
fn primitive_kind_name_short() {
    assert_eq!(PrimitiveKind::Short.name(), "short");
}

#[test]
fn primitive_kind_name_int() {
    assert_eq!(PrimitiveKind::Int.name(), "int");
}

#[test]
fn primitive_kind_name_long() {
    assert_eq!(PrimitiveKind::Long.name(), "long");
}

#[test]
fn primitive_kind_name_float() {
    assert_eq!(PrimitiveKind::Float.name(), "float");
}

#[test]
fn primitive_kind_name_double() {
    assert_eq!(PrimitiveKind::Double.name(), "double");
}

#[test]
fn primitive_kind_name_big_int() {
    assert_eq!(PrimitiveKind::BigInt.name(), "java.math.BigInteger");
}

#[test]
fn primitive_kind_name_big_decimal() {
    assert_eq!(PrimitiveKind::BigDecimal.name(), "java.math.BigDecimal");
}

#[test]
fn primitive_kind_name_char() {
    assert_eq!(PrimitiveKind::Char.name(), "char");
}

#[test]
fn primitive_kind_name_string() {
    assert_eq!(PrimitiveKind::String.name(), "java.lang.String");
}

#[test]
fn primitive_kind_name_date_time() {
    assert_eq!(PrimitiveKind::DateTime.name(), "java.util.Date");
}

#[test]
fn primitive_kind_name_duration() {
    assert_eq!(PrimitiveKind::Duration.name(), "java.time.Duration");
}

// ===========================================================================
// 2. PrimitiveKind::numeric_width() for all 14 variants
// ===========================================================================

#[test]
fn numeric_width_byte() {
    assert_eq!(PrimitiveKind::Byte.numeric_width(), 1);
}

#[test]
fn numeric_width_short() {
    assert_eq!(PrimitiveKind::Short.numeric_width(), 2);
}

#[test]
fn numeric_width_int() {
    assert_eq!(PrimitiveKind::Int.numeric_width(), 3);
}

#[test]
fn numeric_width_long() {
    assert_eq!(PrimitiveKind::Long.numeric_width(), 4);
}

#[test]
fn numeric_width_big_int() {
    assert_eq!(PrimitiveKind::BigInt.numeric_width(), 5);
}

#[test]
fn numeric_width_float() {
    assert_eq!(PrimitiveKind::Float.numeric_width(), 6);
}

#[test]
fn numeric_width_double() {
    assert_eq!(PrimitiveKind::Double.numeric_width(), 7);
}

#[test]
fn numeric_width_big_decimal() {
    assert_eq!(PrimitiveKind::BigDecimal.numeric_width(), 8);
}

#[test]
fn numeric_width_non_numeric_types_are_zero() {
    assert_eq!(PrimitiveKind::Null.numeric_width(), 0);
    assert_eq!(PrimitiveKind::Boolean.numeric_width(), 0);
    assert_eq!(PrimitiveKind::Char.numeric_width(), 0);
    assert_eq!(PrimitiveKind::String.numeric_width(), 0);
    assert_eq!(PrimitiveKind::DateTime.numeric_width(), 0);
    assert_eq!(PrimitiveKind::Duration.numeric_width(), 0);
}

// ===========================================================================
// 3. PrimitiveKind::is_integer()
// ===========================================================================

#[test]
fn is_integer_true_for_integer_types() {
    assert!(PrimitiveKind::Byte.is_integer());
    assert!(PrimitiveKind::Short.is_integer());
    assert!(PrimitiveKind::Int.is_integer());
    assert!(PrimitiveKind::Long.is_integer());
    assert!(PrimitiveKind::BigInt.is_integer());
}

#[test]
fn is_integer_false_for_non_integer_types() {
    assert!(!PrimitiveKind::Null.is_integer());
    assert!(!PrimitiveKind::Boolean.is_integer());
    assert!(!PrimitiveKind::Float.is_integer());
    assert!(!PrimitiveKind::Double.is_integer());
    assert!(!PrimitiveKind::BigDecimal.is_integer());
    assert!(!PrimitiveKind::Char.is_integer());
    assert!(!PrimitiveKind::String.is_integer());
    assert!(!PrimitiveKind::DateTime.is_integer());
    assert!(!PrimitiveKind::Duration.is_integer());
}

// ===========================================================================
// 4. PrimitiveKind::is_floating()
// ===========================================================================

#[test]
fn is_floating_true_for_floating_types() {
    assert!(PrimitiveKind::Float.is_floating());
    assert!(PrimitiveKind::Double.is_floating());
    assert!(PrimitiveKind::BigDecimal.is_floating());
}

#[test]
fn is_floating_false_for_non_floating_types() {
    assert!(!PrimitiveKind::Null.is_floating());
    assert!(!PrimitiveKind::Boolean.is_floating());
    assert!(!PrimitiveKind::Byte.is_floating());
    assert!(!PrimitiveKind::Short.is_floating());
    assert!(!PrimitiveKind::Int.is_floating());
    assert!(!PrimitiveKind::Long.is_floating());
    assert!(!PrimitiveKind::BigInt.is_floating());
    assert!(!PrimitiveKind::Char.is_floating());
    assert!(!PrimitiveKind::String.is_floating());
    assert!(!PrimitiveKind::DateTime.is_floating());
    assert!(!PrimitiveKind::Duration.is_floating());
}

// ===========================================================================
// 5. PrimitiveKind::is_big()
// ===========================================================================

#[test]
fn is_big_true_for_big_types() {
    assert!(PrimitiveKind::BigInt.is_big());
    assert!(PrimitiveKind::BigDecimal.is_big());
}

#[test]
fn is_big_false_for_non_big_types() {
    assert!(!PrimitiveKind::Null.is_big());
    assert!(!PrimitiveKind::Boolean.is_big());
    assert!(!PrimitiveKind::Byte.is_big());
    assert!(!PrimitiveKind::Short.is_big());
    assert!(!PrimitiveKind::Int.is_big());
    assert!(!PrimitiveKind::Long.is_big());
    assert!(!PrimitiveKind::Float.is_big());
    assert!(!PrimitiveKind::Double.is_big());
    assert!(!PrimitiveKind::Char.is_big());
    assert!(!PrimitiveKind::String.is_big());
    assert!(!PrimitiveKind::DateTime.is_big());
    assert!(!PrimitiveKind::Duration.is_big());
}

// ===========================================================================
// 6. PrimitiveKind::widen()
// ===========================================================================

#[test]
fn widen_same_type_returns_self() {
    assert_eq!(
        PrimitiveKind::Int.widen(PrimitiveKind::Int),
        PrimitiveKind::Int
    );
    assert_eq!(
        PrimitiveKind::Long.widen(PrimitiveKind::Long),
        PrimitiveKind::Long
    );
    assert_eq!(
        PrimitiveKind::BigDecimal.widen(PrimitiveKind::BigDecimal),
        PrimitiveKind::BigDecimal
    );
}

#[test]
fn widen_narrow_to_wide() {
    assert_eq!(
        PrimitiveKind::Byte.widen(PrimitiveKind::Short),
        PrimitiveKind::Short
    );
    assert_eq!(
        PrimitiveKind::Byte.widen(PrimitiveKind::Int),
        PrimitiveKind::Int
    );
    assert_eq!(
        PrimitiveKind::Byte.widen(PrimitiveKind::Long),
        PrimitiveKind::Long
    );
    assert_eq!(
        PrimitiveKind::Byte.widen(PrimitiveKind::BigInt),
        PrimitiveKind::BigInt
    );
    assert_eq!(
        PrimitiveKind::Byte.widen(PrimitiveKind::Float),
        PrimitiveKind::Float
    );
    assert_eq!(
        PrimitiveKind::Byte.widen(PrimitiveKind::Double),
        PrimitiveKind::Double
    );
    assert_eq!(
        PrimitiveKind::Byte.widen(PrimitiveKind::BigDecimal),
        PrimitiveKind::BigDecimal
    );
}

#[test]
fn widen_wide_to_narrow_returns_wide() {
    assert_eq!(
        PrimitiveKind::Double.widen(PrimitiveKind::Int),
        PrimitiveKind::Double
    );
    assert_eq!(
        PrimitiveKind::BigDecimal.widen(PrimitiveKind::Float),
        PrimitiveKind::BigDecimal
    );
    assert_eq!(
        PrimitiveKind::Long.widen(PrimitiveKind::Byte),
        PrimitiveKind::Long
    );
}

#[test]
fn widen_non_numeric_types_returns_self() {
    // Null has width 0, so self (also 0) >= other (0) => self
    assert_eq!(
        PrimitiveKind::Null.widen(PrimitiveKind::Null),
        PrimitiveKind::Null
    );
    assert_eq!(
        PrimitiveKind::Boolean.widen(PrimitiveKind::Null),
        PrimitiveKind::Boolean
    );
    assert_eq!(
        PrimitiveKind::String.widen(PrimitiveKind::Char),
        PrimitiveKind::String
    );
    assert_eq!(
        PrimitiveKind::DateTime.widen(PrimitiveKind::Duration),
        PrimitiveKind::DateTime
    );
}

#[test]
fn widen_int_long() {
    assert_eq!(
        PrimitiveKind::Int.widen(PrimitiveKind::Long),
        PrimitiveKind::Long
    );
    assert_eq!(
        PrimitiveKind::Long.widen(PrimitiveKind::Int),
        PrimitiveKind::Long
    );
}

#[test]
fn widen_float_double() {
    assert_eq!(
        PrimitiveKind::Float.widen(PrimitiveKind::Double),
        PrimitiveKind::Double
    );
    assert_eq!(
        PrimitiveKind::Double.widen(PrimitiveKind::Float),
        PrimitiveKind::Double
    );
}

#[test]
fn widen_int_float() {
    assert_eq!(
        PrimitiveKind::Int.widen(PrimitiveKind::Float),
        PrimitiveKind::Float
    );
    assert_eq!(
        PrimitiveKind::Float.widen(PrimitiveKind::Int),
        PrimitiveKind::Float
    );
}

#[test]
fn widen_full_chain() {
    let types = [
        PrimitiveKind::Byte,
        PrimitiveKind::Short,
        PrimitiveKind::Int,
        PrimitiveKind::Long,
        PrimitiveKind::BigInt,
        PrimitiveKind::Float,
        PrimitiveKind::Double,
        PrimitiveKind::BigDecimal,
    ];
    for i in 0..types.len() {
        for j in 0..types.len() {
            let result = types[i].widen(types[j]);
            if types[i].numeric_width() >= types[j].numeric_width() {
                assert_eq!(result, types[i], "widen({:?}, {:?})", types[i], types[j]);
            } else {
                assert_eq!(result, types[j], "widen({:?}, {:?})", types[i], types[j]);
            }
        }
    }
}

// ===========================================================================
// 7. PrimitiveKind::Display
// ===========================================================================

#[test]
fn primitive_kind_display_matches_name() {
    let all = [
        PrimitiveKind::Null,
        PrimitiveKind::Boolean,
        PrimitiveKind::Byte,
        PrimitiveKind::Short,
        PrimitiveKind::Int,
        PrimitiveKind::Long,
        PrimitiveKind::Float,
        PrimitiveKind::Double,
        PrimitiveKind::BigInt,
        PrimitiveKind::BigDecimal,
        PrimitiveKind::Char,
        PrimitiveKind::String,
        PrimitiveKind::DateTime,
        PrimitiveKind::Duration,
    ];
    for pk in &all {
        assert_eq!(format!("{pk}"), pk.name());
    }
}

// ===========================================================================
// 8. TypeDescriptor constants
// ===========================================================================

#[test]
fn constant_int() {
    assert_eq!(
        TypeDescriptor::INT,
        TypeDescriptor::Primitive(PrimitiveKind::Int)
    );
    assert_eq!(TypeDescriptor::INT.name(), "int");
}

#[test]
fn constant_long() {
    assert_eq!(
        TypeDescriptor::LONG,
        TypeDescriptor::Primitive(PrimitiveKind::Long)
    );
    assert_eq!(TypeDescriptor::LONG.name(), "long");
}

#[test]
fn constant_float() {
    assert_eq!(
        TypeDescriptor::FLOAT,
        TypeDescriptor::Primitive(PrimitiveKind::Float)
    );
    assert_eq!(TypeDescriptor::FLOAT.name(), "float");
}

#[test]
fn constant_double() {
    assert_eq!(
        TypeDescriptor::DOUBLE,
        TypeDescriptor::Primitive(PrimitiveKind::Double)
    );
    assert_eq!(TypeDescriptor::DOUBLE.name(), "double");
}

#[test]
fn constant_boolean() {
    assert_eq!(
        TypeDescriptor::BOOLEAN,
        TypeDescriptor::Primitive(PrimitiveKind::Boolean)
    );
    assert_eq!(TypeDescriptor::BOOLEAN.name(), "boolean");
}

#[test]
fn constant_string() {
    assert_eq!(
        TypeDescriptor::STRING,
        TypeDescriptor::Primitive(PrimitiveKind::String)
    );
    assert_eq!(TypeDescriptor::STRING.name(), "java.lang.String");
}

#[test]
fn constant_null() {
    assert_eq!(
        TypeDescriptor::NULL,
        TypeDescriptor::Primitive(PrimitiveKind::Null)
    );
    assert_eq!(TypeDescriptor::NULL.name(), "null");
}

#[test]
fn constant_object_is_named() {
    // OBJECT is a Named variant with empty name
    match &TypeDescriptor::OBJECT {
        TypeDescriptor::Named {
            name,
            type_id,
            generics,
            annotations,
        } => {
            assert!(name.is_empty());
            assert!(type_id.is_none());
            assert!(generics.is_empty());
            assert!(annotations.is_empty());
        }
        other => panic!("Expected Named variant, got {:?}", other),
    }
}

#[test]
fn constant_value_is_named() {
    // VALUE is a Named variant with empty name
    match &TypeDescriptor::VALUE {
        TypeDescriptor::Named {
            name,
            type_id,
            generics,
            annotations,
        } => {
            assert!(name.is_empty());
            assert!(type_id.is_none());
            assert!(generics.is_empty());
            assert!(annotations.is_empty());
        }
        other => panic!("Expected Named variant, got {:?}", other),
    }
}

// ===========================================================================
// 9. TypeDescriptor::from_type_name()
// ===========================================================================

#[test]
fn from_type_name_creates_named() {
    let td = TypeDescriptor::from_type_name("ArrayList");
    match &td {
        TypeDescriptor::Named {
            type_id,
            name,
            generics,
            annotations,
        } => {
            assert!(type_id.is_none());
            assert_eq!(name, "ArrayList");
            assert!(generics.is_empty());
            assert!(annotations.is_empty());
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn from_type_name_with_string() {
    let td = TypeDescriptor::from_type_name("HashMap".to_string());
    assert_eq!(td.name(), "HashMap");
}

#[test]
fn from_type_name_with_static_str() {
    let td = TypeDescriptor::from_type_name("MyType");
    assert_eq!(td.name(), "MyType");
}

// ===========================================================================
// 10. TypeDescriptor::from_type_id()
// ===========================================================================

#[test]
fn from_type_id_string() {
    let td = TypeDescriptor::from_type_id::<String>();
    match &td {
        TypeDescriptor::Named {
            type_id,
            name,
            generics,
            annotations,
        } => {
            assert_eq!(*type_id, Some(TypeId::of::<String>()));
            // type_name::<String>() varies by platform but should contain "String"
            assert!(name.contains("String"), "name was: {}", name);
            assert!(generics.is_empty());
            assert!(annotations.is_empty());
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn from_type_id_i32() {
    let td = TypeDescriptor::from_type_id::<i32>();
    match &td {
        TypeDescriptor::Named { type_id, name, .. } => {
            assert_eq!(*type_id, Some(TypeId::of::<i32>()));
            assert!(name.contains("i32"), "name was: {}", name);
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn from_type_id_vec() {
    let td = TypeDescriptor::from_type_id::<Vec<i32>>();
    match &td {
        TypeDescriptor::Named { type_id, name, .. } => {
            assert_eq!(*type_id, Some(TypeId::of::<Vec<i32>>()));
            assert!(name.contains("Vec"), "name was: {}", name);
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

// ===========================================================================
// 11. TypeDescriptor::with_generic()
// ===========================================================================

#[test]
fn with_generic_adds_generic_to_named() {
    let td = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT);
    match &td {
        TypeDescriptor::Named { generics, .. } => {
            assert_eq!(generics.len(), 1);
            assert_eq!(generics[0], TypeDescriptor::INT);
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn with_generic_multiple() {
    let td = TypeDescriptor::from_type_name("Map")
        .with_generic(TypeDescriptor::STRING)
        .with_generic(TypeDescriptor::INT);
    match &td {
        TypeDescriptor::Named { generics, .. } => {
            assert_eq!(generics.len(), 2);
            assert_eq!(generics[0], TypeDescriptor::STRING);
            assert_eq!(generics[1], TypeDescriptor::INT);
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn with_generic_on_primitive_is_noop() {
    // with_generic only mutates Named variants; on Primitive it is a no-op
    let td = TypeDescriptor::INT.with_generic(TypeDescriptor::STRING);
    assert_eq!(td, TypeDescriptor::INT);
}

// ===========================================================================
// 12. TypeDescriptor::with_annotation()
// ===========================================================================

#[test]
fn with_annotation_adds_to_named() {
    let td = TypeDescriptor::from_type_name("MyClass").with_annotation("Nullable");
    match &td {
        TypeDescriptor::Named { annotations, .. } => {
            assert_eq!(annotations.len(), 1);
            assert_eq!(annotations[0], "Nullable");
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn with_annotation_multiple() {
    let td = TypeDescriptor::from_type_name("MyClass")
        .with_annotation("Nullable")
        .with_annotation("Deprecated");
    match &td {
        TypeDescriptor::Named { annotations, .. } => {
            assert_eq!(annotations.len(), 2);
            assert_eq!(annotations[0], "Nullable");
            assert_eq!(annotations[1], "Deprecated");
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn with_annotation_on_primitive_is_noop() {
    let td = TypeDescriptor::INT.with_annotation("Nullable");
    assert_eq!(td, TypeDescriptor::INT);
}

#[test]
fn with_generic_and_annotation_chaining() {
    let td = TypeDescriptor::from_type_name("List")
        .with_generic(TypeDescriptor::STRING)
        .with_annotation("Nullable");
    match &td {
        TypeDescriptor::Named {
            generics,
            annotations,
            ..
        } => {
            assert_eq!(generics.len(), 1);
            assert_eq!(generics[0], TypeDescriptor::STRING);
            assert_eq!(annotations.len(), 1);
            assert_eq!(annotations[0], "Nullable");
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

// ===========================================================================
// 13. TypeDescriptor::name() for all variants
// ===========================================================================

#[test]
fn name_primitive_uses_primitive_kind_name() {
    assert_eq!(TypeDescriptor::Primitive(PrimitiveKind::Int).name(), "int");
    assert_eq!(
        TypeDescriptor::Primitive(PrimitiveKind::Null).name(),
        "null"
    );
    assert_eq!(
        TypeDescriptor::Primitive(PrimitiveKind::BigInt).name(),
        "java.math.BigInteger"
    );
    assert_eq!(
        TypeDescriptor::Primitive(PrimitiveKind::Duration).name(),
        "java.time.Duration"
    );
}

#[test]
fn name_array_format() {
    let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert_eq!(arr.name(), "int[]");
}

#[test]
fn name_array_nested() {
    let inner = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let outer = TypeDescriptor::Array(Box::new(inner));
    assert_eq!(outer.name(), "int[][]");
}

#[test]
fn name_map_format() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(map.name(), "java.util.Map<java.lang.String, int>");
}

#[test]
fn name_named_without_generics() {
    let td = TypeDescriptor::from_type_name("ArrayList");
    assert_eq!(td.name(), "ArrayList");
}

#[test]
fn name_named_with_one_generic() {
    let td = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::STRING);
    assert_eq!(td.name(), "List<java.lang.String>");
}

#[test]
fn name_named_with_multiple_generics() {
    let td = TypeDescriptor::from_type_name("Map")
        .with_generic(TypeDescriptor::STRING)
        .with_generic(TypeDescriptor::INT);
    assert_eq!(td.name(), "Map<java.lang.String,int>");
}

#[test]
fn name_named_with_nested_generic() {
    let inner_list = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT);
    let td = TypeDescriptor::from_type_name("Map")
        .with_generic(TypeDescriptor::STRING)
        .with_generic(inner_list);
    assert_eq!(td.name(), "Map<java.lang.String,List<int>>");
}

// ===========================================================================
// 14. TypeDescriptor::is_primitive() and is_nullable_primitive()
// ===========================================================================

#[test]
fn is_primitive_true_for_all_primitive_variants() {
    let all_kinds = [
        PrimitiveKind::Null,
        PrimitiveKind::Boolean,
        PrimitiveKind::Byte,
        PrimitiveKind::Short,
        PrimitiveKind::Int,
        PrimitiveKind::Long,
        PrimitiveKind::Float,
        PrimitiveKind::Double,
        PrimitiveKind::BigInt,
        PrimitiveKind::BigDecimal,
        PrimitiveKind::Char,
        PrimitiveKind::String,
        PrimitiveKind::DateTime,
        PrimitiveKind::Duration,
    ];
    for pk in &all_kinds {
        let td = TypeDescriptor::Primitive(*pk);
        assert!(td.is_primitive(), "Expected is_primitive for {:?}", pk);
    }
}

#[test]
fn is_primitive_false_for_non_primitive() {
    assert!(!TypeDescriptor::from_type_name("String").is_primitive());
    assert!(!TypeDescriptor::Array(Box::new(TypeDescriptor::INT)).is_primitive());
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert!(!map.is_primitive());
}

#[test]
fn is_nullable_primitive_only_for_null() {
    assert!(TypeDescriptor::NULL.is_nullable_primitive());
    assert!(!TypeDescriptor::INT.is_nullable_primitive());
    assert!(!TypeDescriptor::BOOLEAN.is_nullable_primitive());
    assert!(!TypeDescriptor::from_type_name("null").is_nullable_primitive());
}

// ===========================================================================
// 15. TypeDescriptor::new() convenience factory
// ===========================================================================

#[test]
fn new_int() {
    assert_eq!(TypeDescriptor::new("int"), TypeDescriptor::INT);
}

#[test]
fn new_integer() {
    assert_eq!(TypeDescriptor::new("Integer"), TypeDescriptor::INT);
}

#[test]
fn new_long() {
    assert_eq!(TypeDescriptor::new("long"), TypeDescriptor::LONG);
}

#[test]
fn new_long_uppercase() {
    assert_eq!(TypeDescriptor::new("Long"), TypeDescriptor::LONG);
}

#[test]
fn new_float() {
    assert_eq!(TypeDescriptor::new("float"), TypeDescriptor::FLOAT);
}

#[test]
fn new_float_uppercase() {
    assert_eq!(TypeDescriptor::new("Float"), TypeDescriptor::FLOAT);
}

#[test]
fn new_double() {
    assert_eq!(TypeDescriptor::new("double"), TypeDescriptor::DOUBLE);
}

#[test]
fn new_double_uppercase() {
    assert_eq!(TypeDescriptor::new("Double"), TypeDescriptor::DOUBLE);
}

#[test]
fn new_boolean() {
    assert_eq!(TypeDescriptor::new("boolean"), TypeDescriptor::BOOLEAN);
}

#[test]
fn new_boolean_uppercase() {
    assert_eq!(TypeDescriptor::new("Boolean"), TypeDescriptor::BOOLEAN);
}

#[test]
fn new_string_uppercase() {
    assert_eq!(TypeDescriptor::new("String"), TypeDescriptor::STRING);
}

#[test]
fn new_object() {
    assert_eq!(TypeDescriptor::new("object"), TypeDescriptor::OBJECT);
}

#[test]
fn new_value() {
    assert_eq!(TypeDescriptor::new("value"), TypeDescriptor::VALUE);
}

#[test]
fn new_null() {
    assert_eq!(TypeDescriptor::new("null"), TypeDescriptor::NULL);
}

#[test]
fn new_unknown_falls_back_to_named() {
    let td = TypeDescriptor::new("CustomType");
    match &td {
        TypeDescriptor::Named {
            type_id,
            name,
            generics,
            annotations,
        } => {
            assert!(type_id.is_none());
            assert_eq!(name, "CustomType");
            assert!(generics.is_empty());
            assert!(annotations.is_empty());
        }
        other => panic!("Expected Named, got {:?}", other),
    }
}

#[test]
fn new_empty_string_is_named() {
    let td = TypeDescriptor::new("");
    match &td {
        TypeDescriptor::Named { name, .. } => assert!(name.is_empty()),
        other => panic!("Expected Named, got {:?}", other),
    }
}

// ===========================================================================
// 16. TypeDescriptor::is_assignable_from()
// ===========================================================================

#[test]
fn assignable_object_accepts_all() {
    let object = TypeDescriptor::OBJECT;
    assert!(object.is_assignable_from(&TypeDescriptor::INT));
    assert!(object.is_assignable_from(&TypeDescriptor::LONG));
    assert!(object.is_assignable_from(&TypeDescriptor::FLOAT));
    assert!(object.is_assignable_from(&TypeDescriptor::DOUBLE));
    assert!(object.is_assignable_from(&TypeDescriptor::BOOLEAN));
    assert!(object.is_assignable_from(&TypeDescriptor::STRING));
    assert!(object.is_assignable_from(&TypeDescriptor::NULL));
    assert!(object.is_assignable_from(&TypeDescriptor::Array(Box::new(TypeDescriptor::INT))));
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert!(object.is_assignable_from(&map));
    assert!(object.is_assignable_from(&TypeDescriptor::from_type_name("Foo")));
}

#[test]
fn assignable_same_type() {
    assert!(TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::INT));
    assert!(TypeDescriptor::LONG.is_assignable_from(&TypeDescriptor::LONG));
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::STRING));
    assert!(TypeDescriptor::BOOLEAN.is_assignable_from(&TypeDescriptor::BOOLEAN));
}

#[test]
fn assignable_numeric_widening() {
    // Int accepts Byte, Short
    assert!(
        TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Byte))
    );
    assert!(
        TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Short))
    );
    assert!(TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::INT));

    // Long accepts Byte, Short, Int
    assert!(
        TypeDescriptor::LONG.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Byte))
    );
    assert!(
        TypeDescriptor::LONG.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Short))
    );
    assert!(TypeDescriptor::LONG.is_assignable_from(&TypeDescriptor::INT));
    assert!(TypeDescriptor::LONG.is_assignable_from(&TypeDescriptor::LONG));

    // Float accepts Byte, Short, Int, Long (width 6 > 4)
    assert!(
        TypeDescriptor::FLOAT.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Byte))
    );
    assert!(TypeDescriptor::FLOAT.is_assignable_from(&TypeDescriptor::INT));
    assert!(TypeDescriptor::FLOAT.is_assignable_from(&TypeDescriptor::LONG));
    assert!(TypeDescriptor::FLOAT.is_assignable_from(&TypeDescriptor::FLOAT));

    // Double accepts everything numeric below it
    assert!(
        TypeDescriptor::DOUBLE.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Byte))
    );
    assert!(TypeDescriptor::DOUBLE.is_assignable_from(&TypeDescriptor::INT));
    assert!(TypeDescriptor::DOUBLE.is_assignable_from(&TypeDescriptor::LONG));
    assert!(TypeDescriptor::DOUBLE.is_assignable_from(&TypeDescriptor::FLOAT));
    assert!(TypeDescriptor::DOUBLE.is_assignable_from(&TypeDescriptor::DOUBLE));
}

#[test]
fn assignable_numeric_narrowing_rejected() {
    // Int does NOT accept Long
    assert!(!TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::LONG));
    // Byte does NOT accept Int
    assert!(
        !TypeDescriptor::Primitive(PrimitiveKind::Byte).is_assignable_from(&TypeDescriptor::INT)
    );
    // Float does NOT accept Double
    assert!(!TypeDescriptor::FLOAT.is_assignable_from(&TypeDescriptor::DOUBLE));
}

#[test]
fn assignable_string_accepts_same_type() {
    // String == String via same-type arm
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::STRING));
}

#[test]
fn assignable_string_accepts_non_primitive_sources() {
    // The "String accepts all" match arm only fires when src is NOT a Primitive,
    // because the Primitive-Primitive numeric widening arm matches first.
    assert!(
        TypeDescriptor::STRING
            .is_assignable_from(&TypeDescriptor::Array(Box::new(TypeDescriptor::INT)))
    );
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::from_type_name("Foo")));
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert!(TypeDescriptor::STRING.is_assignable_from(&map));
}

#[test]
fn assignable_string_accepts_numeric_primitives() {
    // String accepts all types (toString conversion) — the String arm fires
    // before the Primitive-Primitive numeric widening arm.
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::INT));
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::LONG));
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::FLOAT));
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::DOUBLE));
    assert!(
        TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Byte))
    );
}

#[test]
fn assignable_string_accepts_non_numeric_primitives_via_width_zero() {
    // String (width 0) and Boolean/Null (width 0): 0 >= 0 is true via numeric widening arm.
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::BOOLEAN));
    assert!(TypeDescriptor::STRING.is_assignable_from(&TypeDescriptor::NULL));
}

#[test]
fn assignable_map_compatible() {
    let map_a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let map_b = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert!(map_a.is_assignable_from(&map_b));
}

#[test]
fn assignable_map_incompatible_value_type() {
    let map_a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let map_b = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::LONG),
    );
    // INT does not accept LONG, so map assignment fails
    assert!(!map_a.is_assignable_from(&map_b));
}

#[test]
fn assignable_map_incompatible_key_type() {
    let map_a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::INT),
        Box::new(TypeDescriptor::STRING),
    );
    let map_b = TypeDescriptor::Map(
        Box::new(TypeDescriptor::LONG),
        Box::new(TypeDescriptor::STRING),
    );
    // INT does not accept LONG key
    assert!(!map_a.is_assignable_from(&map_b));
}

#[test]
fn assignable_array_compatible() {
    let arr_a = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let arr_b = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert!(arr_a.is_assignable_from(&arr_b));
}

#[test]
fn assignable_array_widening() {
    let arr_long = TypeDescriptor::Array(Box::new(TypeDescriptor::LONG));
    let arr_int = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    // LONG accepts INT
    assert!(arr_long.is_assignable_from(&arr_int));
    // INT does NOT accept LONG
    assert!(!arr_int.is_assignable_from(&arr_long));
}

#[test]
fn assignable_named_same_name() {
    let a = TypeDescriptor::from_type_name("ArrayList");
    let b = TypeDescriptor::from_type_name("ArrayList");
    assert!(a.is_assignable_from(&b));
}

#[test]
fn assignable_named_different_name() {
    let a = TypeDescriptor::from_type_name("ArrayList");
    let b = TypeDescriptor::from_type_name("LinkedList");
    assert!(!a.is_assignable_from(&b));
}

#[test]
fn assignable_cross_variant_rejected() {
    // Primitive vs Named
    assert!(!TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::from_type_name("Integer")));
    // Primitive vs Array
    assert!(
        !TypeDescriptor::INT
            .is_assignable_from(&TypeDescriptor::Array(Box::new(TypeDescriptor::INT)))
    );
    // Array vs Map
    let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert!(!arr.is_assignable_from(&map));
    assert!(!map.is_assignable_from(&arr));
}

#[test]
fn assignable_map_accepts_wider_value() {
    let map_wide = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::DOUBLE),
    );
    let map_narrow = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    // DOUBLE accepts INT, so this should work
    assert!(map_wide.is_assignable_from(&map_narrow));
}

// ===========================================================================
// 17. TypeDescriptor::narrow()
// ===========================================================================

#[test]
fn narrow_int() {
    let result = TypeDescriptor::OBJECT.narrow("int");
    assert_eq!(result, TypeDescriptor::INT);
}

#[test]
fn narrow_long() {
    let result = TypeDescriptor::OBJECT.narrow("long");
    assert_eq!(result, TypeDescriptor::LONG);
}

#[test]
fn narrow_boolean() {
    let result = TypeDescriptor::OBJECT.narrow("boolean");
    assert_eq!(result, TypeDescriptor::BOOLEAN);
}

#[test]
fn narrow_double() {
    let result = TypeDescriptor::OBJECT.narrow("double");
    assert_eq!(result, TypeDescriptor::DOUBLE);
}

#[test]
fn narrow_string() {
    let result = TypeDescriptor::OBJECT.narrow("string");
    assert_eq!(result, TypeDescriptor::STRING);
}

#[test]
fn narrow_unknown_returns_self() {
    let original = TypeDescriptor::from_type_name("MyType");
    let result = original.narrow("unknown");
    assert_eq!(result, TypeDescriptor::from_type_name("MyType"));
}

#[test]
fn narrow_empty_returns_self() {
    let original = TypeDescriptor::INT;
    let result = original.narrow("");
    assert_eq!(result, TypeDescriptor::INT);
}

// ===========================================================================
// 18. get_map_key_type / get_map_value_type
// ===========================================================================

#[test]
fn get_map_key_type_on_map() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(map.get_map_key_type(), Some(&TypeDescriptor::STRING));
}

#[test]
fn get_map_value_type_on_map() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(map.get_map_value_type(), Some(&TypeDescriptor::INT));
}

#[test]
fn get_map_key_type_on_non_map() {
    assert_eq!(TypeDescriptor::INT.get_map_key_type(), None);
    assert_eq!(
        TypeDescriptor::Array(Box::new(TypeDescriptor::INT)).get_map_key_type(),
        None
    );
    assert_eq!(
        TypeDescriptor::from_type_name("Foo").get_map_key_type(),
        None
    );
}

#[test]
fn get_map_value_type_on_non_map() {
    assert_eq!(TypeDescriptor::INT.get_map_value_type(), None);
    assert_eq!(
        TypeDescriptor::Array(Box::new(TypeDescriptor::INT)).get_map_value_type(),
        None
    );
    assert_eq!(
        TypeDescriptor::from_type_name("Foo").get_map_value_type(),
        None
    );
}

// ===========================================================================
// 19. get_element_type
// ===========================================================================

#[test]
fn get_element_type_on_array() {
    let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert_eq!(arr.get_element_type(), Some(&TypeDescriptor::INT));
}

#[test]
fn get_element_type_on_nested_array() {
    let inner = TypeDescriptor::Array(Box::new(TypeDescriptor::STRING));
    let outer = TypeDescriptor::Array(Box::new(inner));
    let elem = outer.get_element_type().unwrap();
    assert!(elem.is_array());
    assert_eq!(elem.get_element_type(), Some(&TypeDescriptor::STRING));
}

#[test]
fn get_element_type_on_non_array() {
    assert_eq!(TypeDescriptor::INT.get_element_type(), None);
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(map.get_element_type(), None);
    assert_eq!(
        TypeDescriptor::from_type_name("Foo").get_element_type(),
        None
    );
}

// ===========================================================================
// 20. is_map() and is_array()
// ===========================================================================

#[test]
fn is_map_true_for_map() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert!(map.is_map());
}

#[test]
fn is_map_false_for_non_map() {
    assert!(!TypeDescriptor::INT.is_map());
    assert!(!TypeDescriptor::Array(Box::new(TypeDescriptor::INT)).is_map());
    assert!(!TypeDescriptor::from_type_name("HashMap").is_map());
    assert!(!TypeDescriptor::NULL.is_map());
}

#[test]
fn is_array_true_for_array() {
    let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert!(arr.is_array());
}

#[test]
fn is_array_false_for_non_array() {
    assert!(!TypeDescriptor::INT.is_array());
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert!(!map.is_array());
    assert!(!TypeDescriptor::from_type_name("ArrayList").is_array());
    assert!(!TypeDescriptor::NULL.is_array());
}

// ===========================================================================
// 21. type_id()
// ===========================================================================

#[test]
fn type_id_on_named_with_type_id() {
    let td = TypeDescriptor::from_type_id::<String>();
    assert_eq!(td.type_id(), Some(TypeId::of::<String>()));
}

#[test]
fn type_id_on_named_without_type_id() {
    let td = TypeDescriptor::from_type_name("Foo");
    assert_eq!(td.type_id(), None);
}

#[test]
fn type_id_on_primitive() {
    assert_eq!(TypeDescriptor::INT.type_id(), None);
    assert_eq!(TypeDescriptor::STRING.type_id(), None);
}

#[test]
fn type_id_on_array() {
    let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert_eq!(arr.type_id(), None);
}

#[test]
fn type_id_on_map() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(map.type_id(), None);
}

// ===========================================================================
// 22. primitive_kind()
// ===========================================================================

#[test]
fn primitive_kind_on_primitive() {
    assert_eq!(
        TypeDescriptor::INT.primitive_kind(),
        Some(PrimitiveKind::Int)
    );
    assert_eq!(
        TypeDescriptor::LONG.primitive_kind(),
        Some(PrimitiveKind::Long)
    );
    assert_eq!(
        TypeDescriptor::BOOLEAN.primitive_kind(),
        Some(PrimitiveKind::Boolean)
    );
    assert_eq!(
        TypeDescriptor::STRING.primitive_kind(),
        Some(PrimitiveKind::String)
    );
    assert_eq!(
        TypeDescriptor::NULL.primitive_kind(),
        Some(PrimitiveKind::Null)
    );
}

#[test]
fn primitive_kind_on_non_primitive() {
    assert_eq!(TypeDescriptor::from_type_name("Foo").primitive_kind(), None);
    assert_eq!(
        TypeDescriptor::Array(Box::new(TypeDescriptor::INT)).primitive_kind(),
        None
    );
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(map.primitive_kind(), None);
}

// ===========================================================================
// 23. PartialEq between all variant combinations
// ===========================================================================

#[test]
fn partial_eq_primitive_same() {
    assert_eq!(
        TypeDescriptor::INT,
        TypeDescriptor::Primitive(PrimitiveKind::Int)
    );
    assert_eq!(
        TypeDescriptor::STRING,
        TypeDescriptor::Primitive(PrimitiveKind::String)
    );
}

#[test]
fn partial_eq_primitive_different() {
    assert_ne!(TypeDescriptor::INT, TypeDescriptor::LONG);
    assert_ne!(TypeDescriptor::INT, TypeDescriptor::BOOLEAN);
    assert_ne!(TypeDescriptor::NULL, TypeDescriptor::INT);
}

#[test]
fn partial_eq_array_same() {
    let a = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let b = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert_eq!(a, b);
}

#[test]
fn partial_eq_array_different() {
    let a = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let b = TypeDescriptor::Array(Box::new(TypeDescriptor::LONG));
    assert_ne!(a, b);
}

#[test]
fn partial_eq_map_same() {
    let a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let b = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(a, b);
}

#[test]
fn partial_eq_map_different_value() {
    let a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let b = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::LONG),
    );
    assert_ne!(a, b);
}

#[test]
fn partial_eq_map_different_key() {
    let a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let b = TypeDescriptor::Map(Box::new(TypeDescriptor::INT), Box::new(TypeDescriptor::INT));
    assert_ne!(a, b);
}

#[test]
fn partial_eq_named_same() {
    let a = TypeDescriptor::from_type_name("Foo");
    let b = TypeDescriptor::from_type_name("Foo");
    assert_eq!(a, b);
}

#[test]
fn partial_eq_named_different_name() {
    let a = TypeDescriptor::from_type_name("Foo");
    let b = TypeDescriptor::from_type_name("Bar");
    assert_ne!(a, b);
}

#[test]
fn partial_eq_named_different_generics() {
    let a = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT);
    let b = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::LONG);
    assert_ne!(a, b);
}

#[test]
fn partial_eq_named_different_annotations() {
    let a = TypeDescriptor::from_type_name("Foo").with_annotation("Nullable");
    let b = TypeDescriptor::from_type_name("Foo");
    assert_ne!(a, b);
}

#[test]
fn partial_eq_cross_variant_all_false() {
    let primitive = TypeDescriptor::INT;
    let array = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let named = TypeDescriptor::from_type_name("int");

    // All cross-variant comparisons must be unequal
    assert_ne!(primitive, array);
    assert_ne!(primitive, map);
    assert_ne!(primitive, named);
    assert_ne!(array, map);
    assert_ne!(array, named);
    assert_ne!(map, named);
}

// ===========================================================================
// 24. Eq trait
// ===========================================================================

#[test]
fn eq_trait_is_reflexive() {
    let types = [
        TypeDescriptor::INT,
        TypeDescriptor::LONG,
        TypeDescriptor::STRING,
        TypeDescriptor::NULL,
        TypeDescriptor::Array(Box::new(TypeDescriptor::INT)),
        TypeDescriptor::Map(
            Box::new(TypeDescriptor::STRING),
            Box::new(TypeDescriptor::INT),
        ),
    ];
    for td in &types {
        assert_eq!(td, td);
    }
}

#[test]
fn eq_trait_is_symmetric() {
    let a = TypeDescriptor::from_type_name("Foo");
    let b = TypeDescriptor::from_type_name("Foo");
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ===========================================================================
// 25. Hash consistency (equal values must hash equal)
// ===========================================================================

#[test]
fn hash_primitive_equal_values() {
    assert_eq!(
        hash_of(&TypeDescriptor::INT),
        hash_of(&TypeDescriptor::Primitive(PrimitiveKind::Int))
    );
    assert_eq!(
        hash_of(&TypeDescriptor::STRING),
        hash_of(&TypeDescriptor::Primitive(PrimitiveKind::String))
    );
}

#[test]
fn hash_array_equal_values() {
    let a = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let b = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_map_equal_values() {
    let a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let b = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_named_equal_values() {
    let a = TypeDescriptor::from_type_name("Foo");
    let b = TypeDescriptor::from_type_name("Foo");
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_named_with_generics_equal() {
    let a = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT);
    let b = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::INT);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn hash_different_values_differ() {
    // Not guaranteed by contract but practically should differ
    let h_int = hash_of(&TypeDescriptor::INT);
    let h_long = hash_of(&TypeDescriptor::LONG);
    assert_ne!(h_int, h_long);
}

#[test]
fn hash_works_in_hashset() {
    let mut set = std::collections::HashSet::new();
    set.insert(TypeDescriptor::INT);
    set.insert(TypeDescriptor::LONG);
    set.insert(TypeDescriptor::INT); // duplicate
    assert_eq!(set.len(), 2);
    assert!(set.contains(&TypeDescriptor::INT));
    assert!(set.contains(&TypeDescriptor::LONG));
}

// ===========================================================================
// 26. Display for TypeDescriptor
// ===========================================================================

#[test]
fn display_primitive() {
    assert_eq!(format!("{}", TypeDescriptor::INT), "int");
    assert_eq!(format!("{}", TypeDescriptor::STRING), "java.lang.String");
    assert_eq!(format!("{}", TypeDescriptor::NULL), "null");
}

#[test]
fn display_array() {
    let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    assert_eq!(format!("{}", arr), "int[]");
}

#[test]
fn display_map() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    assert_eq!(format!("{}", map), "java.util.Map<java.lang.String, int>");
}

#[test]
fn display_named() {
    let td = TypeDescriptor::from_type_name("ArrayList");
    assert_eq!(format!("{}", td), "ArrayList");
}

#[test]
fn display_named_with_generics() {
    let td = TypeDescriptor::from_type_name("List").with_generic(TypeDescriptor::STRING);
    assert_eq!(format!("{}", td), "List<java.lang.String>");
}

// ===========================================================================
// 27. Debug trait
// ===========================================================================

#[test]
fn debug_primitive() {
    let dbg = format!("{:?}", TypeDescriptor::INT);
    assert!(dbg.contains("Primitive"), "Debug output: {}", dbg);
    assert!(dbg.contains("Int"), "Debug output: {}", dbg);
}

#[test]
fn debug_array() {
    let arr = TypeDescriptor::Array(Box::new(TypeDescriptor::INT));
    let dbg = format!("{:?}", arr);
    assert!(dbg.contains("Array"), "Debug output: {}", dbg);
}

#[test]
fn debug_map() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let dbg = format!("{:?}", map);
    assert!(dbg.contains("Map"), "Debug output: {}", dbg);
}

#[test]
fn debug_named() {
    let td = TypeDescriptor::from_type_name("Foo");
    let dbg = format!("{:?}", td);
    assert!(dbg.contains("Named"), "Debug output: {}", dbg);
    assert!(dbg.contains("Foo"), "Debug output: {}", dbg);
}

// ===========================================================================
// 28. Clone trait
// ===========================================================================

#[test]
fn clone_primitive() {
    let a = TypeDescriptor::INT;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_array() {
    let a = TypeDescriptor::Array(Box::new(TypeDescriptor::STRING));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_map() {
    let a = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::INT),
    );
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn clone_named_with_generics() {
    let a = TypeDescriptor::from_type_name("List")
        .with_generic(TypeDescriptor::INT)
        .with_annotation("Nullable");
    let b = a.clone();
    assert_eq!(a, b);
}

// ===========================================================================
// 29. PrimitiveKind derive traits
// ===========================================================================

#[test]
fn primitive_kind_copy() {
    let a = PrimitiveKind::Int;
    let b = a; // copy
    assert_eq!(a, b);
}

#[test]
fn primitive_kind_debug() {
    let dbg = format!("{:?}", PrimitiveKind::Double);
    assert_eq!(dbg, "Double");
}

#[test]
fn primitive_kind_hash() {
    let h1 = hash_of(&PrimitiveKind::Int);
    let h2 = hash_of(&PrimitiveKind::Int);
    assert_eq!(h1, h2);

    let h3 = hash_of(&PrimitiveKind::Long);
    assert_ne!(h1, h3);
}

#[test]
fn primitive_kind_eq_reflexive() {
    let kinds = [
        PrimitiveKind::Null,
        PrimitiveKind::Boolean,
        PrimitiveKind::Byte,
        PrimitiveKind::Short,
        PrimitiveKind::Int,
        PrimitiveKind::Long,
        PrimitiveKind::Float,
        PrimitiveKind::Double,
        PrimitiveKind::BigInt,
        PrimitiveKind::BigDecimal,
        PrimitiveKind::Char,
        PrimitiveKind::String,
        PrimitiveKind::DateTime,
        PrimitiveKind::Duration,
    ];
    for k in &kinds {
        assert_eq!(*k, *k);
    }
}

#[test]
fn primitive_kind_all_distinct() {
    let kinds = [
        PrimitiveKind::Null,
        PrimitiveKind::Boolean,
        PrimitiveKind::Byte,
        PrimitiveKind::Short,
        PrimitiveKind::Int,
        PrimitiveKind::Long,
        PrimitiveKind::Float,
        PrimitiveKind::Double,
        PrimitiveKind::BigInt,
        PrimitiveKind::BigDecimal,
        PrimitiveKind::Char,
        PrimitiveKind::String,
        PrimitiveKind::DateTime,
        PrimitiveKind::Duration,
    ];
    for i in 0..kinds.len() {
        for j in (i + 1)..kinds.len() {
            assert_ne!(
                kinds[i], kinds[j],
                "kinds[{}] == kinds[{}]: {:?}",
                i, j, kinds[i]
            );
        }
    }
}

// ===========================================================================
// 30. Deeply nested structures
// ===========================================================================

#[test]
fn deeply_nested_array_name() {
    let mut td = TypeDescriptor::INT;
    for _ in 0..5 {
        td = TypeDescriptor::Array(Box::new(td));
    }
    assert_eq!(td.name(), "int[][][][][]");
}

#[test]
fn deeply_nested_array_get_element_type() {
    let mut td = TypeDescriptor::INT;
    for _ in 0..5 {
        td = TypeDescriptor::Array(Box::new(td));
    }
    // Walk down 5 levels
    let mut current = &td;
    for _ in 0..5 {
        current = current.get_element_type().expect("should be array");
    }
    assert_eq!(*current, TypeDescriptor::INT);
}

#[test]
fn map_with_array_values() {
    let map = TypeDescriptor::Map(
        Box::new(TypeDescriptor::STRING),
        Box::new(TypeDescriptor::Array(Box::new(TypeDescriptor::INT))),
    );
    assert_eq!(map.name(), "java.util.Map<java.lang.String, int[]>");
    assert!(map.is_map());
    let val = map.get_map_value_type().unwrap();
    assert!(val.is_array());
    assert_eq!(val.get_element_type(), Some(&TypeDescriptor::INT));
}

// ===========================================================================
// 31. Edge cases for is_assignable_from with Double special rules
// ===========================================================================

#[test]
fn assignable_double_accepts_float_special_rule() {
    // Double accepts Float via numeric_width(7) >= numeric_width(6)
    assert!(TypeDescriptor::DOUBLE.is_assignable_from(&TypeDescriptor::FLOAT));
}

#[test]
fn assignable_float_accepts_float_same_type() {
    // Same type match
    assert!(TypeDescriptor::FLOAT.is_assignable_from(&TypeDescriptor::FLOAT));
}

#[test]
fn assignable_non_numeric_primitives_have_width_zero() {
    // Non-numeric primitives (Boolean, Char, Null, DateTime, Duration, String) all have
    // numeric_width() == 0. Because the widening check is `dp.numeric_width() >= sp.numeric_width()`,
    // any numeric primitive (width > 0) accepts them (3 >= 0 is true).
    assert!(TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::BOOLEAN));
    assert!(
        TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Char))
    );
    assert!(TypeDescriptor::INT.is_assignable_from(&TypeDescriptor::NULL));
    assert!(
        TypeDescriptor::LONG
            .is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::DateTime))
    );
    assert!(
        TypeDescriptor::DOUBLE
            .is_assignable_from(&TypeDescriptor::Primitive(PrimitiveKind::Duration))
    );
}

#[test]
fn assignable_non_numeric_cannot_accept_numeric() {
    // Reversed: Boolean (width 0) cannot accept Int (width 3) because 0 >= 3 is false.
    assert!(!TypeDescriptor::BOOLEAN.is_assignable_from(&TypeDescriptor::INT));
    assert!(
        !TypeDescriptor::Primitive(PrimitiveKind::Char).is_assignable_from(&TypeDescriptor::LONG)
    );
    // DateTime cannot accept Long
    assert!(
        !TypeDescriptor::Primitive(PrimitiveKind::DateTime)
            .is_assignable_from(&TypeDescriptor::LONG)
    );
}

#[test]
fn assignable_non_numeric_same_type() {
    // Same-type arm catches these before numeric widening
    assert!(TypeDescriptor::BOOLEAN.is_assignable_from(&TypeDescriptor::BOOLEAN));
    assert!(TypeDescriptor::NULL.is_assignable_from(&TypeDescriptor::NULL));
}

#[test]
fn assignable_named_vs_primitive() {
    // Named "int" is not the same as Primitive(Int)
    let named_int = TypeDescriptor::from_type_name("int");
    assert!(!named_int.is_assignable_from(&TypeDescriptor::INT));
}

// ===========================================================================
// 32. OBJECT and VALUE equality/behavior
// ===========================================================================

#[test]
fn object_and_value_are_equal() {
    // Both have empty name, no type_id, no generics, no annotations
    assert_eq!(TypeDescriptor::OBJECT, TypeDescriptor::VALUE);
}

#[test]
fn new_object_equals_constant() {
    // TypeDescriptor::new("object") returns OBJECT constant
    assert_eq!(TypeDescriptor::new("object"), TypeDescriptor::OBJECT);
}

#[test]
fn new_value_equals_constant() {
    assert_eq!(TypeDescriptor::new("value"), TypeDescriptor::VALUE);
}
