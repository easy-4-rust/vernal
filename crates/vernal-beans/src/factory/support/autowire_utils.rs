//! AutowireUtils — Spring 风格的自动装配工具。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AutowireUtils`。

/// Spring 风格的自动装配工具。
pub struct AutowireUtils;

impl AutowireUtils {
    pub fn new() -> Self { Self }
    pub fn is_autowire_type(name: &str) -> bool {
        let upper = name.to_uppercase();
        upper == "SET" || upper == "GET" || upper == "IS"
    }
    pub fn resolve_autowire_value(name: &str) -> String {
        if name.starts_with("set") && name.len() > 3 {
            let chars: String = name.chars().skip(3).collect();
            let first = chars.chars().next().unwrap_or(' ').to_ascii_lowercase();
            format!("{}{}", first, &chars[1..])
        } else if name.starts_with("get") && name.len() > 3 {
            name[3..].to_string()
        } else {
            name.to_string()
        }
    }
}
impl Default for AutowireUtils { fn default() -> Self { Self::new() } }
