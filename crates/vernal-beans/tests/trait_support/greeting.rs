//! 测试用问候服务 Trait。

/// 验证 Vernal Trait Binding 的最小业务端口。
pub trait Greeting: Send + Sync + 'static {
    /// 返回实现对应的稳定问候语。
    fn message(&self) -> &'static str;
}
