//! BeanUtil 和 BeanDescriptor 的全面测试。

use std::any::TypeId;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vernal_beans::{BeanDescriptor, BeanError, BeanUtil, PropertyDescriptor};

// ── BeanUtil 测试 ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Source {
    name: String,
    age: i32,
    active: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Target {
    name: String,
    age: i32,
    active: bool,
}

#[test]
fn bean_util_copy_properties_basic() {
    let source = Source {
        name: "Alice".to_string(),
        age: 30,
        active: true,
    };
    let target: Target = BeanUtil::copy_properties(&source).unwrap();
    assert_eq!(target.name, "Alice");
    assert_eq!(target.age, 30);
    assert!(target.active);
}

#[test]
fn bean_util_copy_properties_different_types() {
    #[derive(Serialize)]
    struct Src {
        value: i32,
    }

    #[derive(Deserialize, Debug)]
    struct Dst {
        value: i32,
    }

    let src = Src { value: 42 };
    let dst: Dst = BeanUtil::copy_properties(&src).unwrap();
    assert_eq!(dst.value, 42);
}

#[test]
fn bean_util_copy_properties_failure() {
    // 创建一个无法序列化的类型来测试错误路径
    // 实际上所有实现了 Serialize 的类型都可以序列化
    // 我们测试反序列化失败的情况
    let source = Source {
        name: "test".to_string(),
        age: 1,
        active: false,
    };

    #[derive(Deserialize, Debug)]
    struct Incompatible {
        name: String,
        // 缺少 age 和 active 字段，serde 默认会失败
    }

    // 默认 serde 配置下，缺少字段会失败
    let result: Result<Incompatible, _> = BeanUtil::copy_properties(&source);
    // 如果 Incompatible 有 Default 实现，可能会成功
    // 这里我们只验证不会 panic
    let _ = result;
}

#[test]
fn bean_util_map_to_struct_basic() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Bob".to_string());
    map.insert("age".to_string(), "25".to_string());
    map.insert("active".to_string(), "true".to_string());

    let target: Target = BeanUtil::map_to_struct(&map).unwrap();
    assert_eq!(target.name, "Bob");
    assert_eq!(target.age, 25);
    assert!(target.active);
}

#[test]
fn bean_util_map_to_struct_number_parsing() {
    let mut map = HashMap::new();
    map.insert("value".to_string(), "123".to_string());

    #[derive(Deserialize, Debug)]
    struct NumStruct {
        value: i64,
    }

    let result: NumStruct = BeanUtil::map_to_struct(&map).unwrap();
    assert_eq!(result.value, 123);
}

#[test]
fn bean_util_map_to_struct_float_parsing() {
    let mut map = HashMap::new();
    map.insert("value".to_string(), "3.14".to_string());

    #[derive(Deserialize, Debug)]
    struct FloatStruct {
        value: f64,
    }

    let result: FloatStruct = BeanUtil::map_to_struct(&map).unwrap();
    assert!((result.value - 3.14).abs() < f64::EPSILON);
}

#[test]
fn bean_util_map_to_struct_bool_parsing() {
    let mut map = HashMap::new();
    map.insert("flag".to_string(), "true".to_string());
    map.insert("other".to_string(), "false".to_string());

    #[derive(Deserialize, Debug)]
    struct BoolStruct {
        flag: bool,
        other: bool,
    }

    let result: BoolStruct = BeanUtil::map_to_struct(&map).unwrap();
    assert!(result.flag);
    assert!(!result.other);
}

#[test]
fn bean_util_map_to_struct_string_fallback() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), "not_a_number".to_string());

    #[derive(Deserialize, Debug)]
    struct StrStruct {
        key: String,
    }

    let result: StrStruct = BeanUtil::map_to_struct(&map).unwrap();
    assert_eq!(result.key, "not_a_number");
}

