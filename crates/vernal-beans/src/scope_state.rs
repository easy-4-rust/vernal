//! 自定义组件作用域状态对象。

/// 自定义作用域的显式释放状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScopeState {
    /// 可以解析作用域组件并注册关闭钩子。
    #[default]
    Open,
    /// 已禁止新解析，正在等待既有构造并执行关闭钩子。
    Closing,
    /// 缓存和关闭钩子已经释放。
    Closed,
}
