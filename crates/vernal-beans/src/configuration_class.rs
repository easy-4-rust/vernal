//! ConfigurationClass — Spring 风格的配置类。
//!
//! 对应 Java 类：`org.springframework.context.annotation.ConfigurationClass`。
//!
//! 表示一个被 `@Configuration` 注解的类，包含其声明的 `@Bean` 方法以及来源资源。

use std::fmt;

/// 单个 `@Bean` 方法的元数据。
///
/// 对应 Spring 的 `BeanMethod`。
#[derive(Debug, Clone)]
pub struct ConfigurationBeanMethod {
    /// 方法名。
    pub method_name: String,
    /// 方法返回类型名。
    pub return_type_name: String,
    /// Bean 名称（默认为方法名）。
    pub bean_name: String,
}

impl ConfigurationBeanMethod {
    /// 创建新的 Bean 方法元数据。
    pub fn new(method_name: impl Into<String>, return_type_name: impl Into<String>) -> Self {
        let method_name = method_name.into();
        let bean_name = method_name.clone();
        Self {
            method_name,
            return_type_name: return_type_name.into(),
            bean_name,
        }
    }

    /// 设置自定义 Bean 名称。
    pub fn with_bean_name(mut self, bean_name: impl Into<String>) -> Self {
        self.bean_name = bean_name.into();
        self
    }
}

/// Spring 风格的配置类。
///
/// 对应 Spring 的 `ConfigurationClass`。
///
/// 表示一个通过 `@Configuration` 注解声明的类，记录其中通过 `@Bean`
/// 注解定义的工厂方法，以及该配置类的来源资源描述。
#[derive(Clone)]
pub struct ConfigurationClass {
    /// 配置类的全限定名。
    class_name: String,
    /// 该类中声明的 `@Bean` 方法名列表（便捷表示）。
    bean_methods: Vec<String>,
    /// 完整的 `@Bean` 方法元数据列表。
    bean_method_details: Vec<ConfigurationBeanMethod>,
    /// 通过 `@Import` 引入的类名列表。
    imported: Vec<String>,
    /// 来源资源描述（如文件路径、类路径位置）。
    source: Option<String>,
    /// 是否为完整 `@Configuration`（`proxyBeanMethods = true`）。
    full: bool,
}

impl fmt::Debug for ConfigurationClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigurationClass")
            .field("class_name", &self.class_name)
            .field("bean_methods", &self.bean_methods)
            .field("imported", &self.imported)
            .field("source", &self.source)
            .field("full", &self.full)
            .finish()
    }
}

impl ConfigurationClass {
    /// 创建新的配置类。
    pub fn new(class_name: impl Into<String>) -> Self {
        Self {
            class_name: class_name.into(),
            bean_methods: Vec::new(),
            bean_method_details: Vec::new(),
            imported: Vec::new(),
            source: None,
            full: true,
        }
    }

    /// 获取配置类全限定名。
    pub fn class_name(&self) -> &str {
        &self.class_name
    }

    /// 获取 `@Bean` 方法名列表。
    pub fn bean_methods(&self) -> &[String] {
        &self.bean_methods
    }

    /// 获取 `@Bean` 方法元数据列表。
    pub fn bean_method_details(&self) -> &[ConfigurationBeanMethod] {
        &self.bean_method_details
    }

    /// 获取通过 `@Import` 引入的类名列表。
    pub fn imported(&self) -> &[String] {
        &self.imported
    }

    /// 获取来源资源描述。
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// 是否为完整 `@Configuration`。
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// 添加一个 `@Bean` 方法名（便捷形式）。
    pub fn add_bean_method(&mut self, method_name: impl Into<String>) {
        let name = method_name.into();
        if !self.bean_methods.contains(&name) {
            self.bean_methods.push(name);
        }
    }

    /// 添加一个完整的 `@Bean` 方法元数据。
    pub fn add_bean_method_detail(&mut self, method: ConfigurationBeanMethod) {
        self.add_bean_method(method.method_name.clone());
        self.bean_method_details.push(method);
    }

    /// 添加一个 `@Import` 的类名。
    pub fn add_import(&mut self, class_name: impl Into<String>) {
        let name = class_name.into();
        if !self.imported.contains(&name) {
            self.imported.push(name);
        }
    }

    /// 设置来源资源描述。
    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = Some(source.into());
    }

    /// 设置是否为完整配置。
    pub fn set_full(&mut self, full: bool) {
        self.full = full;
    }

    /// `@Bean` 方法数量。
    pub fn bean_method_count(&self) -> usize {
        self.bean_methods.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_class_build() {
        let mut config = ConfigurationClass::new("com.example.AppConfig");
        config.add_bean_method_detail(
            ConfigurationBeanMethod::new("createFoo", "com.example.Foo").with_bean_name("foo"),
        );
        config.add_import("com.example.OtherConfig");
        config.set_source("classpath:com/example/AppConfig.class");

        assert_eq!(config.class_name(), "com.example.AppConfig");
        assert_eq!(config.bean_methods(), &["createFoo".to_owned()]);
        assert_eq!(config.bean_method_details().len(), 1);
        assert_eq!(config.bean_method_details()[0].bean_name, "foo");
        assert_eq!(config.imported(), &["com.example.OtherConfig".to_owned()]);
        assert_eq!(
            config.source(),
            Some("classpath:com/example/AppConfig.class")
        );
        assert!(config.is_full());
    }
}
