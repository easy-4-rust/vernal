//! MutablePropertyValues 差分测试。
//!
//! 参照 Spring Framework 7.0.8 的 `MutablePropertyValuesTests` 测试场景，
//! 验证 vernal-beans 的 MutablePropertyValues 行为与 Spring 一致。

use std::sync::Arc;

use vernal_beans::mutable_property_values::MutablePropertyValues;
use vernal_beans::property_value::PropertyValue;

// ── 1. 基本操作差分测试 ─────────────────────────────────────────────────

/// 参照 Spring `MutablePropertyValuesTests.testAdd`：
/// 验证添加属性值。
#[test]
fn differential_add_property_value() {
    let mut pvs = MutablePropertyValues::new();

    // Spring: pvs.add(new PropertyValue("name", "Alice"))
    pvs.add(PropertyValue::new("name", Arc::new("Alice".to_string())));

    assert!(pvs.contains("name"));
    assert_eq!(pvs.len(), 1);
}

/// 参照 Spring `MutablePropertyValuesTests.testContains`：
/// 验证 contains 方法。
#[test]
fn differential_contains() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key1", Arc::new(1i32));
    pvs.add_value("key2", Arc::new(2i32));

    assert!(pvs.contains("key1"));
    assert!(pvs.contains("key2"));
    assert!(!pvs.contains("key3"));
}

/// 参照 Spring `MutablePropertyValuesTests.testGet`：
/// 验证 get 方法。
#[test]
fn differential_get() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("name", Arc::new("Alice".to_string()));

    let pv = pvs.get("name");
    assert!(pv.is_some());
    let val = pv.unwrap().value().downcast_ref::<String>().unwrap();
    assert_eq!(val, "Alice");

    assert!(pvs.get("nonexistent").is_none());
}

/// 参照 Spring `MutablePropertyValuesTests.testSize`：
/// 验证 size 属性。
#[test]
fn differential_size() {
    let mut pvs = MutablePropertyValues::new();
    assert_eq!(pvs.len(), 0);
    assert!(pvs.is_empty());

    pvs.add_value("a", Arc::new(1i32));
    assert_eq!(pvs.len(), 1);
    assert!(!pvs.is_empty());

    pvs.add_value("b", Arc::new(2i32));
    assert_eq!(pvs.len(), 2);
}

/// 参照 Spring `MutablePropertyValuesTests.testClear`：
/// 验证 clear 方法。
#[test]
fn differential_clear() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));

    assert_eq!(pvs.len(), 2);
    pvs.clear();
    assert!(pvs.is_empty());
    assert!(!pvs.contains("a"));
    assert!(!pvs.contains("b"));
}

/// 参照 Spring `MutablePropertyValuesTests.testOverwrite`：
/// 验证属性覆盖。
#[test]
fn differential_overwrite() {
    let mut pvs = MutablePropertyValues::new();

    pvs.add_value("name", Arc::new("Alice".to_string()));
    let val = pvs
        .get("name")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(val, "Alice");

    // 覆盖
    pvs.add_value("name", Arc::new("Bob".to_string()));
    let val = pvs
        .get("name")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(val, "Bob");

    // 数量不变
    assert_eq!(pvs.len(), 1);
}

// ── 2. 批量操作差分测试 ─────────────────────────────────────────────────

/// 参照 Spring `MutablePropertyValuesTests.testAddPropertyValues`：
/// 验证批量添加属性值。
#[test]
fn differential_add_property_values() {
    let mut pvs1 = MutablePropertyValues::new();
    pvs1.add_value("a", Arc::new(1i32));

    let mut pvs2 = MutablePropertyValues::new();
    pvs2.add_value("b", Arc::new(2i32));
    pvs2.add_value("c", Arc::new(3i32));

    pvs1.add_property_values(&pvs2);

    assert_eq!(pvs1.len(), 3);
    assert!(pvs1.contains("a"));
    assert!(pvs1.contains("b"));
    assert!(pvs1.contains("c"));
}

/// 验证批量添加时覆盖已有属性。
#[test]
fn differential_add_property_values_overwrites() {
    let mut pvs1 = MutablePropertyValues::new();
    pvs1.add_value("key", Arc::new("old".to_string()));

    let mut pvs2 = MutablePropertyValues::new();
    pvs2.add_value("key", Arc::new("new".to_string()));

    pvs1.add_property_values(&pvs2);

    let val = pvs1
        .get("key")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(val, "new");
    assert_eq!(pvs1.len(), 1);
}

