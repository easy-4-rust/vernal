//! 返回敏感根因的失败 Runner 对象。

use vernal_context::ApplicationRunner;

/// 模拟安全缓存预热失败并携带不应进入默认日志的下游正文。
pub struct FailingRunner;

impl ApplicationRunner for FailingRunner {
    type Error = std::io::Error;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        Err(std::io::Error::other("secret-runner-downstream-response"))
    }
}
