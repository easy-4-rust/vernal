//! 模板解析器上下文。
//!
//! 对标 Spring 的 `TemplateParserContext`：支持 `#{expr}` 模板语法。

use crate::parser_context::ParserContext;

/// 默认模板解析器上下文。
///
/// 使用 `#{` 前缀和 `}` 后缀。
/// 对标 Spring 的 `org.springframework.expression.common.TemplateParserContext`。
pub struct TemplateParserContextImpl {
    prefix: String,
    suffix: String,
}

impl TemplateParserContextImpl {
    /// 创建模板解析器上下文。
    #[must_use]
    pub fn new() -> Self {
        Self {
            prefix: "#{".to_string(),
            suffix: "}".to_string(),
        }
    }

    /// 自定义前后缀。
    #[must_use]
    pub fn with_delimiters(prefix: impl Into<String>, suffix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            suffix: suffix.into(),
        }
    }
}

impl Default for TemplateParserContextImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserContext for TemplateParserContextImpl {
    fn is_template(&self) -> bool {
        true
    }

    fn expression_prefix(&self) -> &str {
        &self.prefix
    }

    fn expression_suffix(&self) -> &str {
        &self.suffix
    }
}
