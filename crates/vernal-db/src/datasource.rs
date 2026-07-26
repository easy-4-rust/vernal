//! 数据源 trait。

/// 数据源 trait。
///
/// 对标 Spring 的 `DataSource`。
/// 具体实现由 hutool-vernal-db 桥接提供。
pub trait DataSource: Send + Sync {
    /// 获取数据源名称。
    fn name(&self) -> &str;

    /// 获取连接状态。
    fn is_connected(&self) -> bool;
}
