//! AutowireCandidateQualifier — Spring 风格自动装配候选限定符。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AutowireCandidateQualifier`。
//!
//! 用于标识 Bean 的限定符注解类型及其属性，以实现精确的自动装配匹配。

use std::collections::HashMap;
use std::sync::Mutex;

/// Spring 风格自动装配候选限定符。
///
/// 对应 Spring 的 `AutowireCandidateQualifier`。
///
/// 存储一个限定符注解的类型名和属性，用于自动装配时的精确匹配。
pub struct AutowireCandidateQualifier {
    qualifier_type: String,
    attributes: Mutex<HashMap<String, String>>,
}

impl AutowireCandidateQualifier {
    /// 创建新的限定符，指定限定符注解类型名。
    pub fn new(qualifier_type: String) -> Self {
        Self {
            qualifier_type,
            attributes: Mutex::new(HashMap::new()),
        }
    }

    /// 获取限定符注解类型名。
    pub fn qualifier_type(&self) -> &str {
        &self.qualifier_type
    }

    /// 设置属性。
    pub fn set_attribute(&self, name: String, value: String) {
        self.attributes.lock().unwrap().insert(name, value);
    }

    /// 获取属性值。
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.lock().unwrap().get(name).cloned()
    }

    /// 属性数量。
    pub fn attribute_count(&self) -> usize {
        self.attributes.lock().unwrap().len()
    }

    /// 是否包含指定属性。
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.lock().unwrap().contains_key(name)
    }
}

impl std::fmt::Display for AutowireCandidateQualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Qualifier[{}]", self.qualifier_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifier_type_and_attributes() {
        let q = AutowireCandidateQualifier::new("javax.inject.Named".to_string());
        assert_eq!(q.qualifier_type(), "javax.inject.Named");
        assert_eq!(q.attribute_count(), 0);

        q.set_attribute("value".to_string(), "dataSource".to_string());
        assert_eq!(q.get_attribute("value"), Some("dataSource".to_string()));
        assert!(q.has_attribute("value"));
        assert_eq!(q.attribute_count(), 1);
    }

    #[test]
    fn get_missing_attribute_returns_none() {
        let q = AutowireCandidateQualifier::new("Q".to_string());
        assert_eq!(q.get_attribute("missing"), None);
        assert!(!q.has_attribute("missing"));
    }

    #[test]
    fn display_format() {
        let q = AutowireCandidateQualifier::new("org.example.MyQualifier".to_string());
        assert_eq!(format!("{}", q), "Qualifier[org.example.MyQualifier]");
    }
}
