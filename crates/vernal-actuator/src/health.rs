//! 健康检查。

/// 健康状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// 健康
    Up,
    /// 不健康
    Down,
    /// 未知
    Unknown,
}

/// 健康信息。
#[derive(Debug, Clone)]
pub struct Health {
    pub status: HealthStatus,
    pub details: Vec<(&'static str, String)>,
}

/// 健康检查指示器 trait。
///
/// 对标 Spring 的 `HealthIndicator`。
pub trait HealthIndicator: Send + Sync {
    /// 指示器名称。
    fn name(&self) -> &'static str;

    /// 执行健康检查。
    fn check(&self) -> Health;
}