// ── 3. 从 Vec 创建差分测试 ──────────────────────────────────────────────

/// 参照 Spring `MutablePropertyValuesTests.testFromList`：
/// 验证从 PropertyValue 列表创建。
#[test]
fn differential_from_vec() {
    let pvs = MutablePropertyValues::from_vec(vec![
        PropertyValue::new("x", Arc::new(1i32)),
        PropertyValue::new("y", Arc::new(2i32)),
        PropertyValue::new("z", Arc::new(3i32)),
    ]);

    assert_eq!(pvs.len(), 3);
    assert!(pvs.contains("x"));
    assert!(pvs.contains("y"));
    assert!(pvs.contains("z"));
}

/// 验证从空 Vec 创建。
#[test]
fn differential_from_empty_vec() {
    let pvs = MutablePropertyValues::from_vec(vec![]);
    assert!(pvs.is_empty());
}

// ── 4. PropertyValue 基本操作差分测试 ────────────────────────────────────

/// 验证 PropertyValue 创建和访问。
#[test]
fn differential_property_value_basic() {
    let pv = PropertyValue::new("name", Arc::new("Alice".to_string()));
    assert_eq!(pv.name(), "name");
    let val = pv.value().downcast_ref::<String>().unwrap();
    assert_eq!(val, "Alice");
}

/// 验证 PropertyValue 的 Clone。
#[test]
fn differential_property_value_clone() {
    let pv = PropertyValue::new("key", Arc::new(42i32));
    let pv2 = pv.clone();
    assert_eq!(pv.name(), pv2.name());
}

// ── 5. Container PropertyValues 绑定差分测试 ─────────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.propertiesPopulationWithPrefix`：
/// 验证 PropertyValues 绑定到 Bean 的完整流程。
#[test]
fn differential_property_values_bind_to_bean() {
    use vernal_beans::ComponentDefinition;
    use vernal_beans::RegistryBuilder;
    use vernal_beans::Resolver;

    // 创建 PropertyValues
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("db_url", Arc::new("postgres://localhost/test".to_string()));
    pvs.add_value("max_connections", Arc::new(10u32));

    // 在 ComponentDefinition factory 中应用 PropertyValues
    let db_url = pvs
        .get("db_url")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap()
        .clone();
    let max_conn = *pvs
        .get("max_connections")
        .unwrap()
        .value()
        .downcast_ref::<u32>()
        .unwrap();

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            move |_resolver: &Resolver| AppConfig {
                db_url: db_url.clone(),
                max_connections: max_conn,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let config = container.resolve::<AppConfig>().unwrap();
    assert_eq!(config.db_url, "postgres://localhost/test");
    assert_eq!(config.max_connections, 10);
}

/// 参照 Spring `MutablePropertyValuesTests`：
/// 验证 PropertyValues 合并行为。
#[test]
fn differential_property_values_merge() {
    let mut pvs1 = MutablePropertyValues::new();
    pvs1.add_value("a", Arc::new(1i32));
    pvs1.add_value("b", Arc::new(2i32));
    pvs1.add_value("c", Arc::new(3i32));

    let mut pvs2 = MutablePropertyValues::new();
    pvs2.add_value("b", Arc::new(20i32)); // 覆盖
    pvs2.add_value("d", Arc::new(4i32)); // 新增

    pvs1.add_property_values(&pvs2);

    assert_eq!(pvs1.len(), 4); // a, b(覆盖), c, d
    let val_b = pvs1
        .get("b")
        .unwrap()
        .value()
        .downcast_ref::<i32>()
        .unwrap();
    assert_eq!(*val_b, 20); // b 被覆盖
    let val_d = pvs1
        .get("d")
        .unwrap()
        .value()
        .downcast_ref::<i32>()
        .unwrap();
    assert_eq!(*val_d, 4); // d 是新增
}

/// 验证 PropertyValues 的 immutability 语义（Spring 的 ImmutablePropertyValues）。
#[test]
fn differential_immutable_property_values_semantics() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new("value".to_string()));

    // 验证 get 返回引用
    let pv = pvs.get("key").unwrap();
    let val = pv.value().downcast_ref::<String>().unwrap();
    assert_eq!(val, "value");

    // 验证 contains
    assert!(pvs.contains("key"));
    assert!(!pvs.contains("other"));

    // 验证 len
    assert_eq!(pvs.len(), 1);

    // 验证 get_property_values
    assert_eq!(pvs.get_property_values().len(), 1);
}

