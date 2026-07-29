//! NamespaceHandler — XML 命名空间处理器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.NamespaceHandler`。
//!
//! 为特定 XML 命名空间（如 `http://www.springframework.org/schema/context`）
//! 注册并驱动自定义元素/属性的解析逻辑。

use crate::bean_definition_parser::BeanDefinitionParser;
use crate::document_loader::Element;

/// Spring 风格的 XML 命名空间处理器 trait。
///
/// 对应 Spring 的 `NamespaceHandler`。
///
/// 由 [`NamespaceHandlerResolver`] 在首次遇到某命名空间时实例化并调用
/// [`NamespaceHandler::init`] 注册解析器，之后框架按元素名查找对应解析器。
pub trait NamespaceHandler: Send + Sync {
    /// 初始化：注册本命名空间下各元素对应的 [`BeanDefinitionParser`]。
    ///
    /// 对应 Spring 的 `void init()`。
    fn init(&mut self);

    /// 返回本地元素名对应的解析器。
    ///
    /// 对应 Spring 的 `BeanDefinitionParser findParserForElement(...)` 的底层。
    fn find_parser(&self, element_name: &str) -> Option<&dyn BeanDefinitionParser>;

    /// 处理元素后返回是否已消费该元素。
    ///
    /// 默认实现返回 `false`，表示交给默认 beans 解析。
    fn decorate(&self) -> bool {
        false
    }
}

/// 一个简单的注册式命名空间处理器基类实现。
///
/// 持有"元素名 → 解析器"映射，便于具体处理器复用。
pub struct NamespaceHandlerSupport {
    /// 元素名 → 解析器。
    parsers: std::collections::HashMap<String, Box<dyn BeanDefinitionParser>>,
}

impl NamespaceHandlerSupport {
    /// 创建空的支持实现。
    pub fn new() -> Self {
        Self {
            parsers: std::collections::HashMap::new(),
        }
    }

    /// 注册一个元素解析器。
    ///
    /// 对应 Spring 的 `registerBeanDefinitionParser`。
    pub fn register_parser(
        &mut self,
        element_name: impl Into<String>,
        parser: Box<dyn BeanDefinitionParser>,
    ) {
        self.parsers.insert(element_name.into(), parser);
    }

    /// 返回已注册的元素名列表。
    pub fn registered_names(&self) -> Vec<String> {
        self.parsers.keys().cloned().collect()
    }
}

impl Default for NamespaceHandlerSupport {
    fn default() -> Self {
        Self::new()
    }
}

impl NamespaceHandler for NamespaceHandlerSupport {
    fn init(&mut self) {
        // 默认无操作：具体子类型覆盖。
    }

    fn find_parser(&self, element_name: &str) -> Option<&dyn BeanDefinitionParser> {
        self.parsers.get(element_name).map(|b| b.as_ref())
    }
}

impl std::fmt::Debug for NamespaceHandlerSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NamespaceHandlerSupport")
            .field("registered_names", &self.parsers.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// 暴露给命名空间处理器使用、忽略元素的占位解析工具。
///
/// 返回给定元素是否为已知命名空间元素（用于错误诊断）。
pub fn is_known_namespace(element: &Element, namespace: &str) -> bool {
    element.namespace_uri == namespace
}
