//! GroovyBeanDefinitionReader — 对应 Java 类：org.springframework.beans.factory.groovy.GroovyBeanDefinitionReader。
//!
//! 对应 Spring beans.factory.groovy 包。
//!
//! 在 Spring 中，`GroovyBeanDefinitionReader` 允许使用 Groovy DSL 语法来定义 Bean。
//! 它提供了 `beans {}` 闭包风格的 API，使得 Bean 定义更加简洁和表达力强。
//! 例如：
//!
//! ```groovy
//! beans {
//!     myService(MyService) {
//!         repository = ref('myRepository')
//!     }
//! }
//! ```
//!
//! ## vernal 中的设计
//!
//! 在 vernal 中，Groovy DSL 被映射为 Rust 的 builder 模式。
//! `GroovyBeanDefinitionReader` 提供链式 API 来模拟 Groovy DSL 的声明式风格。

use std::collections::HashMap;

/// GroovyBeanDefinitionReader — Spring 风格的 Groovy Bean 定义读取器。
///
/// 对应 Java 类：`org.springframework.beans.factory.groovy.GroovyBeanDefinitionReader`。
///
/// 提供 builder 风格的 API 来定义 Bean，模拟 Groovy DSL 的声明式语法。
///
/// ## Java 对比
///
/// | Java Groovy DSL | Rust Builder |
/// |-----------------|--------------|
/// | `beans { ... }` | `GroovyBeanDefinitionReader::new().bean(...)` |
/// | `beanName(Type) { ... }` | `.bean("name", "Type").property("k", v)` |
/// | `ref('name')` | `.bean_ref("name", "target")` |
#[derive(Debug, Clone)]
pub struct GroovyBeanDefinitionReader {
    /// Bean 定义列表
    bean_definitions: Vec<GroovyBeanDefinition>,
    /// 当前正在构建的 Bean 索引
    current_index: Option<usize>,
    /// 全局属性
    global_properties: HashMap<String, String>,
}

/// Groovy 风格的 Bean 定义。
#[derive(Debug, Clone)]
struct GroovyBeanDefinition {
    /// Bean 名称
    name: String,
    /// Bean 类型名
    type_name: String,
    /// 属性注入
    properties: HashMap<String, GroovyValue>,
    /// 构造器参数
    constructor_args: Vec<GroovyValue>,
    /// 是否为单例
    singleton: bool,
    /// 初始化方法
    init_method: Option<String>,
    /// 销毁方法
    destroy_method: Option<String>,
    /// 依赖的 Bean 引用
    depends_on: Vec<String>,
}

/// Groovy 值类型。
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum GroovyValue {
    /// 字面量值
    Literal(String),
    /// Bean 引用
    Ref(String),
    /// 嵌套 Bean 定义
    Nested(GroovyBeanDefinition),
}

impl GroovyBeanDefinitionReader {
    /// 创建新的 GroovyBeanDefinitionReader。
    pub fn new() -> Self {
        Self {
            bean_definitions: Vec::new(),
            current_index: None,
            global_properties: HashMap::new(),
        }
    }

    /// 定义一个新的 Bean。
    ///
    /// 对应 Groovy DSL 的 `beanName(Type) { ... }`。
    ///
    /// # 参数
    /// - `name` — Bean 名称
    /// - `type_name` — Bean 类型名
    pub fn bean(mut self, name: impl Into<String>, type_name: impl Into<String>) -> Self {
        let def = GroovyBeanDefinition {
            name: name.into(),
            type_name: type_name.into(),
            properties: HashMap::new(),
            constructor_args: Vec::new(),
            singleton: true,
            init_method: None,
            destroy_method: None,
            depends_on: Vec::new(),
        };
        self.bean_definitions.push(def);
        self.current_index = Some(self.bean_definitions.len() - 1);
        self
    }

    /// 为当前 Bean 添加属性。
    ///
    /// 对应 Groovy DSL 的 `property = value`。
    pub fn property(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        if let Some(idx) = self.current_index {
            self.bean_definitions[idx]
                .properties
                .insert(name.into(), GroovyValue::Literal(value.into()));
        }
        self
    }

    /// 为当前 Bean 添加 Bean 引用属性。
    ///
    /// 对应 Groovy DSL 的 `property = ref('beanName')`。
    pub fn bean_ref(mut self, name: impl Into<String>, ref_name: impl Into<String>) -> Self {
        if let Some(idx) = self.current_index {
            self.bean_definitions[idx]
                .properties
                .insert(name.into(), GroovyValue::Ref(ref_name.into()));
        }
        self
    }

    /// 为当前 Bean 添加构造器参数。
    ///
    /// 对应 Groovy DSL 的构造器闭包。
    pub fn constructor_arg(mut self, value: impl Into<String>) -> Self {
        if let Some(idx) = self.current_index {
            self.bean_definitions[idx]
                .constructor_args
                .push(GroovyValue::Literal(value.into()));
        }
        self
    }

    /// 设置当前 Bean 为原型作用域。
    pub fn prototype(mut self) -> Self {
        if let Some(idx) = self.current_index {
            self.bean_definitions[idx].singleton = false;
        }
        self
    }