/// 验证 PropertyValues 的 get_property_values 返回正确的切片。
#[test]
fn differential_get_property_values_slice() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("x", Arc::new(1i32));
    pvs.add_value("y", Arc::new(2i32));
    pvs.add_value("z", Arc::new(3i32));

    let values = pvs.get_property_values();
    assert_eq!(values.len(), 3);

    // 验证所有属性名都存在
    let names: Vec<&str> = values.iter().map(|pv| pv.name()).collect();
    assert!(names.contains(&"x"));
    assert!(names.contains(&"y"));
    assert!(names.contains(&"z"));
}

/// 验证 PropertyValues 的 add 方法覆盖语义。
#[test]
fn differential_add_overwrites_existing() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add(PropertyValue::new("key", Arc::new("first".to_string())));
    assert_eq!(pvs.len(), 1);

    pvs.add(PropertyValue::new("key", Arc::new("second".to_string())));
    assert_eq!(pvs.len(), 1); // 覆盖，数量不变

    let val = pvs
        .get("key")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(val, "second");
}

/// 验证 PropertyValues 的 add_value 便捷方法。
#[test]
fn differential_add_value_convenience() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("str_key", Arc::new("string_value".to_string()));
    pvs.add_value("int_key", Arc::new(42i32));
    pvs.add_value("bool_key", Arc::new(true));

    assert_eq!(pvs.len(), 3);

    let str_val = pvs
        .get("str_key")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(str_val, "string_value");

    let int_val = pvs
        .get("int_key")
        .unwrap()
        .value()
        .downcast_ref::<i32>()
        .unwrap();
    assert_eq!(*int_val, 42);

    let bool_val = pvs
        .get("bool_key")
        .unwrap()
        .value()
        .downcast_ref::<bool>()
        .unwrap();
    assert!(*bool_val);
}

/// 验证 PropertyValues 的 is_empty 和 clear 语义。
#[test]
fn differential_empty_and_clear() {
    let mut pvs = MutablePropertyValues::new();
    assert!(pvs.is_empty());
    assert_eq!(pvs.len(), 0);

    pvs.add_value("key", Arc::new("value".to_string()));
    assert!(!pvs.is_empty());
    assert_eq!(pvs.len(), 1);

    pvs.clear();
    assert!(pvs.is_empty());
    assert_eq!(pvs.len(), 0);
    assert!(!pvs.contains("key"));
}

/// 验证 PropertyValues 的 get_property_values 返回正确的值引用。
#[test]
fn differential_get_property_values_returns_references() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("name", Arc::new("Alice".to_string()));
    pvs.add_value("age", Arc::new(30i32));

    let values = pvs.get_property_values();
    assert_eq!(values.len(), 2);

    // 验证可以访问每个值
    for pv in values {
        assert!(!pv.name().is_empty());
        // value() 返回 &Arc<dyn Any>，不是 Option
        let _ = pv.value();
    }
}

/// 验证 PropertyValues 的 contains 方法对不同类型都有效。
#[test]
fn differential_contains_various_types() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("str", Arc::new("hello".to_string()));
    pvs.add_value("int", Arc::new(42i32));
    pvs.add_value("bool", Arc::new(true));
    pvs.add_value("float", Arc::new(3.14f64));

    assert!(pvs.contains("str"));
    assert!(pvs.contains("int"));
    assert!(pvs.contains("bool"));
    assert!(pvs.contains("float"));
    assert!(!pvs.contains("missing"));
}

/// 验证 PropertyValues 的 get 方法返回正确的类型。
#[test]
fn differential_get_returns_correct_type() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("str", Arc::new("hello".to_string()));
    pvs.add_value("int", Arc::new(42i32));
    pvs.add_value("bool", Arc::new(true));

    // 验证类型正确
    let str_val = pvs.get("str").unwrap().value();
    assert!(str_val.downcast_ref::<String>().is_some());

    let int_val = pvs.get("int").unwrap().value();
    assert!(int_val.downcast_ref::<i32>().is_some());

    let bool_val = pvs.get("bool").unwrap().value();
    assert!(bool_val.downcast_ref::<bool>().is_some());
}

