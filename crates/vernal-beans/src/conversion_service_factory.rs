//! ConversionServiceFactory — Spring 风格转换服务工厂。
//!
//! 对应 Java 类：`org.springframework.core.convert.support.DefaultConversionService`
//! 和 `org.springframework.beans.factory.support.ConversionServiceBeanFactory`。
//!
//! 在 Spring 中，`ConversionServiceFactory` 负责创建和配置
//! `ConversionService` 实例，注册默认的类型转换器。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 转换服务工厂。
///
/// 对应 Spring 的 `ConversionServiceFactory`。
///
/// 负责创建 `ConversionService` 实例并注册默认转换器。
/// 支持按源类型和目标类型的组合查找转换器。
pub struct ConversionServiceFactory {
    /// 已注册的转换器映射：(源类型名, 目标类型名) -> 转换函数
    converters: RwLock<
        HashMap<
            (String, String),
            Arc<dyn Fn(&dyn std::any::Any) -> Option<Box<dyn std::any::Any>> + Send + Sync>,
        >,
    >,
    /// 是否已注册默认转换器
    defaults_registered: RwLock<bool>,
}

impl ConversionServiceFactory {
    /// 创建新的转换服务工厂。
    pub fn new() -> Self {
        Self {
            converters: RwLock::new(HashMap::new()),
            defaults_registered: RwLock::new(false),
        }
    }

    /// 注册默认转换器。
    ///
    /// 对应 Spring 的 `DefaultConversionService.addDefaultConverters`。
    /// 注册常见的内置转换器，如 String->i32, String->bool 等。
    pub fn register_defaults(&self) {
        let mut registered = self.defaults_registered.write().unwrap();
        if *registered {
            return;
        }

        let mut converters = self.converters.write().unwrap();

        // String -> i32
        converters.insert(
            ("String".to_string(), "i32".to_string()),
            Arc::new(|val: &dyn std::any::Any| {
                val.downcast_ref::<String>()
                    .and_then(|s| s.parse::<i32>().ok())
                    .map(|v| Box::new(v) as Box<dyn std::any::Any>)
            }),
        );

        // String -> i64
        converters.insert(
            ("String".to_string(), "i64".to_string()),
            Arc::new(|val: &dyn std::any::Any| {
                val.downcast_ref::<String>()
                    .and_then(|s| s.parse::<i64>().ok())
                    .map(|v| Box::new(v) as Box<dyn std::any::Any>)
            }),
        );

        // String -> f64
        converters.insert(
            ("String".to_string(), "f64".to_string()),
            Arc::new(|val: &dyn std::any::Any| {
                val.downcast_ref::<String>()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|v| Box::new(v) as Box<dyn std::any::Any>)
            }),
        );

        // String -> bool
        converters.insert(
            ("String".to_string(), "bool".to_string()),
            Arc::new(|val: &dyn std::any::Any| {
                val.downcast_ref::<String>()
                    .and_then(|s| s.parse::<bool>().ok())
                    .map(|v| Box::new(v) as Box<dyn std::any::Any>)
            }),
        );

        // i32 -> String
        converters.insert(
            ("i32".to_string(), "String".to_string()),
            Arc::new(|val: &dyn std::any::Any| {
                val.downcast_ref::<i32>()
                    .map(|v| Box::new(v.to_string()) as Box<dyn std::any::Any>)
            }),
        );

        // i64 -> String
        converters.insert(
            ("i64".to_string(), "String".to_string()),
            Arc::new(|val: &dyn std::any::Any| {
                val.downcast_ref::<i64>()
                    .map(|v| Box::new(v.to_string()) as Box<dyn std::any::Any>)
            }),
        );

