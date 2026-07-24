//! 框架中立 Web 错误详情对象。

use std::sync::Arc;

use crate::ProblemKind;

/// 由 Adapter 映射为原生响应的稳定问题描述。
///
/// 该对象借鉴 RFC 9457 的核心字段，但不依赖 HTTP crate，因此也可映射为 Tonic
/// Status。`detail` 默认应为脱敏文本，内部错误通过观测系统关联 `RequestId`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProblemDetails {
    kind: ProblemKind,
    status: u16,
    title: Arc<str>,
    detail: Option<Arc<str>>,
    instance: Option<Arc<str>>,
}

impl ProblemDetails {
    /// 创建问题描述。
    #[must_use]
    pub fn new(kind: ProblemKind, status: u16, title: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            status,
            title: title.into(),
            detail: None,
            instance: None,
        }
    }

    /// 设置脱敏详情。
    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<Arc<str>>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// 设置问题实例标识或 URI。
    #[must_use]
    pub fn with_instance(mut self, instance: impl Into<Arc<str>>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    /// 返回问题分类。
    #[must_use]
    pub const fn kind(&self) -> ProblemKind {
        self.kind
    }

    /// 返回建议协议状态码。
    #[must_use]
    pub const fn status(&self) -> u16 {
        self.status
    }

    /// 返回稳定标题。
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 返回可选脱敏详情。
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    /// 返回可选问题实例。
    #[must_use]
    pub fn instance(&self) -> Option<&str> {
        self.instance.as_deref()
    }
}