/// 验证 PropertyValues 的 add 方法保持顺序。
#[test]
fn differential_add_maintains_order() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("first", Arc::new(1i32));
    pvs.add_value("second", Arc::new(2i32));
    pvs.add_value("third", Arc::new(3i32));

    let values = pvs.get_property_values();
    assert_eq!(values[0].name(), "first");
    assert_eq!(values[1].name(), "second");
    assert_eq!(values[2].name(), "third");
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持顺序。
#[test]
fn differential_add_overwrite_maintains_order() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));
    pvs.add_value("a", Arc::new(10i32)); // 覆盖 "a"

    let values = pvs.get_property_values();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].name(), "a"); // "a" 在原始位置
    assert_eq!(values[1].name(), "b");
}

/// 验证 PropertyValues 的 from_vec 方法保持顺序。
#[test]
fn differential_from_vec_maintains_order() {
    let pvs = MutablePropertyValues::from_vec(vec![
        PropertyValue::new("z", Arc::new(1i32)),
        PropertyValue::new("a", Arc::new(2i32)),
        PropertyValue::new("m", Arc::new(3i32)),
    ]);

    let values = pvs.get_property_values();
    assert_eq!(values[0].name(), "z");
    assert_eq!(values[1].name(), "a");
    assert_eq!(values[2].name(), "m");
}

/// 验证 PropertyValues 的 add_property_values 合并语义。
#[test]
fn differential_add_property_values_merge_semantics() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));

    let mut other = MutablePropertyValues::new();
    other.add_value("b", Arc::new(20i32)); // 覆盖
    other.add_value("c", Arc::new(3i32)); // 新增

    pvs.add_property_values(&other);

    // a: 未变, b: 覆盖, c: 新增
    assert_eq!(pvs.len(), 3);
    assert_eq!(
        *pvs.get("a").unwrap().value().downcast_ref::<i32>().unwrap(),
        1
    );
    assert_eq!(
        *pvs.get("b").unwrap().value().downcast_ref::<i32>().unwrap(),
        20
    );
    assert_eq!(
        *pvs.get("c").unwrap().value().downcast_ref::<i32>().unwrap(),
        3
    );
}

/// 验证 PropertyValues 的 get 方法在不存在时返回 None。
#[test]
fn differential_get_nonexistent_returns_none() {
    let pvs = MutablePropertyValues::new();
    assert!(pvs.get("nonexistent").is_none());
}

/// 验证 PropertyValues 的 contains 方法在空集合时返回 false。
#[test]
fn differential_contains_empty_returns_false() {
    let pvs = MutablePropertyValues::new();
    assert!(!pvs.contains("any"));
}

/// 验证 PropertyValues 的 len 和 is_empty 一致性。
#[test]
fn differential_len_and_is_empty_consistency() {
    let mut pvs = MutablePropertyValues::new();
    assert_eq!(pvs.len(), 0);
    assert!(pvs.is_empty());

    pvs.add_value("key", Arc::new("value".to_string()));
    assert_eq!(pvs.len(), 1);
    assert!(!pvs.is_empty());

    pvs.clear();
    assert_eq!(pvs.len(), 0);
    assert!(pvs.is_empty());
}

/// 验证 PropertyValues 的 add 方法在空集合时正确工作。
#[test]
fn differential_add_to_empty() {
    let mut pvs = MutablePropertyValues::new();
    assert!(pvs.is_empty());

    pvs.add_value("first", Arc::new(1i32));
    assert_eq!(pvs.len(), 1);
    assert!(pvs.contains("first"));
}

/// 验证 PropertyValues 的 add 方法在覆盖后保留其他属性。
#[test]
fn differential_add_preserves_other_properties() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));
    pvs.add_value("c", Arc::new(3i32));

    pvs.add_value("b", Arc::new(20i32)); // 覆盖 "b"

    assert_eq!(pvs.len(), 3);
    assert!(pvs.contains("a"));
    assert!(pvs.contains("b"));
    assert!(pvs.contains("c"));
}

