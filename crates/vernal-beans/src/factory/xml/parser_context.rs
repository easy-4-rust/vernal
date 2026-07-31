//! ParserContext — Spring 风格的解析器上下文。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.ParserContext`。
//!
//! 在 Spring 中，`ParserContext` 封装了解析过程中需要的上下文信息，
//! 包括 BeanDefinitionReader、BeanDefinitionRegistry 和嵌套状态。
//! 它提供了解析过程中常用的操作方法。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`ParserContext` 存储解析状态和配置，
//! 传递给各个解析器使用。


/// 解析器上下文。
///
/// 对应 Spring 的 `ParserContext`。
///
/// 封装 XML 解析过程中需要的上下文信息。
#[derive(Debug)]
pub struct ParserContext {
    /// 当前解析的命名空间 URI。
    namespace_uri: Option<String>,
    /// 嵌套层级（用于嵌套 Bean 定义）。
    nesting_level: usize,
    /// 文档默认值。
    defaults: DocumentDefaults,
    /// 解析过程中的错误列表。
    errors: Vec<String>,
    /// 解析过程中的警告列表。
    warnings: Vec<String>,
}

/// 文档默认值。
#[derive(Debug, Clone, Default)]
pub struct DocumentDefaults {
    /// 默认作用域。
    pub default_scope: String,
    /// 默认延迟初始化。
    pub default_lazy_init: bool,
    /// 默认自动装配。
    pub default_autowire: String,
}

impl ParserContext {
    /// 创建解析器上下文。
    pub fn new() -> Self {
        Self {
            namespace_uri: None,
            nesting_level: 0,
            defaults: DocumentDefaults::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// 设置命名空间 URI。
    pub fn with_namespace(mut self, uri: impl Into<String>) -> Self {
        self.namespace_uri = Some(uri.into());
        self
    }

    /// 设置文档默认值。
    pub fn with_defaults(mut self, defaults: DocumentDefaults) -> Self {
        self.defaults = defaults;
        self
    }

    /// 获取命名空间 URI。
    pub fn namespace_uri(&self) -> Option<&str> {
        self.namespace_uri.as_deref()
    }

    /// 获取当前嵌套层级。
    pub fn nesting_level(&self) -> usize {
        self.nesting_level
    }

    /// 进入嵌套层级。
    pub fn push_nesting(&mut self) {
        self.nesting_level += 1;
    }

    /// 退出嵌套层级。
    pub fn pop_nesting(&mut self) {
        if self.nesting_level > 0 {
            self.nesting_level -= 1;
        }
    }

    /// 获取文档默认值。
    pub fn defaults(&self) -> &DocumentDefaults {
        &self.defaults
    }

    /// 记录解析错误。
    pub fn add_error(&mut self, message: impl Into<String>) {
        self.errors.push(message.into());
    }

    /// 记录解析警告。
    pub fn add_warning(&mut self, message: impl Into<String>) {
        self.warnings.push(message.into());
    }

    /// 获取错误列表。
    pub fn errors(&self) -> &[String] {
        &self.errors
    }

    /// 获取警告列表。
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// 是否有错误。
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for ParserContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_context_default() {
        let ctx = ParserContext::new();
        assert!(ctx.namespace_uri().is_none());
        assert_eq!(ctx.nesting_level(), 0);
        assert!(!ctx.has_errors());
    }

    #[test]
    fn nesting_level_management() {
        let mut ctx = ParserContext::new();
        assert_eq!(ctx.nesting_level(), 0);
        ctx.push_nesting();
        assert_eq!(ctx.nesting_level(), 1);
        ctx.push_nesting();
        assert_eq!(ctx.nesting_level(), 2);
        ctx.pop_nesting();
        assert_eq!(ctx.nesting_level(), 1);
        ctx.pop_nesting();
        ctx.pop_nesting(); // 不会下溢
        assert_eq!(ctx.nesting_level(), 0);
    }

    #[test]
    fn error_and_warning_collection() {
        let mut ctx = ParserContext::new();
        ctx.add_error("missing class attribute");
        ctx.add_warning("deprecated element");
        assert!(ctx.has_errors());
        assert_eq!(ctx.errors().len(), 1);
        assert_eq!(ctx.warnings().len(), 1);
    }

    #[test]
    fn with_namespace_and_defaults() {
        let defaults = DocumentDefaults {
            default_scope: "prototype".to_string(),
            default_lazy_init: true,
            default_autowire: "byName".to_string(),
        };
        let ctx = ParserContext::new()
            .with_namespace("http://test.ns")
            .with_defaults(defaults);
        assert_eq!(ctx.namespace_uri(), Some("http://test.ns"));
        assert_eq!(ctx.defaults().default_scope, "prototype");
    }
}
