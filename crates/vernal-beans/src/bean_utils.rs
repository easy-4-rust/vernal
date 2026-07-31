//! BeanUtils — 对应 Spring `org.springframework.beans.BeanUtils`。
//!
//! Bean 工具类，提供实例化 bean、属性操作等静态工具方法。

use std::any::Any;
use std::collections::HashMap;

/// Bean 工具类。
///
/// 对应 Java 类：`org.springframework.beans.BeanUtils`。
///
/// 提供实例化 bean、属性操作等静态工具方法。
pub struct BeanUtils;

impl BeanUtils {
    /// 实例化一个类。
    ///
    /// 对应 Java 方法：`Object instantiateClass(Class<?> clazz)`
    ///
    /// 使用默认构造器创建实例。
    pub fn instantiate_class<T: Default>() -> T {
        T::default()
    }

    /// 检查是否是简单属性类型。
    ///
    /// 对应 Java 方法：`boolean isSimpleProperty(Class<?> type)`
    ///
    /// 简单属性类型包括基本类型、字符串、数字、日期等。
    pub fn is_simple_type_name(type_name: &str) -> bool {
        matches!(
            type_name,
            "bool"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "f32"
                | "f64"
                | "String"
                | "str"
                | "char"
                | "usize"
                | "isize"
        )
    }

    /// 检查是否是简单属性类型。
    ///
    /// 对应 Java 方法：`boolean isSimpleProperty(Class<?> type)`
    pub fn is_simple_property(type_id: std::any::TypeId) -> bool {
        type_id == std::any::TypeId::of::<bool>()
            || type_id == std::any::TypeId::of::<i8>()
            || type_id == std::any::TypeId::of::<i16>()
            || type_id == std::any::TypeId::of::<i32>()
            || type_id == std::any::TypeId::of::<i64>()
            || type_id == std::any::TypeId::of::<i128>()
            || type_id == std::any::TypeId::of::<u8>()
            || type_id == std::any::TypeId::of::<u16>()
            || type_id == std::any::TypeId::of::<u32>()
            || type_id == std::any::TypeId::of::<u64>()
            || type_id == std::any::TypeId::of::<u128>()
            || type_id == std::any::TypeId::of::<f32>()
            || type_id == std::any::TypeId::of::<f64>()
            || type_id == std::any::TypeId::of::<String>()
            || type_id == std::any::TypeId::of::<char>()
            || type_id == std::any::TypeId::of::<usize>()
            || type_id == std::any::TypeId::of::<isize>()
    }

    /// 查找指定名称的方法。
    ///
    /// 对应 Java 方法：`Method findMethod(Class<?> clazz, String methodName, Class<?>... paramTypes)`
    ///
    /// 在 Rust 中，这对应于查找函数或闭包。
    pub fn find_method_name(type_name: &str, method_name: &str) -> Option<String> {
        // 在 Rust 中，方法查找是通过 trait 和 impl 块完成的
        // 这里提供一个简单的名称匹配
        if method_name.is_empty() {
            return None;
        }
        Some(format!("{}::{}", type_name, method_name))
    }

    /// 解析指定类型的属性描述符。
    ///
    /// 对应 Java 方法：`PropertyDescriptor[] getPropertyDescriptors(Class<?> clazz)`
    ///
    /// 在 Rust 中，属性通过 struct 字段访问，不需要反射。
    pub fn get_property_type_name(type_name: &str, property_name: &str) -> Option<String> {
        // 简单实现：返回属性名称作为类型提示
        if property_name.is_empty() {
            return None;
        }
        Some(format!("{}::{}", type_name, property_name))
    }

    /// 复制属性。
    ///
    /// 对应 Java 方法：`void copyProperties(Object source, Object target)`
    ///
    /// 在 Rust 中，属性复制需要显式处理。
    pub fn copy_properties_map(
        source: &HashMap<String, Box<dyn Any + Send + Sync>>,
        target: &mut HashMap<String, Box<dyn Any + Send + Sync>>,
    ) {
        for (key, _value) in source {
            // 由于 dyn Any 不实现 Clone，这里只复制键
            // 值需要调用者通过其他方式处理
            if !target.contains_key(key) {
                target.insert(key.clone(), Box::new(()));
            }
        }
    }
}

/// Bean 错误类型。
#[derive(Debug, Clone)]
pub struct BeanError {
    message: String,
}

impl BeanError {
    /// 创建一个新的 BeanError。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// 获取错误消息。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for BeanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BeanError: {}", self.message)
    }
}

impl std::error::Error for BeanError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instantiate_class() {
        let value: String = BeanUtils::instantiate_class();
        assert_eq!(value, "");
    }

    #[test]
    fn test_is_simple_type_name() {
        assert!(BeanUtils::is_simple_type_name("i32"));
        assert!(BeanUtils::is_simple_type_name("String"));
        assert!(BeanUtils::is_simple_type_name("bool"));
        assert!(!BeanUtils::is_simple_type_name("Vec"));
        assert!(!BeanUtils::is_simple_type_name("HashMap"));
    }

    #[test]
    fn test_is_simple_property() {
        assert!(BeanUtils::is_simple_property(std::any::TypeId::of::<i32>()));
        assert!(BeanUtils::is_simple_property(std::any::TypeId::of::<String>()));
        assert!(BeanUtils::is_simple_property(std::any::TypeId::of::<bool>()));
        assert!(!BeanUtils::is_simple_property(std::any::TypeId::of::<Vec<String>>()));
    }

    #[test]
    fn test_find_method_name() {
        assert!(BeanUtils::find_method_name("MyStruct", "my_method").is_some());
        assert!(BeanUtils::find_method_name("MyStruct", "").is_none());
    }

    #[test]
    fn test_get_property_type_name() {
        assert!(BeanUtils::get_property_type_name("MyStruct", "name").is_some());
        assert!(BeanUtils::get_property_type_name("MyStruct", "").is_none());
    }

    #[test]
    fn test_copy_properties_map() {
        let mut source = HashMap::new();
        source.insert("key".to_string(), Box::new("value".to_string()) as Box<dyn Any + Send + Sync>);

        let mut target = HashMap::new();
        BeanUtils::copy_properties_map(&source, &mut target);

        assert!(target.contains_key("key"));
    }

    #[test]
    fn test_bean_error() {
        let error = BeanError::new("test error");
        assert_eq!(error.message(), "test error");
        assert_eq!(format!("{}", error), "BeanError: test error");
    }
}
