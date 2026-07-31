//! AutowireUtils — Spring 风格的自动装配工具。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AutowireUtils`。
//!
//! 提供自动装配相关的工具方法，包括属性名提取和
//! Java Bean 命名约定的处理。

/// Spring 风格的自动装配工具。
///
/// 对应 Spring 的 `AutowireUtils`。
///
/// 提供属性名提取和 Java Bean 命名约定相关的工具方法。
pub struct AutowireUtils;

impl AutowireUtils {
    /// 创建工具实例。
    pub fn new() -> Self { Self }

    /// 判断名称是否为 setter/getter 前缀（set/get/is）。
    pub fn is_autowire_type(name: &str) -> bool {
        let upper = name.to_uppercase();
        upper == "SET" || upper == "GET" || upper == "IS"
    }

    /// 从 Java Bean 风格的方法名提取属性名。
    ///
    /// - `setName` -> `name`
    /// - `getName` -> `name`
    /// - `isValid` -> `valid`
    /// - 其他 -> 原样返回
    pub fn resolve_autowire_value(name: &str) -> String {
        if name.starts_with("set") && name.len() > 3 {
            let rest = &name[3..];
            let mut chars = rest.chars();
            match chars.next() {
                Some(first) => {
                    let lower = first.to_ascii_lowercase();
                    format!("{}{}", lower, chars.as_str())
                }
                None => name.to_string(),
            }
        } else if name.starts_with("get") && name.len() > 3 {
            let rest = &name[3..];
            let mut chars = rest.chars();
            match chars.next() {
                Some(first) => {
                    let lower = first.to_ascii_lowercase();
                    format!("{}{}", lower, chars.as_str())
                }
                None => name.to_string(),
            }
        } else if name.starts_with("is") && name.len() > 2 {
            let rest = &name[2..];
            let mut chars = rest.chars();
            match chars.next() {
                Some(first) => {
                    let lower = first.to_ascii_lowercase();
                    format!("{}{}", lower, chars.as_str())
                }
                None => name.to_string(),
            }
        } else {
            name.to_string()
        }
    }
}

impl Default for AutowireUtils { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_autowire_type_recognizes_prefixes() {
        assert!(AutowireUtils::is_autowire_type("set"));
        assert!(AutowireUtils::is_autowire_type("get"));
        assert!(AutowireUtils::is_autowire_type("is"));
        assert!(AutowireUtils::is_autowire_type("SET"));
        assert!(!AutowireUtils::is_autowire_type("name"));
    }

    #[test]
    fn resolve_setter_to_property_name() {
        assert_eq!(AutowireUtils::resolve_autowire_value("setName"), "name");
        assert_eq!(AutowireUtils::resolve_autowire_value("setURL"), "uRL");
    }

    #[test]
    fn resolve_getter_to_property_name() {
        assert_eq!(AutowireUtils::resolve_autowire_value("getName"), "name");
        assert_eq!(AutowireUtils::resolve_autowire_value("getAge"), "age");
    }

    #[test]
    fn resolve_is_to_property_name() {
        assert_eq!(AutowireUtils::resolve_autowire_value("isValid"), "valid");
        assert_eq!(AutowireUtils::resolve_autowire_value("isActive"), "active");
    }

    #[test]
    fn resolve_plain_name_unchanged() {
        assert_eq!(AutowireUtils::resolve_autowire_value("foo"), "foo");
    }
}
