//! BeanDescriptorUtils — 对应 Spring `org.springframework.beans.BeanDescriptorUtils`。
//!
//! Bean 描述符工具类。
//!
//! 提供 Bean 描述符相关的实用方法，用于判断类型是否为简单类型、
//! 获取类型默认属性名等操作。
//!
//! # Spring 对标
//!
//! 对应 Java 类 `org.springframework.beans.BeanDescriptorUtils`。
//! 在 Rust 中，由于没有运行时反射，这些方法基于类型名字符串或 `TypeId` 进行判断。

/// Bean 描述符工具类。
///
/// 对应 Java 类：`org.springframework.beans.BeanDescriptorUtils`。
///
/// 提供对 Bean 类型元数据的操作方法。
pub struct BeanDescriptorUtils;

impl BeanDescriptorUtils {
    /// 判断类型名是否为简单类型。
    ///
    /// 简单类型包括：布尔类型、整数类型、浮点类型、字符类型和字符串类型。
    ///
    /// 对应 Java 方法：`boolean isSimpleProperty(Class<?> type)`
    ///
    /// # Arguments
    ///
    /// * `type_name` - 类型名称字符串
    ///
    /// # Returns
    ///
    /// 如果是简单类型返回 `true`，否则返回 `false`。
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

    /// 判断类型名是否为数字类型。
    ///
    /// # Arguments
    ///
    /// * `type_name` - 类型名称字符串
    ///
    /// # Returns
    ///
    /// 如果是数字类型返回 `true`，否则返回 `false`。
    pub fn is_numeric_type_name(type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16"
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
                | "usize"
                | "isize"
        )
    }

    /// 判断类型名是否为整数类型。
    ///
    /// # Arguments
    ///
    /// * `type_name` - 类型名称字符串
    ///
    /// # Returns
    ///
    /// 如果是整数类型返回 `true`，否则返回 `false`。
    pub fn is_integer_type_name(type_name: &str) -> bool {
        matches!(
            type_name,
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "isize"
        )
    }

    /// 判断类型名是否为浮点类型。
    ///
    /// # Arguments
    ///
    /// * `type_name` - 类型名称字符串
    ///
    /// # Returns
    ///
    /// 如果是浮点类型返回 `true`，否则返回 `false`。
    pub fn is_floating_point_type_name(type_name: &str) -> bool {
        matches!(type_name, "f32" | "f64")
    }

    /// 获取类型的默认属性名。
    ///
    /// 将类型名转换为属性名风格（首字母小写）。
    ///
    /// 对应 Java 方法：`String getDefaultPropertyName(Class<?> type)`
    ///
    /// # Arguments
    ///
    /// * `type_name` - 类型名称字符串
    ///
    /// # Returns
    ///
    /// 转换后的属性名。
    pub fn get_default_property_name(type_name: &str) -> String {
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
    fn test_is_simple_type_name() {
        // 布尔类型
        assert!(BeanDescriptorUtils::is_simple_type_name("bool"));

        // 整数类型
        assert!(BeanDescriptorUtils::is_simple_type_name("i32"));
        assert!(BeanDescriptorUtils::is_simple_type_name("u64"));
        assert!(BeanDescriptorUtils::is_simple_type_name("usize"));

        // 浮点类型
        assert!(BeanDescriptorUtils::is_simple_type_name("f32"));
        assert!(BeanDescriptorUtils::is_simple_type_name("f64"));

        // 字符串类型
        assert!(BeanDescriptorUtils::is_simple_type_name("String"));
        assert!(BeanDescriptorUtils::is_simple_type_name("str"));
        assert!(BeanDescriptorUtils::is_simple_type_name("char"));

        // 非简单类型
        assert!(!BeanDescriptorUtils::is_simple_type_name("Vec"));
        assert!(!BeanDescriptorUtils::is_simple_type_name("HashMap"));
        assert!(!BeanDescriptorUtils::is_simple_type_name("Option"));
    }

    #[test]
    fn test_is_numeric_type_name() {
        assert!(BeanDescriptorUtils::is_numeric_type_name("i32"));
        assert!(BeanDescriptorUtils::is_numeric_type_name("f64"));
        assert!(BeanDescriptorUtils::is_numeric_type_name("usize"));
        assert!(!BeanDescriptorUtils::is_numeric_type_name("bool"));
        assert!(!BeanDescriptorUtils::is_numeric_type_name("String"));
    }

    #[test]
    fn test_is_integer_type_name() {
        assert!(BeanDescriptorUtils::is_integer_type_name("i32"));
        assert!(BeanDescriptorUtils::is_integer_type_name("u64"));
        assert!(BeanDescriptorUtils::is_integer_type_name("usize"));
        assert!(!BeanDescriptorUtils::is_integer_type_name("f32"));
        assert!(!BeanDescriptorUtils::is_integer_type_name("bool"));
    }

    #[test]
    fn test_is_floating_point_type_name() {
        assert!(BeanDescriptorUtils::is_floating_point_type_name("f32"));
        assert!(BeanDescriptorUtils::is_floating_point_type_name("f64"));
        assert!(!BeanDescriptorUtils::is_floating_point_type_name("i32"));
        assert!(!BeanDescriptorUtils::is_floating_point_type_name("bool"));
    }

    #[test]
    fn test_get_default_property_name() {
        assert_eq!(
            BeanDescriptorUtils::get_default_property_name("String"),
            "string"
        );
        assert_eq!(
            BeanDescriptorUtils::get_default_property_name("MyType"),
            "myType"
        );
        assert_eq!(BeanDescriptorUtils::get_default_property_name("ABC"), "aBC");
        assert_eq!(BeanDescriptorUtils::get_default_property_name(""), "");
    }
}