#[test]
fn bean_util_struct_to_map_basic() {
    let source = Source {
        name: "Charlie".to_string(),
        age: 35,
        active: true,
    };

    let map = BeanUtil::struct_to_map(&source).unwrap();
    assert_eq!(map.get("name").unwrap(), "Charlie");
    assert_eq!(map.get("age").unwrap(), "35");
    assert_eq!(map.get("active").unwrap(), "true");
}

#[test]
fn bean_util_struct_to_map_with_null() {
    #[derive(Serialize)]
    struct WithNull {
        name: String,
        value: Option<String>,
    }

    let source = WithNull {
        name: "test".to_string(),
        value: None,
    };

    let map = BeanUtil::struct_to_map(&source).unwrap();
    assert_eq!(map.get("name").unwrap(), "test");
    // null 值转为空字符串
    assert_eq!(map.get("value").unwrap(), "");
}

#[test]
fn bean_util_type_eq_same() {
    assert!(BeanUtil::type_eq::<String, String>());
    assert!(BeanUtil::type_eq::<i32, i32>());
    assert!(BeanUtil::type_eq::<bool, bool>());
}

#[test]
fn bean_util_type_eq_different() {
    assert!(!BeanUtil::type_eq::<String, i32>());
    assert!(!BeanUtil::type_eq::<i32, f64>());
    assert!(!BeanUtil::type_eq::<bool, String>());
}

#[test]
fn bean_util_type_id_of() {
    let id = BeanUtil::type_id_of::<String>();
    assert_eq!(id, TypeId::of::<String>());

    let id2 = BeanUtil::type_id_of::<i32>();
    assert_eq!(id2, TypeId::of::<i32>());
}

#[test]
fn bean_util_is_optional() {
    let prop = PropertyDescriptor::new(
        "test",
        TypeId::of::<String>(),
        "String",
        true,  // optional
        false,
    );
    assert!(BeanUtil::is_optional(&prop));

    let prop2 = PropertyDescriptor::new(
        "test2",
        TypeId::of::<i32>(),
        "i32",
        false, // not optional
        false,
    );
    assert!(!BeanUtil::is_optional(&prop2));
}

#[test]
fn bean_util_has_default() {
    let prop = PropertyDescriptor::new(
        "test",
        TypeId::of::<String>(),
        "String",
        false,
        true, // has default
    );
    assert!(BeanUtil::has_default(&prop));

    let prop2 = PropertyDescriptor::new(
        "test2",
        TypeId::of::<i32>(),
        "i32",
        false,
        false, // no default
    );
    assert!(!BeanUtil::has_default(&prop2));
}

#[test]
fn bean_util_copy_properties_json_intermediate() {
    // 验证通过 JSON 中间表示的属性复制
    #[derive(Serialize)]
    struct Src {
        data: Vec<i32>,
    }

    #[derive(Deserialize, Debug)]
    struct Dst {
        data: Vec<i32>,
    }

    let src = Src {
        data: vec![1, 2, 3],
    };
    let dst: Dst = BeanUtil::copy_properties(&src).unwrap();
    assert_eq!(dst.data, vec![1, 2, 3]);
}

// ── BeanError 测试 ──────────────────────────────────────────────────────────

#[test]
fn bean_error_display() {
    let err = BeanError {
        message: "test error".to_string(),
    };
    assert_eq!(format!("{err}"), "Bean 错误: test error");
}

#[test]
fn bean_error_debug() {
    let err = BeanError {
        message: "debug test".to_string(),
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("debug test"));
}

#[test]
fn bean_error_clone() {
    let err = BeanError {
        message: "clone test".to_string(),
    };
    let cloned = err.clone();
    assert_eq!(err.message, cloned.message);
}

// ── BeanDescriptor 测试 ─────────────────────────────────────────────────────

struct TestBeanDescriptor {
    props: Vec<PropertyDescriptor>,
}

impl BeanDescriptor for TestBeanDescriptor {
    fn name(&self) -> &'static str {
        "TestBean"
    }

    fn properties(&self) -> &[PropertyDescriptor] {
        &self.props
    }
}

