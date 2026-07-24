//! Web 请求作用域状态对象。

/// 请求作用域的显式释放状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScopeState {
    /// 可以解析请求级对象并注册关闭钩子。
    #[default]
    Open,
    /// 正在取消并逆序执行关闭钩子。
    Closing,
    /// 所有缓存与钩子已经释放。
    Closed,
}
