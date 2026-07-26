//! 解析器上下文 trait。
//!
//! 对标 Spring 的 `ParserContext`。

/// 解析器上下文 trait。
///
/// 影响表达式解析（模板模式 vs 标准模式）。
/// 对标 Spring 的 `org.springframework.expression.ParserContext`。
pub trait ParserContext: Send + Sync {
    /// 是否为模板模式。
    fn is_template(&self) -> bool;

    /// 获取表达式前缀（如 `#{`）。
    fn expression_prefix(&self) -> &str;

    /// 获取表达式后缀（如 `}`）。
    fn expression_suffix(&self) -> &str;
}

/// 默认模板解析器上下文。
///
/// 使用 `#{` 前缀和 `}` 后缀。
#[derive(Debug, Clone)]
pub struct TemplateParserContext {
    prefix: String,
    suffix: String,
}

impl TemplateParserContext {
    /// 创建模板解析器上下文。
    #[must_use]
    pub fn new(prefix: impl Into<String>, suffix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            suffix: suffix.into(),
        }
    }
}

impl Default for TemplateParserContext {
    fn default() -> Self {
        Self::new("#{", "}")
    }
}

impl ParserContext for TemplateParserContext {
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