        *registered = true;
    }

    /// 注册自定义转换器。
    pub fn register_converter(
        &self,
        source_type: &str,
        target_type: &str,
        converter: Arc<dyn Fn(&dyn std::any::Any) -> Option<Box<dyn std::any::Any>> + Send + Sync>,
    ) {
        self.converters.write().unwrap().insert(
            (source_type.to_string(), target_type.to_string()),
            converter,
        );
    }

    /// 执行类型转换。
    ///
    /// # 参数
    /// - `value` — 要转换的值
    /// - `source_type` — 源类型名称
    /// - `target_type` — 目标类型名称
    ///
    /// # 返回
    /// - `Ok(Some(converted))` — 转换成功
    /// - `Ok(None)` — 无匹配的转换器
    /// - `Err(msg)` — 转换失败
    pub fn convert(
        &self,
        value: &dyn std::any::Any,
        source_type: &str,
        target_type: &str,
    ) -> Result<Option<Box<dyn std::any::Any>>, String> {
        // 同类型直接返回
        if source_type == target_type {
            return Ok(None);
        }

        let converters = self.converters.read().unwrap();
        if let Some(converter) = converters.get(&(source_type.to_string(), target_type.to_string()))
        {
            Ok(converter(value))
        } else {
            Ok(None)
        }
    }

    /// 是否已注册默认转换器。
    pub fn has_defaults(&self) -> bool {
        *self.defaults_registered.read().unwrap()
    }

    /// 已注册的转换器数量。
    pub fn converter_count(&self) -> usize {
        self.converters.read().unwrap().len()
    }

    /// 是否包含指定的转换器。
    pub fn has_converter(&self, source_type: &str, target_type: &str) -> bool {
        self.converters
            .read()
            .unwrap()
            .contains_key(&(source_type.to_string(), target_type.to_string()))
    }

    /// 清空所有转换器。
    pub fn clear(&self) {
        self.converters.write().unwrap().clear();
        *self.defaults_registered.write().unwrap() = false;
    }
}

impl Default for ConversionServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ConversionServiceFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversionServiceFactory")
            .field("converter_count", &self.converter_count())
            .field("has_defaults", &self.has_defaults())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_defaults_creates_builtin_converters() {
        let factory = ConversionServiceFactory::new();
        assert!(!factory.has_defaults());

        factory.register_defaults();
        assert!(factory.has_defaults());
        assert!(factory.converter_count() > 0);
    }

    #[test]
    fn convert_string_to_i32() {
        let factory = ConversionServiceFactory::new();
        factory.register_defaults();

        let input = "42".to_string();
        let result = factory.convert(&input, "String", "i32").unwrap();
        assert!(result.is_some());
        let val = result.unwrap().downcast::<i32>().unwrap();
        assert_eq!(*val, 42);
    }

    #[test]
    fn convert_string_to_bool() {
        let factory = ConversionServiceFactory::new();
        factory.register_defaults();

        let input = "true".to_string();
        let result = factory.convert(&input, "String", "bool").unwrap();
        assert!(result.is_some());
        let val = result.unwrap().downcast::<bool>().unwrap();
        assert!(*val);
    }

    #[test]
    fn convert_i32_to_string() {
        let factory = ConversionServiceFactory::new();
        factory.register_defaults();

        let input = 42_i32;
        let result = factory.convert(&input, "i32", "String").unwrap();
        assert!(result.is_some());
        let val = result.unwrap().downcast::<String>().unwrap();
        assert_eq!(*val, "42");
    }

    #[test]
    fn same_type_returns_none() {
        let factory = ConversionServiceFactory::new();
        let input = "hello".to_string();
        let result = factory.convert(&input, "String", "String").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn custom_converter() {
        let factory = ConversionServiceFactory::new();
        factory.register_converter(
            "i32",
            "f64",
            Arc::new(|val: &dyn std::any::Any| {
                val.downcast_ref::<i32>()
                    .map(|v| Box::new(*v as f64) as Box<dyn std::any::Any>)
            }),
        );

        let input = 42_i32;
        let result = factory.convert(&input, "i32", "f64").unwrap();
        assert!(result.is_some());
        let val = result.unwrap().downcast::<f64>().unwrap();
        assert_eq!(*val, 42.0);
    }

    #[test]
    fn clear_removes_all() {
        let factory = ConversionServiceFactory::new();
        factory.register_defaults();
        assert!(factory.converter_count() > 0);

        factory.clear();
        assert_eq!(factory.converter_count(), 0);
        assert!(!factory.has_defaults());
    }
}
