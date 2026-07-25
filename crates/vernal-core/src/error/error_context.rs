//! 错误附加上下文。
//!
//! 当需要在保留错误身份（domain + code）的同时附带动态诊断信息时，
//! 使用 [`ErrorContext`] 而非直接修改错误消息。
//!
//! 上下文信息用于服务端诊断，**不应**直接暴露给客户端（可能包含敏感数据）。

use std::fmt;

/// 错误附加上下文。
///
/// 携带一组键值对形式的诊断信息，与 [`super::VernalError::WithContext`] 配合使用。
/// 上下文信息在 `Display` 实现中被省略（防止泄露到日志默认输出），
/// 仅在显式调用 [`ErrorContext::entries`] 时才可访问。
///
/// # 示例
///
/// ```rust
/// use vernal_core::error::ErrorContext;
///
/// let ctx = ErrorContext::new()
///     .with("component", "DatabasePool")
///     .with("reason", "connection timeout after 30s");
///
/// assert_eq!(ctx.entries().len(), 2);
/// ```
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// 键值对列表，保持插入顺序
    entries: Vec<(&'static str, String)>,
}

impl ErrorContext {
    /// 创建空的错误上下文。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 附加一个键值对。键必须是静态字符串（零分配），值可以是动态字符串。
    ///
    /// # 参数
    /// - `key`：诊断键名（如 "component"、"field"、"reason"）
    /// - `value`：诊断值（如组件名、字段名、失败原因）
    #[must_use]
    pub fn with(mut self, key: &'static str, value: impl Into<String>) -> Self {
        self.entries.push((key, value.into()));
        self
    }

    /// 获取所有上下文条目。
    #[must_use]
    pub fn entries(&self) -> &[(&'static str, String)] {
        &self.entries
    }

    /// 上下文是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 上下文条目数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display 中不输出上下文内容，防止敏感信息泄露到日志默认格式
        write!(f, "[{} diagnostic entries]", self.entries.len())
    }
}
