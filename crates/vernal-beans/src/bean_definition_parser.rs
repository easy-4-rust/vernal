//! BeanDefinitionParser — 自定义 XML 元素解析器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.BeanDefinitionParser`。
//!
//! 由各命名空间处理器注册，负责把特定的自定义 XML 元素解析为一条或多条
//! Bean 定义并注册到注册表。

use crate::bean_definition::BeanDefinition;
use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::document_loader::Element;

/// Spring 风格的 Bean 定义解析器 trait。
///
/// 对应 Spring 的 `BeanDefinitionParser`。
///
/// 实现方负责解析传入的 [`Element`] 并向注册表注册若干 Bean 定义，
/// 返回新注册的 Bean 定义数量。
pub trait BeanDefinitionParser: Send + Sync + std::fmt::Debug {
    /// 解析元素并注册 Bean 定义。
    ///
    /// 对应 Spring 的
    /// `BeanDefinition parse(Element element, ParserContext parserContext)`。
    ///
    /// # 错误
    ///
    /// 解析或注册失败时返回 `Err`。
    fn parse(
        &self,
        element: &Element,
        registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;

    /// 返回此解析器负责的 Bean 定义类型名（用于诊断），默认为空。
    fn bean_type_name(&self) -> &str {
        ""
    }
}

/// 便捷函数：把单个 Bean 定义注册到注册表，返回 `Ok(1)`。
///
/// 对应 Spring 中各解析器返回单条定义的常见模式。
pub fn register_single(
    registry: &mut dyn BeanDefinitionRegistry,
    bean_name: String,
    definition: Box<dyn BeanDefinition>,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    registry.register_bean_definition(bean_name, definition)?;
    Ok(1)
}