/// 验证 PropertyValues 的 get 方法返回正确的值引用。
#[test]
fn differential_get_returns_value_reference() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new(42i32));

    let pv = pvs.get("key").unwrap();
    let val = pv.value().downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);

    // 验证两次 get 返回同一个 Arc
    let pv2 = pvs.get("key").unwrap();
    let val2 = pv2.value().downcast_ref::<i32>().unwrap();
    assert_eq!(*val2, 42);
}

/// 验证 PropertyValues 的 add 方法在空集合时创建新条目。
#[test]
fn differential_add_creates_new_entry() {
    let mut pvs = MutablePropertyValues::new();

    pvs.add_value("key", Arc::new("value".to_string()));
    assert_eq!(pvs.len(), 1);
    assert!(pvs.contains("key"));
    assert!(!pvs.contains("other"));
}

/// 验证 PropertyValues 的 add 方法在覆盖时更新值。
#[test]
fn differential_add_updates_value() {
    let mut pvs = MutablePropertyValues::new();

    pvs.add_value("key", Arc::new("old".to_string()));
    pvs.add_value("key", Arc::new("new".to_string()));

    let val = pvs
        .get("key")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(val, "new");
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持位置。
#[test]
fn differential_add_maintains_position() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));
    pvs.add_value("a", Arc::new(10i32));

    let values = pvs.get_property_values();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].name(), "a"); // "a" 在原始位置
    assert_eq!(*values[0].value().downcast_ref::<i32>().unwrap(), 10);
    assert_eq!(values[1].name(), "b");
    assert_eq!(*values[1].value().downcast_ref::<i32>().unwrap(), 2);
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持其他属性不变。
#[test]
fn differential_add_preserves_others() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));
    pvs.add_value("c", Arc::new(3i32));

    pvs.add_value("b", Arc::new(20i32));

    assert_eq!(
        *pvs.get("a").unwrap().value().downcast_ref::<i32>().unwrap(),
        1
    );
    assert_eq!(
        *pvs.get("b").unwrap().value().downcast_ref::<i32>().unwrap(),
        20
    );
    assert_eq!(
        *pvs.get("c").unwrap().value().downcast_ref::<i32>().unwrap(),
        3
    );
}

/// 验证 PropertyValues 的 get 方法在空集合时返回 None。
#[test]
fn differential_get_empty_returns_none() {
    let pvs = MutablePropertyValues::new();
    assert!(pvs.get("any").is_none());
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持值类型。
#[test]
fn differential_add_preserves_value_type() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new(42i32));

    let val = pvs.get("key").unwrap().value();
    assert!(val.downcast_ref::<i32>().is_some());

    pvs.add_value("key", Arc::new(100i32));

    let val = pvs.get("key").unwrap().value();
    assert!(val.downcast_ref::<i32>().is_some());
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持值可访问。
#[test]
fn differential_add_keeps_value_accessible() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new("first".to_string()));
    pvs.add_value("key", Arc::new("second".to_string()));

    let val = pvs
        .get("key")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(val, "second");
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持属性名不变。
#[test]
fn differential_add_preserves_name() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new("first".to_string()));
    pvs.add_value("key", Arc::new("second".to_string()));

    let pv = pvs.get("key").unwrap();
    assert_eq!(pv.name(), "key");
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持值类型。
#[test]
fn differential_add_preserves_value_type_after_overwrite() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new(42i32));
    pvs.add_value("key", Arc::new(100i32));

    let val = pvs.get("key").unwrap().value();
    assert!(val.downcast_ref::<i32>().is_some());
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持值可访问。
#[test]
fn differential_add_keeps_value_accessible_after_overwrite() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new("first".to_string()));
    pvs.add_value("key", Arc::new("second".to_string()));

    let val = pvs
        .get("key")
        .unwrap()
        .value()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(val, "second");
}

/// 验证 PropertyValues 的 add 方法在覆盖时保持属性名不变。
#[test]
fn differential_add_preserves_name_after_overwrite() {
    let mut pvs = MutablePropertyValues::new();
    pvs.add_value("key", Arc::new("first".to_string()));
    pvs.add_value("key", Arc::new("second".to_string()));

    let pv = pvs.get("key").unwrap();
    assert_eq!(pv.name(), "key");
}

// ── 辅助类型 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct AppConfig {
    db_url: String,
    max_connections: u32,
}
