//! PropertyDescriptorUtils — 对应 Spring `org.springframework.beans.PropertyDescriptorUtils`。
//!
//! 属性描述符工具类。

use std::any::TypeId;

/// 属性描述符工具类。
///
/// 对应 Java 类：`org.springframework.beans.PropertyDescriptorUtils`。
pub struct PropertyDescriptorUtils;

impl PropertyDescriptorUtils {
    /// 检查是否是简单属性类型。
    ///
    /// 对应 Java 方法：`boolean isSimpleProperty(Class<?> type)`
    pub fn is_simple_type(type_id: TypeId) -> bool {
        type_id == TypeId::of::<bool>()
            || type_id == TypeId::of::<i8>()
            || type_id == TypeId::of::<i16>()
            || type_id == TypeId::of::<i32>()
            || type_id == TypeId::of::<i64>()
            || type_id == TypeId::of::<i128>()
            || type_id == TypeId::of::<u8>()
            || type_id == TypeId::of::<u16>()
            || type_id == TypeId::of::<u32>()
            || type_id == TypeId::of::<u64>()
            || type_id == TypeId::of::<u128>()
            || type_id == TypeId::of::<f32>()
            || type_id == TypeId::of::<f64>()
            || type_id == TypeId::of::<String>()
            || type_id == TypeId::of::<char>()
    }

    /// 检查是否是简单属性类型（按类型名）。
    pub fn is_simple_type_name(type_name: &str) -> bool {
        matches!(
            type_name,
            "bool" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "u8" | "u16" | "u32" | "u64" | "u128"
                | "f32" | "f64" | "String" | "str" | "char"
        )
    }

    /// 获取类型的默认属性名。
    ///
    /// 对应 Java 方法：`String getDefaultPropertyName(Class<?> type)`
    pub fn get_default_property_name(type_name: &str) -> String {
        // 将类型名转换为属性名风格（首字母小写）
        let mut chars = type_name.chars();
        match chars.next() {
            Some(first) => {
                let lower = first.to_lowercase().to_string();
                format!("{}{}", lower, chars.as_str())
            }
            None => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_simple_type() {
        assert!(PropertyDescriptorUtils::is_simple_type(TypeId::of::<i32>()));
        assert!(PropertyDescriptorUtils::is_simple_type(TypeId::of::<String>()));
        assert!(PropertyDescriptorUtils::is_simple_type(TypeId::of::<bool>()));
        assert!(!PropertyDescriptorUtils::is_simple_type(TypeId::of::<Vec<String>>()));
    }

    #[test]
    fn test_is_simple_type_name() {
        assert!(PropertyDescriptorUtils::is_simple_type_name("i32"));
        assert!(PropertyDescriptorUtils::is_simple_type_name("String"));
        assert!(!PropertyDescriptorUtils::is_simple_type_name("Vec"));
    }

    #[test]
    fn test_get_default_property_name() {
        assert_eq!(PropertyDescriptorUtils::get_default_property_name("String"), "string");
        assert_eq!(PropertyDescriptorUtils::get_default_property_name("MyType"), "myType");
    }
}
