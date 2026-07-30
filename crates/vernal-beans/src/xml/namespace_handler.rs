//! NamespaceHandler — 命名空间处理器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.NamespaceHandler`。
//!
//! 负责处理自定义 XML 命名空间的元素和属性。

use crate::xml::bean_definition_parser_delegate::BeanDefinitionParserDelegate;

/// 命名空间处理器接口。
///
/// 对应 Spring 的 `NamespaceHandler`。
///
/// 每个自定义命名空间（如 `<context:component-scan>`）
/// 都需要一个 `NamespaceHandler` 来解析其元素。
pub trait NamespaceHandler: Send + Sync {
    /// 初始化处理器。
    ///
    /// 在解析开始前调用，用于注册解析器。
    fn init(&mut self);

    /// 解析命名空间元素。
    ///
    /// # 参数
    /// - `element` — 要解析的 XML 元素
    /// - `delegate` — Bean 定义解析委托
    fn parse(
        &self,
        element_name: &str,
        delegate: &dyn BeanDefinitionParserDelegate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 装饰 Bean 定义。
    ///
    /// 用于在解析后修改 Bean 定义。
    fn decorate(
        &self,
        _element_name: &str,
        _delegate: &dyn BeanDefinitionParserDelegate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
