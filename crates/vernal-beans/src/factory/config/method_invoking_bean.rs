//! MethodInvokingBean — 对应 Spring `org.springframework.beans.factory.config.MethodInvokingBean`。
//!
//! 方法调用 Bean，用于调用指定方法并获取返回值。

use std::any::Any;
use std::sync::Arc;

/// 方法调用 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.MethodInvokingBean`。
///
/// 用于调用指定对象的方法或静态方法，并将返回值作为 Bean。
///
/// ## 使用场景
///
/// - 调用工厂方法创建 Bean
/// - 调用静态方法获取实例
/// - 调用已有 Bean 的方法
#[derive(Debug)]
pub struct MethodInvokingBean {
    /// 目标对象。
    target_object: Option<Arc<dyn Any + Send + Sync>>,
    /// 目标类名（用于静态方法调用）。
    target_class: Option<String>,
    /// 方法名称。
    method_name: String,
    /// 方法参数。
    arguments: Vec<Arc<dyn Any + Send + Sync>>,
}

impl MethodInvokingBean {
    /// 创建新的 MethodInvokingBean。
    pub fn new(method_name: impl Into<String>) -> Self {
        Self {
            target_object: None,
            target_class: None,
            method_name: method_name.into(),
            arguments: Vec::new(),
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

    /// 是否是静态方法调用。
    pub fn is_static(&self) -> bool {
        self.target_class.is_some() && self.target_object.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_invoking_bean_new() {
        let bean = MethodInvokingBean::new("toString");
        assert_eq!(bean.method_name(), "toString");
        assert!(bean.target_object().is_none());
        assert!(bean.target_class().is_none());
        assert!(bean.arguments().is_empty());
    }

    #[test]
    fn test_method_invoking_bean_with_target_object() {
        let mut bean = MethodInvokingBean::new("size");
        bean.set_target_object(Arc::new(vec![1, 2, 3]));

        assert!(bean.target_object().is_some());
        assert!(!bean.is_static());
    }

    #[test]
    fn test_method_invoking_bean_static() {
        let mut bean = MethodInvokingBean::new("valueOf");
        bean.set_target_class("java.lang.String");
        bean.set_arguments(vec![Arc::new(42i32)]);

        assert!(bean.is_static());
        assert_eq!(bean.target_class(), Some("java.lang.String"));
        assert_eq!(bean.arguments().len(), 1);
    }
}
