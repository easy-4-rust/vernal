//! AutowireCandidateQualifier — Spring 风格的自动装配候选限定符。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AutowireCandidateQualifier`。
//!
//! 用于标记和限定自动装配候选 Bean，支持按限定符名称和属性过滤。

use std::any::Any;
use std::collections::HashMap;

/// Spring 风格的自动装配候选限定符。
///
/// 对应 Spring 的 `AutowireCandidateQualifier`。
///
/// 为自动装配候选提供限定信息，支持：
/// - 限定符类型名称（如 `@Qualifier("myBean")`）
/// - 自定义属性（K-V 对）
///
/// 用于 `QualifierAnnotationAutowireCandidateResolver` 等高级解析器，
/// 在多个同类型候选 Bean 中按限定符匹配最合适的 Bean。
///
/// ## 示例
///
/// ```rust,ignore
/// use vernal_beans::autowire_candidate_qualifier::AutowireCandidateQualifier;
///
/// let mut qualifier = AutowireCandidateQualifier::new("primary");
/// qualifier.set_attribute("value", Box::new("main".to_string()));
/// assert_eq!(qualifier.type_name(), "primary");
/// ```
#[derive(Debug)]
pub struct AutowireCandidateQualifier {
    /// 限定符类型名称（如 `"org.springframework.beans.factory.annotation.Qualifier"`）。
    type_name: String,
    /// 限定符的属性集合。
    attributes: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl AutowireCandidateQualifier {
    /// 创建一个新的 AutowireCandidateQualifier。
    ///
    /// # 参数
    ///
    /// * `type_name` — 限定符的类型名称
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
            attributes: HashMap::new(),
        }
    }

    /// 获取限定符的类型名称。
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// 获取指定属性的值。
    ///
    /// # 参数
    ///
    /// * `key` — 属性名
    ///
    /// # 返回
    ///
    /// - `Some(&dyn Any)` — 属性值
    /// - `None` — 属性不存在
    pub fn get_attribute(&self, key: &str) -> Option<&dyn Any> {
        self.attributes.get(key).map(|b| &**b as &dyn Any)
    }

    /// 设置属性值。
    ///
    /// # 参数
    ///
    /// * `key` — 属性名
    /// * `value` — 属性值
    pub fn set_attribute(&mut self, key: impl Into<String>, value: Box<dyn Any + Send + Sync>) {
        self.attributes.insert(key.into(), value);
    }

    /// 检查是否包含指定属性。
    ///
    /// # 参数
    ///
    /// * `key` — 属性名
    pub fn has_attribute(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// 获取属性数量。
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// 获取所有属性名。
    pub fn attribute_names(&self) -> Vec<&str> {
        self.attributes.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_qualifier() {
        let qualifier = AutowireCandidateQualifier::new("test");
        assert_eq!(qualifier.type_name(), "test");
        assert_eq!(qualifier.attribute_count(), 0);
    }

    #[test]
    fn test_set_and_get_attribute() {
        let mut qualifier = AutowireCandidateQualifier::new("primary");
        qualifier.set_attribute("value", Box::new("main".to_string()));
        assert!(qualifier.has_attribute("value"));
        assert_eq!(qualifier.attribute_count(), 1);

        let attr = qualifier.get_attribute("value");
        assert!(attr.is_some());
        let val = attr.unwrap().downcast_ref::<String>();
        assert_eq!(val, Some(&"main".to_string()));
    }

    #[test]
    fn test_attribute_names() {
        let mut qualifier = AutowireCandidateQualifier::new("custom");
        qualifier.set_attribute("a", Box::new(1i32));
        qualifier.set_attribute("b", Box::new(2i32));
        let names = qualifier.attribute_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
    }
}
