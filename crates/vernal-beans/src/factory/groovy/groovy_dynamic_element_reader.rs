//! GroovyDynamicElementReader — 对应 Java 类：org.springframework.beans.factory.groovy.GroovyDynamicElementReader。
//!
//! 对应 Spring beans.factory.groovy 包。
//!
//! 在 Spring 中，`GroovyDynamicElementReader` 用于读取 Groovy DSL 中的动态元素。
//! 它能够处理 Groovy 脚本中动态生成的 Bean 定义元素，支持运行时的
//! 动态属性解析和 Bean 注册。
//!
//! ## 使用场景
//!
//! - 运行时动态 Bean 定义
//! - Groovy 脚本中的条件 Bean 注册
//! - 动态属性解析

use std::collections::HashMap;

/// GroovyDynamicElementReader — Spring 风格的动态元素读取器。
///
/// 对应 Java 类：`org.springframework.beans.factory.groovy.GroovyDynamicElementReader`。
///
/// 用于读取和解析 Groovy DSL 中的动态元素。
/// 支持动态属性、条件 Bean 定义和运行时类型解析。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `GroovyDynamicElementReader` | `GroovyDynamicElementReader` |
/// | 动态属性读取 | `read_property(name)` |
/// | 元素注册 | `register_element(name, type)` |
#[derive(Debug, Clone)]
pub struct GroovyDynamicElementReader {
    /// 已注册的元素类型（元素名 -> 类型名）
    elements: HashMap<String, String>,
    /// 元素属性（元素名 -> 属性映射）
    element_properties: HashMap<String, HashMap<String, String>>,
    /// 当前正在处理的元素
    current_element: Option<String>,
    /// 读取计数
    read_count: usize,
    /// 是否启用严格模式
    strict: bool,
}

impl GroovyDynamicElementReader {
    /// 创建新的 GroovyDynamicElementReader。
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            element_properties: HashMap::new(),
            current_element: None,
            read_count: 0,
            strict: true,
        }
    }

    /// 创建非严格模式的读取器。
    pub fn lenient() -> Self {
        Self {
            strict: false,
            ..Self::new()
        }
    }

    /// 注册一个动态元素。
    ///
    /// # 参数
    /// - `name` — 元素名称
    /// - `type_name` — 元素类型名
    pub fn register_element(&mut self, name: impl Into<String>, type_name: impl Into<String>) {
        let n = name.into();
        let t = type_name.into();
        self.elements.insert(n.clone(), t);
        self.element_properties.entry(n).or_default();
    }

    /// 设置当前正在处理的元素。
    pub fn set_current_element(&mut self, name: impl Into<String>) -> bool {
        let n = name.into();
        if self.elements.contains_key(&n) {
            self.current_element = Some(n);
            true
        } else if !self.strict {
            // 非严格模式下自动注册未知元素
            self.register_element(&n, "dynamic");
            self.current_element = Some(n);
            true
        } else {
            false
        }
    }

    /// 获取当前元素名称。
    pub fn current_element(&self) -> Option<&str> {
        self.current_element.as_deref()
    }

    /// 为当前元素设置属性。
    pub fn set_property(&mut self, name: impl Into<String>, value: impl Into<String>) -> bool {
        if let Some(ref current) = self.current_element {
            if let Some(props) = self.element_properties.get_mut(current) {
                props.insert(name.into(), value.into());
                return true;
            }
        }
        false
    }

    /// 读取当前元素的属性。
    pub fn read_property(&mut self, name: &str) -> Option<String> {
        self.read_count += 1;
        self.current_element.as_ref().and_then(|current| {
            self.element_properties
                .get(current)
                .and_then(|props| props.get(name).cloned())
        })
    }

    /// 读取指定元素的属性。
    pub fn read_element_property(&mut self, element: &str, name: &str) -> Option<String> {
        self.read_count += 1;
        self.element_properties
            .get(element)
            .and_then(|props| props.get(name).cloned())
    }

    /// 获取已注册的元素数量。
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }

    /// 获取所有元素名称。
    pub fn element_names(&self) -> Vec<&str> {
        self.elements.keys().map(|s| s.as_str()).collect()
    }

    /// 检查元素是否已注册。
    pub fn contains_element(&self, name: &str) -> bool {
        self.elements.contains_key(name)
    }

    /// 获取元素的类型名。
    pub fn element_type(&self, name: &str) -> Option<&str> {
        self.elements.get(name).map(|s| s.as_str())
    }

    /// 获取元素的属性数量。
    pub fn element_property_count(&self, name: &str) -> usize {
        self.element_properties
            .get(name)
            .map(|props| props.len())
            .unwrap_or(0)
    }

    /// 获取总读取次数。
    pub fn read_count(&self) -> usize {
        self.read_count
    }

    /// 是否为严格模式。
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// 清空所有元素。
    pub fn clear(&mut self) {
        self.elements.clear();
        self.element_properties.clear();
        self.current_element = None;
        self.read_count = 0;
    }

    /// 移除指定元素。
    pub fn remove_element(&mut self, name: &str) -> bool {
        let removed = self.elements.remove(name).is_some();
        self.element_properties.remove(name);
        if self.current_element.as_deref() == Some(name) {
            self.current_element = None;
        }
        removed
    }
}

