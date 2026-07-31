//! Bean 工具类。
//!
//! 对标 hutool-core 的 `BeanUtil`。
//! 提供属性复制、类型检查等实用功能。

use std::any::TypeId;
use std::collections::HashMap;

use super::bean_descriptor::PropertyDescriptor;

/// Bean 工具类。
///
/// 对标 hutool-core 的 `BeanUtil`。
pub struct BeanUtil;

impl BeanUtil {
    /// 将源结构体的属性复制到目标结构体。
    ///
    /// 通过 serde 实现属性复制，对标 hutool-core 的 `BeanUtil::copyProperties`。
    /// 源和目标都必须实现 `serde::Serialize` 和 `serde::Deserialize`。
    ///
    /// # 参数
    /// - `source`：源结构体
    ///
    /// # 返回
    /// 复制后的目标结构体，或复制错误。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use vernal_beans::BeanUtil;
    ///
    /// #[derive(Serialize)]
    /// struct Source { name: String, age: i32 }
    ///
    /// #[derive(Deserialize)]
    /// struct Target { name: String, age: i32 }
    ///
    /// let source = Source { name: "Alice".to_string(), age: 30 };
    /// let target: Target = BeanUtil::copy_properties(&source).unwrap();
    /// assert_eq!(target.name, "Alice");
    /// assert_eq!(target.age, 30);
    /// ```
    pub fn copy_properties<S, T>(source: &S) -> Result<T, BeanError>
    where
        S: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        // 通过 JSON 作为中间表示进行属性复制
        // 这是通用的属性复制方法，适用于任意 serde 兼容的结构体
        let json_value = serde_json::to_value(source).map_err(|e| BeanError {
            message: format!("序列化源对象失败: {}", e),
        })?;

        serde_json::from_value(json_value).map_err(|e| BeanError {
            message: format!("反序列化目标对象失败: {}", e),
        })
    }

    /// 将源 Map 的属性复制到目标结构体。
    ///
    /// 对标 hutool-core 的 `BeanUtil::mapToBean`。
    /// 用于配置属性绑定场景。
    ///
    /// # 参数
    /// - `map`：源属性键值对
    ///
    /// # 返回
    /// 复制后的目标结构体，或复制错误。
    pub fn map_to_struct<T: serde::de::DeserializeOwned>(
        map: &HashMap<String, String>,
    ) -> Result<T, BeanError> {
        let json_value = Self::map_to_json_value(map);
        serde_json::from_value(json_value).map_err(|e| BeanError {
            message: format!("属性转换失败: {}", e),
        })
    }

    /// 将结构体属性复制到 Map。
    ///
    /// 对标 hutool-core 的 `BeanUtil::beanToMap`。
    ///
    /// # 参数
    /// - `source`：源结构体
    ///
    /// # 返回
    /// 属性键值对 Map，或转换错误。
    pub fn struct_to_map<S: serde::Serialize>(
        source: &S,
    ) -> Result<HashMap<String, String>, BeanError> {
        let json_value = serde_json::to_value(source).map_err(|e| BeanError {
            message: format!("序列化源对象失败: {}", e),
        })?;

        let mut map = HashMap::new();
        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                let str_value = match value {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    _ => value.to_string(),
                };
                map.insert(key, str_value);
            }
        }
        Ok(map)
    }

    /// 检查两个类型是否相同。
    #[must_use]
    pub fn type_eq<A: 'static, B: 'static>() -> bool {
        TypeId::of::<A>() == TypeId::of::<B>()
    }

    /// 获取类型的 TypeId。
    #[must_use]
    pub fn type_id_of<T: 'static>() -> TypeId {
        TypeId::of::<T>()
    }

    /// 检查属性是否为可选类型。
    #[must_use]
    pub fn is_optional(property: &PropertyDescriptor) -> bool {
        property.optional
    }

    /// 检查属性是否有默认值。
    #[must_use]
    pub fn has_default(property: &PropertyDescriptor) -> bool {
        property.has_default
    }

    /// 将 HashMap 转换为 serde_json::Value（内部辅助方法）。
    fn map_to_json_value(map: &HashMap<String, String>) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        for (key, value) in map {
            // 尝试解析为数字
            if let Ok(n) = value.parse::<i64>() {
                obj.insert(key.clone(), serde_json::Value::Number(n.into()));
            } else if let Ok(f) = value.parse::<f64>() {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    obj.insert(key.clone(), serde_json::Value::Number(n));
                }
            } else if value == "true" {
                obj.insert(key.clone(), serde_json::Value::Bool(true));
            } else if value == "false" {
                obj.insert(key.clone(), serde_json::Value::Bool(false));
            } else {
                obj.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }
        serde_json::Value::Object(obj)
    }
}

