//! 宏测试使用的问候服务 Trait。

/// 验证 Trait Object 字段注入的最小服务端口。
pub trait Greeting: Send + Sync + 'static {
    /// 返回实现对应的稳定文本。
    fn message(&self) -> &'static str;
}
