//! Registry 诊断测试使用的问候端口。

/// 用于验证 Trait Binding 快照的测试端口。
pub trait Greeting: Send + Sync {
    /// 返回固定问候语。
    fn greet(&self) -> &'static str;
}