/// Bean 操作错误。
#[derive(Debug, Clone)]
pub struct BeanError {
    /// 错误消息
    pub message: String,
}

impl std::fmt::Display for BeanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bean 错误: {}", self.message)
    }
}

impl std::error::Error for BeanError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestSource {
        name: String,
        age: i32,
        active: bool,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestTarget {
        name: String,
        age: i32,
        active: bool,
    }

    #[derive(Deserialize, Debug)]
    struct PartialTarget {
        name: String,
    }

    #[test]
    fn copy_properties_success() {
        let source = TestSource {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        };
        let target: TestTarget = BeanUtil::copy_properties(&source).unwrap();
        assert_eq!(target.name, "Alice");
        assert_eq!(target.age, 30);
        assert!(target.active);
    }

    #[test]
    fn copy_properties_partial() {
        let source = TestSource {
            name: "Bob".to_string(),
            age: 25,
            active: false,
        };
        let target: PartialTarget = BeanUtil::copy_properties(&source).unwrap();
        assert_eq!(target.name, "Bob");
    }

    #[test]
    fn map_to_struct_success() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "Charlie".to_string());
        map.insert("age".to_string(), "35".to_string());
        map.insert("active".to_string(), "true".to_string());
        let target: TestTarget = BeanUtil::map_to_struct(&map).unwrap();
        assert_eq!(target.name, "Charlie");
        assert_eq!(target.age, 35);
        assert!(target.active);
    }

    #[test]
    fn map_to_struct_missing_field() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "Dave".to_string());
        // Missing "age" and "active" - should fail
        let result: Result<TestTarget, _> = BeanUtil::map_to_struct(&map);
        assert!(result.is_err());
    }

    #[test]
    fn struct_to_map_success() {
        let source = TestSource {
            name: "Eve".to_string(),
            age: 28,
            active: true,
        };
        let map = BeanUtil::struct_to_map(&source).unwrap();
        assert_eq!(map.get("name").unwrap(), "Eve");
        assert_eq!(map.get("age").unwrap(), "28");
        assert_eq!(map.get("active").unwrap(), "true");
    }

    #[test]
    fn struct_to_map_with_null_field() {
        #[derive(Serialize)]
        struct WithOption {
            name: String,
            value: Option<String>,
        }
        let source = WithOption {
            name: "test".to_string(),
            value: None,
        };
        let map = BeanUtil::struct_to_map(&source).unwrap();
        assert_eq!(map.get("name").unwrap(), "test");
        assert_eq!(map.get("value").unwrap(), "");
    }

    #[test]
    fn type_eq_same_type() {
        assert!(BeanUtil::type_eq::<String, String>());
        assert!(BeanUtil::type_eq::<i32, i32>());
    }

    #[test]
    fn type_eq_different_types() {
        assert!(!BeanUtil::type_eq::<String, i32>());
        assert!(!BeanUtil::type_eq::<i32, i64>());
    }

    #[test]
    fn type_id_of_returns_correct_id() {
        assert_eq!(BeanUtil::type_id_of::<String>(), TypeId::of::<String>());
        assert_eq!(BeanUtil::type_id_of::<i32>(), TypeId::of::<i32>());
    }

    #[test]
    fn is_optional_true() {
        let prop = crate::bean_descriptor::PropertyDescriptor::new(
            "field",
            TypeId::of::<String>(),
            "String",
            true,
            false,
        );
        assert!(BeanUtil::is_optional(&prop));
    }

    #[test]
    fn is_optional_false() {
        let prop = crate::bean_descriptor::PropertyDescriptor::new(
            "field",
            TypeId::of::<String>(),
            "String",
            false,
            false,
        );
        assert!(!BeanUtil::is_optional(&prop));
    }

    #[test]
    fn has_default_true() {
        let prop = crate::bean_descriptor::PropertyDescriptor::new(
            "field",
            TypeId::of::<String>(),
            "String",
            false,
            true,
        );
        assert!(BeanUtil::has_default(&prop));
    }

    #[test]
    fn has_default_false() {
        let prop = crate::bean_descriptor::PropertyDescriptor::new(
            "field",
            TypeId::of::<String>(),
            "String",
            false,
            false,
        );
        assert!(!BeanUtil::has_default(&prop));
    }

    #[test]
    fn map_to_json_value_numbers() {
        let mut map = HashMap::new();
        map.insert("int_val".to_string(), "42".to_string());
        map.insert("float_val".to_string(), "3.14".to_string());
        map.insert("bool_true".to_string(), "true".to_string());
        map.insert("bool_false".to_string(), "false".to_string());
        map.insert("string_val".to_string(), "hello".to_string());
        let json = BeanUtil::map_to_json_value(&map);
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("int_val").unwrap().as_i64().unwrap(), 42);
        assert_eq!(obj.get("bool_true").unwrap().as_bool().unwrap(), true);
        assert_eq!(obj.get("bool_false").unwrap().as_bool().unwrap(), false);
        assert_eq!(obj.get("string_val").unwrap().as_str().unwrap(), "hello");
    }

    #[test]
    fn bean_error_display() {
        let err = BeanError {
            message: "test error".to_string(),
        };
        assert!(format!("{}", err).contains("test error"));
    }

    #[test]
    fn bean_error_is_clone() {
        let err = BeanError {
            message: "clone me".to_string(),
        };
        let err2 = err.clone();
        assert_eq!(err.message, err2.message);
    }

    #[test]
    fn bean_error_is_debug() {
        let err = BeanError {
            message: "debug test".to_string(),
        };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("BeanError"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn bean_error_is_std_error() {
        let err = BeanError {
            message: "error test".to_string(),
        };
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn copy_properties_with_nested_struct() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Nested {
            value: i32,
        }
        #[derive(Serialize, Debug, PartialEq)]
        struct SourceWithNested {
            name: String,
            nested: Nested,
        }
        #[derive(Deserialize, Debug, PartialEq)]
        struct TargetWithNested {
            name: String,
            nested: Nested,
        }

        let source = SourceWithNested {
            name: "test".to_string(),
            nested: Nested { value: 42 },
        };
        let target: TargetWithNested = BeanUtil::copy_properties(&source).unwrap();
        assert_eq!(target.name, "test");
        assert_eq!(target.nested.value, 42);
    }

    #[test]
    fn struct_to_map_with_array() {
        #[derive(Serialize)]
        struct WithArray {
            name: String,
            items: Vec<String>,
        }
        let source = WithArray {
            name: "test".to_string(),
            items: vec!["a".to_string(), "b".to_string()],
        };
        let map = BeanUtil::struct_to_map(&source).unwrap();
        assert_eq!(map.get("name").unwrap(), "test");
        // Arrays are serialized as JSON strings
        let items_str = map.get("items").unwrap();
        assert!(items_str.contains("a"));
        assert!(items_str.contains("b"));
    }

    #[test]
    fn map_to_json_value_float() {
        let mut map = HashMap::new();
        map.insert("price".to_string(), "19.99".to_string());
        let json = BeanUtil::map_to_json_value(&map);
        let obj = json.as_object().unwrap();
        let price = obj.get("price").unwrap().as_f64().unwrap();
        assert!((price - 19.99).abs() < f64::EPSILON);
    }

    #[test]
    fn map_to_json_value_string() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "hello world".to_string());
        let json = BeanUtil::map_to_json_value(&map);
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "hello world");
    }

    #[test]
    fn type_eq_reflexive() {
        assert!(BeanUtil::type_eq::<i32, i32>());
        assert!(BeanUtil::type_eq::<String, String>());
        assert!(BeanUtil::type_eq::<bool, bool>());
        assert!(BeanUtil::type_eq::<f64, f64>());
    }

    #[test]
    fn type_id_of_various_types() {
        assert_ne!(BeanUtil::type_id_of::<i32>(), BeanUtil::type_id_of::<i64>());
        assert_ne!(BeanUtil::type_id_of::<String>(), BeanUtil::type_id_of::<i32>());
    }

    #[test]
    fn is_optional_and_has_default_both_true() {
        let prop = crate::bean_descriptor::PropertyDescriptor::new(
            "field",
            TypeId::of::<String>(),
            "String",
            true,
            true,
        );
        assert!(BeanUtil::is_optional(&prop));
        assert!(BeanUtil::has_default(&prop));
    }

    #[test]
    fn copy_properties_empty_struct() {
        #[derive(Serialize)]
        struct Empty {}
        #[derive(Deserialize, Debug)]
        struct EmptyTarget {}
        let source = Empty {};
        let _: EmptyTarget = BeanUtil::copy_properties(&source).unwrap();
    }

    #[test]
    fn struct_to_map_with_bool_false() {
        #[derive(Serialize)]
        struct WithBool {
            flag: bool,
        }
        let source = WithBool { flag: false };
        let map = BeanUtil::struct_to_map(&source).unwrap();
        assert_eq!(map.get("flag").unwrap(), "false");
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn map_to_json_value_integer_and_float() {
        let mut map = HashMap::new();
        map.insert("count".to_string(), "100".to_string());
        map.insert("price".to_string(), "9.99".to_string());
        let json = BeanUtil::map_to_json_value(&map);
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("count").unwrap().as_i64().unwrap(), 100);
        let price = obj.get("price").unwrap().as_f64().unwrap();
        assert!((price - 9.99).abs() < f64::EPSILON);
    }

    #[test]
    fn map_to_json_value_boolean_values() {
        let mut map = HashMap::new();
        map.insert("yes".to_string(), "true".to_string());
        map.insert("no".to_string(), "false".to_string());
        let json = BeanUtil::map_to_json_value(&map);
        let obj = json.as_object().unwrap();
        assert!(obj.get("yes").unwrap().as_bool().unwrap());
        assert!(!obj.get("no").unwrap().as_bool().unwrap());
    }

    #[test]
    fn map_to_json_value_string_value() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "hello world".to_string());
        let json = BeanUtil::map_to_json_value(&map);
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("name").unwrap().as_str().unwrap(), "hello world");
    }

    #[test]
    fn map_to_struct_with_float() {
        #[derive(Deserialize, Debug)]
        struct WithFloat {
            price: f64,
        }
        let mut map = HashMap::new();
        map.insert("price".to_string(), "19.99".to_string());
        let target: WithFloat = BeanUtil::map_to_struct(&map).unwrap();
        assert!((target.price - 19.99).abs() < f64::EPSILON);
    }

    #[test]
    fn copy_properties_with_option_none() {
        #[derive(Serialize)]
        struct Source {
            name: String,
            value: Option<i32>,
        }
        #[derive(Deserialize, Debug)]
        struct Target {
            name: String,
            value: Option<i32>,
        }
        let source = Source {
            name: "test".to_string(),
            value: None,
        };
        let target: Target = BeanUtil::copy_properties(&source).unwrap();
        assert_eq!(target.name, "test");
        assert!(target.value.is_none());
    }

    #[test]
    fn struct_to_map_empty_struct() {
        #[derive(Serialize)]
        struct Empty {}
        let source = Empty {};
        let map = BeanUtil::struct_to_map(&source).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn type_eq_different_sizes() {
        assert!(!BeanUtil::type_eq::<u8, u64>());
        assert!(!BeanUtil::type_eq::<f32, f64>());
    }

    #[test]
    fn bean_error_display_format() {
        let err = BeanError {
            message: "test".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Bean 错误"));
        assert!(display.contains("test"));
    }
}
