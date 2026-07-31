//! BeanDefinitionDecorator — Spring 风格的 Bean 定义装饰器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.BeanDefinitionDecorator`。
//!
//! 在 Spring 中，`BeanDefinitionDecorator` 用于在解析 XML 时装饰已有的 Bean 定义。
//! 当 XML 中出现自定义属性或子元素时，装饰器会修改 Bean 定义。
//! 例如，AOP 的 `<aop:scoped-proxy>` 就是通过装饰器实现的。
//!
//! ## 设计说明
//!
//! 在 vernal 中，装饰器返回一个新的装饰结果，
//! 包含原始 Bean 定义名称和附加的元数据。

use std::collections::HashMap;

/// Bean 定义装饰结果。
#[derive(Debug, Clone)]
pub struct DecoratedBeanDefinition {
    /// 被装饰的 Bean 名称。
    pub bean_name: String,
    /// 装饰后的附加属性。
    pub additional_properties: HashMap<String, String>,
    /// 装饰器类型标识。
    pub decorator_type: String,
}

/// Bean 定义装饰器接口。
///
/// 对应 Spring 的 `BeanDefinitionDecorator`。
///
/// 用于在 XML 解析时装饰已有的 Bean 定义。
pub trait BeanDefinitionDecorator: Send + Sync {
    /// 装饰 Bean 定义。
    ///
    /// 对应 Spring 的 `BeanDefinitionHolder decorate(Node, BeanDefinitionHolder, ParserContext)`。
    ///
    /// # 参数
    /// - `bean_name` — 要装饰的 Bean 名称
    /// - `element_name` — 触发装饰的 XML 元素名
    /// - `attributes` — 装饰元素的属性
    ///
    /// # 返回
    /// 装饰结果。
    fn decorate(
        &self,
        bean_name: &str,
        element_name: &str,
        attributes: &[(String, String)],
    ) -> Result<DecoratedBeanDefinition, Box<dyn std::error::Error + Send + Sync>>;
}

/// 作用域代理装饰器。
///
/// 对应 Spring AOP 的 `<aop:scoped-proxy>` 装饰器。
#[derive(Debug, Default)]
pub struct ScopedProxyDecorator;

impl ScopedProxyDecorator {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self
    }
}

impl BeanDefinitionDecorator for ScopedProxyDecorator {
    fn decorate(
        &self,
        bean_name: &str,
        _element_name: &str,
        _attributes: &[(String, String)],
    ) -> Result<DecoratedBeanDefinition, Box<dyn std::error::Error + Send + Sync>> {
        let mut props = HashMap::new();
        props.insert("scoped-proxy".to_string(), "true".to_string());
        props.insert("proxy-target-class".to_string(), "true".to_string());

        Ok(DecoratedBeanDefinition {
            bean_name: bean_name.to_string(),
            additional_properties: props,
            decorator_type: "scoped-proxy".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_proxy_decorator_adds_properties() {
        let decorator = ScopedProxyDecorator::new();
        let result = decorator.decorate("myBean", "scoped-proxy", &[]).unwrap();
        assert_eq!(result.bean_name, "myBean");
        assert_eq!(result.decorator_type, "scoped-proxy");
        assert_eq!(result.additional_properties.get("scoped-proxy"), Some(&"true".to_string()));
    }

    #[test]
    fn decorated_result_preserves_bean_name() {
        let decorator = ScopedProxyDecorator::new();
        let result = decorator.decorate("serviceA", "scoped-proxy", &[]).unwrap();
        assert_eq!(result.bean_name, "serviceA");
    }

    #[test]
    fn proxy_target_class_defaults_to_true() {
        let decorator = ScopedProxyDecorator::new();
        let result = decorator.decorate("bean", "scoped-proxy", &[]).unwrap();
        assert_eq!(result.additional_properties.get("proxy-target-class"), Some(&"true".to_string()));
    }
}
