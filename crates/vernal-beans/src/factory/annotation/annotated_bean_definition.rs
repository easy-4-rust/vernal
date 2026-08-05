//! AnnotatedBeanDefinition — Spring 风格注解 Bean 定义接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AnnotatedBeanDefinition`。
//!
//! 在 Spring 中，`AnnotatedBeanDefinition` 扩展了 `BeanDefinition`，
//! 添加了注解元数据的访问能力。它是 `@Component`、`@Service` 等
//! 注解驱动 Bean 定义的基础接口。
//!
//! ## 主要功能
//!
//! - 获取类的注解元数据
//! - 判断是否为工厂方法
//! - 获取工厂方法名称

use std::any::TypeId;

/// 注解 Bean 定义接口。
///
/// 对应 Spring 的 `AnnotatedBeanDefinition`。
///
/// 提供注解元数据访问能力，用于注解驱动的 Bean 定义。
pub trait AnnotatedBeanDefinition: Send + Sync {
    /// 获取 Bean 的注解元数据。
    fn get_metadata(&self) -> Option<BeanMetadata>;

    /// 判断指定方法是否为工厂方法。
    fn is_factory_method(&self, name: &str) -> bool;

    /// 获取工厂方法名称。
    fn get_factory_method_name(&self) -> Option<String>;

    /// 获取 Bean 的类型 ID。
    fn get_bean_type(&self) -> TypeId;
}

/// Bean 注解元数据。
///
/// 对应 Spring 的 `AnnotationMetadata`。
///
/// 描述 Bean 类上的注解信息。
#[derive(Debug, Clone)]
pub struct BeanMetadata {
    /// 类名
    pub class_name: String,
    /// 注解数量
    pub annotation_count: usize,
    /// 注解名称列表
    pub annotation_names: Vec<String>,
    /// 是否有 @Configuration 注解
    pub is_configuration: bool,
    /// 是否有 @Component 注解
    pub is_component: bool,
}

impl BeanMetadata {
    /// 创建新的 Bean 元数据。
    pub fn new(class_name: String, annotation_count: usize) -> Self {
        Self {
            class_name,
            annotation_count,
            annotation_names: Vec::new(),
            is_configuration: false,
            is_component: false,
        }
    }

    /// 设置注解名称列表。
    pub fn with_annotation_names(mut self, names: Vec<String>) -> Self {
        self.annotation_names = names;
        self.annotation_count = self.annotation_names.len();
        self
    }

    /// 设置是否为 @Configuration。
    pub fn with_configuration(mut self, is_config: bool) -> Self {
        self.is_configuration = is_config;
        self
    }

    /// 设置是否为 @Component。
    pub fn with_component(mut self, is_component: bool) -> Self {
        self.is_component = is_component;
        self
    }

    /// 是否包含指定注解。
    pub fn has_annotation(&self, name: &str) -> bool {
        self.annotation_names.iter().any(|n| n == name)
    }
}

/// 通用注解 Bean 定义实现。
///
/// 对应 Spring 的 `AnnotatedGenericBeanDefinition`。
pub struct GenericAnnotatedBeanDefinition {
    bean_class_name: String,
    factory_method: Option<String>,
    metadata: BeanMetadata,
}

impl GenericAnnotatedBeanDefinition {
    /// 创建新的通用注解 Bean 定义。
    pub fn new(bean_class_name: String) -> Self {
        Self {
            bean_class_name: bean_class_name.clone(),
            factory_method: None,
            metadata: BeanMetadata::new(bean_class_name, 0),
        }
    }

    /// 设置工厂方法。
    pub fn with_factory_method(mut self, name: String) -> Self {
        self.factory_method = Some(name);
        self
    }

    /// 设置注解数量。
    pub fn with_annotation_count(mut self, count: usize) -> Self {
        self.metadata.annotation_count = count;
        self
    }

    /// 设置注解元数据。
    pub fn with_metadata(mut self, metadata: BeanMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// 获取 Bean 类名。
    pub fn bean_class_name(&self) -> &str {
        &self.bean_class_name
    }
}

impl AnnotatedBeanDefinition for GenericAnnotatedBeanDefinition {
    fn get_metadata(&self) -> Option<BeanMetadata> {
        Some(self.metadata.clone())
    }
    fn is_factory_method(&self, name: &str) -> bool {
        self.factory_method.as_deref() == Some(name)
    }
    fn get_factory_method_name(&self) -> Option<String> {
        self.factory_method.clone()
    }
    fn get_bean_type(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bean_metadata_creation() {
        let metadata = BeanMetadata::new("com.example.MyService".to_string(), 3);
        assert_eq!(metadata.class_name, "com.example.MyService");
        assert_eq!(metadata.annotation_count, 3);
    }

    #[test]
    fn bean_metadata_with_annotations() {
        let metadata = BeanMetadata::new("MyClass".to_string(), 0)
            .with_annotation_names(vec!["@Component".to_string(), "@Service".to_string()])
            .with_component(true);

        assert_eq!(metadata.annotation_count, 2);
        assert!(metadata.has_annotation("@Component"));
        assert!(metadata.has_annotation("@Service"));
        assert!(!metadata.has_annotation("@Configuration"));
        assert!(metadata.is_component);
    }

    #[test]
    fn annotated_definition_basic() {
        let def = GenericAnnotatedBeanDefinition::new("MyService".to_string());
        assert_eq!(def.bean_class_name(), "MyService");
        assert!(def.get_factory_method_name().is_none());
        assert!(def.get_metadata().is_some());
    }

    #[test]
    fn annotated_definition_with_factory_method() {
        let def = GenericAnnotatedBeanDefinition::new("Factory".to_string())
            .with_factory_method("createBean".to_string());

        assert!(def.is_factory_method("createBean"));
        assert!(!def.is_factory_method("other"));
        assert_eq!(
            def.get_factory_method_name(),
            Some("createBean".to_string())
        );
    }

    #[test]
    fn annotated_definition_with_metadata() {
        let metadata = BeanMetadata::new("Config".to_string(), 0).with_configuration(true);
        let def = GenericAnnotatedBeanDefinition::new("Config".to_string()).with_metadata(metadata);

        let meta = def.get_metadata().unwrap();
        assert!(meta.is_configuration);
    }
}