    /// 设置当前 Bean 的初始化方法。
    pub fn init_method(mut self, method: impl Into<String>) -> Self {
        if let Some(idx) = self.current_index {
            self.bean_definitions[idx].init_method = Some(method.into());
        }
        self
    }

    /// 设置当前 Bean 的销毁方法。
    pub fn destroy_method(mut self, method: impl Into<String>) -> Self {
        if let Some(idx) = self.current_index {
            self.bean_definitions[idx].destroy_method = Some(method.into());
        }
        self
    }

    /// 添加依赖关系。
    pub fn depends_on(mut self, bean_name: impl Into<String>) -> Self {
        if let Some(idx) = self.current_index {
            self.bean_definitions[idx].depends_on.push(bean_name.into());
        }
        self
    }

    /// 设置全局属性。
    pub fn global_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.global_properties.insert(key.into(), value.into());
        self
    }

    /// 获取已注册的 Bean 定义数量。
    pub fn bean_count(&self) -> usize {
        self.bean_definitions.len()
    }

    /// 获取所有 Bean 名称。
    pub fn bean_names(&self) -> Vec<&str> {
        self.bean_definitions.iter().map(|d| d.name.as_str()).collect()
    }

    /// 获取指定 Bean 的类型名。
    pub fn bean_type(&self, name: &str) -> Option<&str> {
        self.bean_definitions
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.type_name.as_str())
    }

    /// 获取指定 Bean 的属性数量。
    pub fn bean_property_count(&self, name: &str) -> usize {
        self.bean_definitions
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.properties.len())
            .unwrap_or(0)
    }

    /// 检查指定 Bean 是否为单例。
    pub fn is_singleton(&self, name: &str) -> bool {
        self.bean_definitions
            .iter()
            .find(|d| d.name == name)
            .map(|d| d.singleton)
            .unwrap_or(true)
    }

    /// 获取全局属性。
    pub fn global_properties(&self) -> &HashMap<String, String> {
        &self.global_properties
    }

    /// 检查是否包含指定 Bean。
    pub fn contains_bean(&self, name: &str) -> bool {
        self.bean_definitions.iter().any(|d| d.name == name)
    }

    /// 清空所有定义。
    pub fn clear(&mut self) {
        self.bean_definitions.clear();
        self.current_index = None;
        self.global_properties.clear();
    }
}

impl Default for GroovyBeanDefinitionReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reader_is_empty() {
        let reader = GroovyBeanDefinitionReader::new();
        assert_eq!(reader.bean_count(), 0);
    }

    #[test]
    fn define_single_bean() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("myService", "com.example.MyService");

        assert_eq!(reader.bean_count(), 1);
        assert!(reader.contains_bean("myService"));
        assert_eq!(reader.bean_type("myService"), Some("com.example.MyService"));
    }

    #[test]
    fn bean_with_properties() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("myBean", "MyType")
            .property("name", "test")
            .property("age", "30");

        assert_eq!(reader.bean_property_count("myBean"), 2);
    }

    #[test]
    fn bean_with_ref() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("service", "Service")
            .bean_ref("repository", "myRepository");

        assert_eq!(reader.bean_property_count("service"), 1);
    }

    #[test]
    fn multiple_beans() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("bean1", "Type1")
            .bean("bean2", "Type2")
            .bean("bean3", "Type3");

        assert_eq!(reader.bean_count(), 3);
        let names = reader.bean_names();
        assert!(names.contains(&"bean1"));
        assert!(names.contains(&"bean2"));
        assert!(names.contains(&"bean3"));
    }

    #[test]
    fn prototype_scope() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("proto", "Type")
            .prototype();

        assert!(!reader.is_singleton("proto"));
    }

    #[test]
    fn singleton_by_default() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("single", "Type");

        assert!(reader.is_singleton("single"));
    }

    #[test]
    fn init_and_destroy_methods() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("lifecycle", "Type")
            .init_method("onInit")
            .destroy_method("onDestroy");

        assert!(reader.contains_bean("lifecycle"));
    }

    #[test]
    fn constructor_args() {
        let reader = GroovyBeanDefinitionReader::new()
            .bean("bean", "Type")
            .constructor_arg("arg1")
            .constructor_arg("arg2");

        assert_eq!(reader.bean_count(), 1);
    }

    #[test]
    fn depends_on() {
        let _reader = GroovyBeanDefinitionReader::new()
            .bean("service", "Service")
            .depends_on("dataSource")
            .depends_on("config");
    }

    #[test]
    fn global_properties() {
        let reader = GroovyBeanDefinitionReader::new()
            .global_property("env", "production")
            .global_property("debug", "false");

        assert_eq!(reader.global_properties().len(), 2);
        assert_eq!(reader.global_properties().get("env").unwrap(), "production");
    }

    #[test]
    fn clear_removes_everything() {
        let mut reader = GroovyBeanDefinitionReader::new()
            .bean("bean", "Type")
            .global_property("k", "v");

        reader.clear();
        assert_eq!(reader.bean_count(), 0);
        assert!(reader.global_properties().is_empty());
    }
}
