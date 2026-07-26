//! Bean 工具类。
//!
//! 对标 hutool-core 的 `BeanUtil`。
//! 提供属性复制、类型检查等实用功能。

use std::any::TypeId;
use std::collections::HashMap;

use super::bean_descriptor::{BeanDescriptor, PropertyDescriptor};

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