impl Default for GroovyDynamicElementReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reader_is_empty() {
        let reader = GroovyDynamicElementReader::new();
        assert_eq!(reader.element_count(), 0);
        assert!(reader.current_element().is_none());
    }

    #[test]
    fn register_element() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("service", "MyService");
        assert_eq!(reader.element_count(), 1);
        assert!(reader.contains_element("service"));
        assert_eq!(reader.element_type("service"), Some("MyService"));
    }

    #[test]
    fn set_current_element() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("bean", "Type");
        assert!(reader.set_current_element("bean"));
        assert_eq!(reader.current_element(), Some("bean"));
    }

    #[test]
    fn set_current_unknown_strict_fails() {
        let mut reader = GroovyDynamicElementReader::new();
        assert!(!reader.set_current_element("unknown"));
        assert!(reader.current_element().is_none());
    }

    #[test]
    fn set_current_unknown_lenient_auto_registers() {
        let mut reader = GroovyDynamicElementReader::lenient();
        assert!(reader.set_current_element("unknown"));
        assert_eq!(reader.current_element(), Some("unknown"));
        assert!(reader.contains_element("unknown"));
    }

    #[test]
    fn set_and_read_property() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("bean", "Type");
        reader.set_current_element("bean");
        reader.set_property("name", "value");

        assert_eq!(reader.read_property("name"), Some("value".to_string()));
        assert_eq!(reader.read_count(), 1);
    }

    #[test]
    fn read_missing_property_returns_none() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("bean", "Type");
        reader.set_current_element("bean");
        assert_eq!(reader.read_property("missing"), None);
    }

    #[test]
    fn read_without_current_element() {
        let mut reader = GroovyDynamicElementReader::new();
        assert_eq!(reader.read_property("any"), None);
    }

    #[test]
    fn read_element_property_directly() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("bean", "Type");
        reader.set_current_element("bean");
        reader.set_property("key", "val");

        assert_eq!(
            reader.read_element_property("bean", "key"),
            Some("val".to_string())
        );
    }

    #[test]
    fn element_names() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("a", "A");
        reader.register_element("b", "B");
        let mut names = reader.element_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn remove_element() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("bean", "Type");
        assert!(reader.remove_element("bean"));
        assert!(!reader.contains_element("bean"));
        assert_eq!(reader.element_count(), 0);
    }

    #[test]
    fn remove_current_element_clears_current() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("bean", "Type");
        reader.set_current_element("bean");
        reader.remove_element("bean");
        assert!(reader.current_element().is_none());
    }

    #[test]
    fn clear_removes_everything() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("a", "A");
        reader.register_element("b", "B");
        reader.set_current_element("a");
        reader.set_property("k", "v");

        reader.clear();
        assert_eq!(reader.element_count(), 0);
        assert!(reader.current_element().is_none());
        assert_eq!(reader.read_count(), 0);
    }

    #[test]
    fn element_property_count() {
        let mut reader = GroovyDynamicElementReader::new();
        reader.register_element("bean", "Type");
        reader.set_current_element("bean");
        reader.set_property("a", "1");
        reader.set_property("b", "2");
        assert_eq!(reader.element_property_count("bean"), 2);
    }
}
