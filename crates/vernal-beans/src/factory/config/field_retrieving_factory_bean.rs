//! FieldRetrievingFactoryBean — 对应 Spring `org.springframework.beans.factory.config.FieldRetrievingFactoryBean`。
//!
//! 字段检索工厂 Bean，用于通过反射获取静态或实例字段的值。

use std::any::Any;
use std::sync::Arc;

/// 字段检索工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.FieldRetrievingFactoryBean`。
///
/// 用于通过反射获取类的静态字段或实例字段的值。
/// 在 Spring XML 配置中使用 `<field name="..."/>` 标签时使用。
///
/// ## 使用场景
///
/// - 获取枚举常量值
/// - 获取静态常量字段
/// - 获取实例字段值
#[derive(Debug)]
pub struct FieldRetrievingFactoryBean {
    /// 目标类名。
    target_class: Option<String>,
    /// 目标对象。
    target_object: Option<Arc<dyn Any + Send + Sync>>,
    /// 字段名称。
    field_name: Option<String>,
    /// 静态字段（类名.字段名）。
    static_field: Option<String>,
}

impl FieldRetrievingFactoryBean {
    /// 创建新的 FieldRetrievingFactoryBean。
    pub fn new() -> Self {
        Self {
            target_class: None,
            target_object: None,
            field_name: None,
            static_field: None,
        }
    }

    /// 设置目标类名。
    pub fn set_target_class(&mut self, target_class: impl Into<String>) {
        self.target_class = Some(target_class.into());
    }

    /// 获取目标类名。
    pub fn target_class(&self) -> Option<&str> {
        self.target_class.as_deref()
    }

    /// 设置目标对象。
    pub fn set_target_object(&mut self, target_object: Arc<dyn Any + Send + Sync>) {
        self.target_object = Some(target_object);
    }

    /// 获取目标对象。
    pub fn target_object(&self) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.target_object.as_ref()
    }

    /// 设置字段名称。
    pub fn set_field_name(&mut self, field_name: impl Into<String>) {
        self.field_name = Some(field_name.into());
    }

    /// 获取字段名称。
    pub fn field_name(&self) -> Option<&str> {
        self.field_name.as_deref()
    }

    /// 设置静态字段（格式：`ClassName.fieldName`）。
    pub fn set_static_field(&mut self, static_field: impl Into<String>) {
        let static_field_str = static_field.into();
        if let Some(dot_pos) = static_field_str.rfind('.') {
            self.target_class = Some(static_field_str[..dot_pos].to_string());
            self.field_name = Some(static_field_str[dot_pos + 1..].to_string());
        }
        self.static_field = Some(static_field_str);
    }

    /// 获取静态字段。
    pub fn static_field(&self) -> Option<&str> {
        self.static_field.as_deref()
    }
}

impl Default for FieldRetrievingFactoryBean {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_retrieving_factory_bean_new() {
        let factory = FieldRetrievingFactoryBean::new();
        assert!(factory.target_class().is_none());
        assert!(factory.target_object().is_none());
        assert!(factory.field_name().is_none());
        assert!(factory.static_field().is_none());
    }

    #[test]
    fn test_field_retrieving_factory_bean_static_field() {
        let mut factory = FieldRetrievingFactoryBean::new();
        factory.set_static_field("java.lang.Integer.MAX_VALUE");

        assert_eq!(factory.static_field(), Some("java.lang.Integer.MAX_VALUE"));
        assert_eq!(factory.target_class(), Some("java.lang.Integer"));
        assert_eq!(factory.field_name(), Some("MAX_VALUE"));
    }

    #[test]
    fn test_field_retrieving_factory_bean_target_and_field() {
        let mut factory = FieldRetrievingFactoryBean::new();
        factory.set_target_class("com.example.MyClass");
        factory.set_field_name("myField");

        assert_eq!(factory.target_class(), Some("com.example.MyClass"));
        assert_eq!(factory.field_name(), Some("myField"));
    }
}
