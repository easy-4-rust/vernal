//! MethodInvokingFactoryBean — 对应 Spring `org.springframework.beans.factory.config.MethodInvokingFactoryBean`。
//!
//! 方法调用工厂 Bean，用于通过方法调用创建 Bean 实例。

use std::any::Any;
use std::sync::Arc;

/// 方法调用工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.MethodInvokingFactoryBean`。
///
/// 与 `MethodInvokingBean` 类似，但作为 FactoryBean 使用，
/// 返回方法调用的结果作为 Bean 实例。
///
/// ## 使用场景
///
/// - 通过工厂方法创建 Bean
/// - 调用静态工厂方法
/// - 获取已有 Bean 的属性值
#[derive(Debug)]
pub struct MethodInvokingFactoryBean {
    /// 目标对象。
    target_object: Option<Arc<dyn Any + Send + Sync>>,
    /// 目标类名（用于静态方法调用）。
    target_class: Option<String>,
    /// 方法名称。
    method_name: String,
    /// 方法参数。
    arguments: Vec<Arc<dyn Any + Send + Sync>>,
    /// 是否是单例。
    singleton: bool,
}

impl MethodInvokingFactoryBean {
    /// 创建新的 MethodInvokingFactoryBean。
    pub fn new(method_name: impl Into<String>) -> Self {
        Self {
            target_object: None,
            target_class: None,
            method_name: method_name.into(),
            arguments: Vec::new(),
            singleton: true,
        }
    }

    /// 设置目标对象。
    pub fn set_target_object(&mut self, target_object: Arc<dyn Any + Send + Sync>) {
        self.target_object = Some(target_object);
    }

    /// 获取目标对象。
    pub fn target_object(&self) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.target_object.as_ref()
    }

    /// 设置目标类名（用于静态方法调用）。
    pub fn set_target_class(&mut self, target_class: impl Into<String>) {
        self.target_class = Some(target_class.into());
    }

    /// 获取目标类名。
    pub fn target_class(&self) -> Option<&str> {
        self.target_class.as_deref()
    }

    /// 设置方法参数。
    pub fn set_arguments(&mut self, arguments: Vec<Arc<dyn Any + Send + Sync>>) {
        self.arguments = arguments;
    }

    /// 获取方法参数。
    pub fn arguments(&self) -> &[Arc<dyn Any + Send + Sync>] {
        &self.arguments
    }

    /// 获取方法名称。
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// 设置是否单例。
    pub fn set_singleton(&mut self, singleton: bool) {
        self.singleton = singleton;
    }

    /// 是否单例。
    pub fn is_singleton(&self) -> bool {
        self.singleton
    }

    /// 是否是静态方法调用。
    pub fn is_static(&self) -> bool {
        self.target_class.is_some() && self.target_object.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_invoking_factory_bean_new() {
        let bean = MethodInvokingFactoryBean::new("create");
        assert_eq!(bean.method_name(), "create");
        assert!(bean.target_object().is_none());
        assert!(bean.target_class().is_none());
        assert!(bean.is_singleton());
    }

    #[test]
    fn test_method_invoking_factory_bean_with_target_object() {
        let mut bean = MethodInvokingFactoryBean::new("get");
        bean.set_target_object(Arc::new(String::from("hello")));

        assert!(bean.target_object().is_some());
        assert!(!bean.is_static());
    }

    #[test]
    fn test_method_invoking_factory_bean_static() {
        let mut bean = MethodInvokingFactoryBean::new("getInstance");
        bean.set_target_class("com.example.Factory");

        assert!(bean.is_static());
        assert_eq!(bean.target_class(), Some("com.example.Factory"));
    }
}
