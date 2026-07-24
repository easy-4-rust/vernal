//! 子系统诊断状态对象。

use serde::Serialize;

/// Adapter 或外部依赖在应用构建时声明的脱敏状态。
///
/// 状态只表达可用性级别，不携带地址、凭证、请求内容或任意错误字符串。需要
/// 进一步排障时应通过受访问控制的日志与可观测系统查询原始错误。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticState {
    /// 构建阶段没有足够证据判断状态。
    #[default]
    Unknown,
    /// 子系统已经配置并可供应用使用。
    Available,
    /// 子系统可以工作，但存在降级能力或非致命告警。
    Degraded,
    /// 子系统在当前应用中不可用。
    Unavailable,
}

impl DiagnosticState {
    /// 返回适合稳定序列化和监控标签的状态名称。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Available => "available",
            Self::Degraded => "degraded",
            Self::Unavailable => "unavailable",
        }
    }
}
