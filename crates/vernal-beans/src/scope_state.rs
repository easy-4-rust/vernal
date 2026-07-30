//! 作用域状态枚举。

/// 作用域的生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeState {
    /// 作用域已打开，可以创建 Bean。
    #[default]
    Open,
    /// 作用域正在关闭，不再创建新 Bean。
    Closing,
    /// 作用域已关闭。
    Closed,
}