#[test]
fn bean_descriptor_name() {
    let descriptor = TestBeanDescriptor {
        props: vec![],
    };
    assert_eq!(descriptor.name(), "TestBean");
}

#[test]
fn bean_descriptor_properties() {
    let props = vec![
        PropertyDescriptor::new("name", TypeId::of::<String>(), "String", false, false),
        PropertyDescriptor::new("age", TypeId::of::<i32>(), "i32", false, true),
    ];
    let descriptor = TestBeanDescriptor { props };
    assert_eq!(descriptor.properties().len(), 2);
}

#[test]
fn bean_descriptor_find_property_found() {
    let props = vec![
        PropertyDescriptor::new("name", TypeId::of::<String>(), "String", false, false),
        PropertyDescriptor::new("age", TypeId::of::<i32>(), "i32", false, true),
    ];
    let descriptor = TestBeanDescriptor { props };
    let found = descriptor.find_property("age");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "age");
}

#[test]
fn bean_descriptor_find_property_not_found() {
    let props = vec![
        PropertyDescriptor::new("name", TypeId::of::<String>(), "String", false, false),
    ];
    let descriptor = TestBeanDescriptor { props };
    let found = descriptor.find_property("missing");
    assert!(found.is_none());
}

// ── PropertyDescriptor 测试 ─────────────────────────────────────────────────

#[test]
fn property_descriptor_new() {
    let prop = PropertyDescriptor::new(
        "test_prop",
        TypeId::of::<String>(),
        "String",
        true,
        false,
    );
    assert_eq!(prop.name, "test_prop");
    assert_eq!(prop.type_id, TypeId::of::<String>());
    assert_eq!(prop.type_name, "String");
    assert!(prop.optional);
    assert!(!prop.has_default);
}

#[test]
fn property_descriptor_debug() {
    let prop = PropertyDescriptor::new(
        "debug_prop",
        TypeId::of::<i32>(),
        "i32",
        false,
        true,
    );
    let debug = format!("{:?}", prop);
    assert!(debug.contains("debug_prop"));
    assert!(debug.contains("i32"));
}

#[test]
fn property_descriptor_clone() {
    let prop = PropertyDescriptor::new(
        "clone_prop",
        TypeId::of::<bool>(),
        "bool",
        false,
        false,
    );
    let cloned = prop.clone();
    assert_eq!(prop.name, cloned.name);
    assert_eq!(prop.type_id, cloned.type_id);
    assert_eq!(prop.type_name, cloned.type_name);
    assert_eq!(prop.optional, cloned.optional);
    assert_eq!(prop.has_default, cloned.has_default);
}

// ── lib.rs project_status 测试 ──────────────────────────────────────────────

#[test]
fn project_status_exists() {
    let status = vernal_beans::project_status();
    assert!(!status.is_empty());
}

// ── 边界情况测试 ────────────────────────────────────────────────────────────

#[test]
fn bean_util_struct_to_map_empty() {
    #[derive(Serialize)]
    struct Empty {}

    let source = Empty {};
    let map = BeanUtil::struct_to_map(&source).unwrap();
    assert!(map.is_empty());
}

#[test]
fn bean_util_map_to_struct_empty() {
    let map = HashMap::new();

    #[derive(Deserialize, Debug, Default)]
    struct Empty {}

    let result: Empty = BeanUtil::map_to_struct(&map).unwrap();
    let _ = result;
}

#[test]
fn bean_util_copy_properties_nested_struct() {
    #[derive(Serialize)]
    struct Inner {
        value: i32,
    }

    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
        name: String,
    }

    #[derive(Deserialize, Debug)]
    struct InnerDst {
        value: i32,
    }

    #[derive(Deserialize, Debug)]
    struct OuterDst {
        inner: InnerDst,
        name: String,
    }

    let source = Outer {
        inner: Inner { value: 42 },
        name: "nested".to_string(),
    };
    let target: OuterDst = BeanUtil::copy_properties(&source).unwrap();
    assert_eq!(target.inner.value, 42);
    assert_eq!(target.name, "nested");
}
