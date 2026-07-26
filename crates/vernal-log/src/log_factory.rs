//! 日志工厂。

/// 日志工厂。
///
/// 对标 Spring 的 `LogFactory`，底层使用 `tracing`。
pub struct LogFactory;

impl LogFactory {
    /// 初始化日志系统（使用默认配置）。
    pub fn init() {
        // 默认使用 tracing_subscriber 的 env_filter
        // 用户可通过 RUST_LOG 环境变量控制日志级别
    }

    /// 初始化日志系统（指定默认级别）。
    pub fn init_with_level(level: &str) {
        let _ = level;
        // TODO: 集成 tracing_subscriber::fmt::init()
    }
}
