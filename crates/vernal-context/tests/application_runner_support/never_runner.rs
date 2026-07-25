//! 永不主动完成的 Runner 对象。

use std::convert::Infallible;

use vernal_context::ApplicationRunner;

/// 模拟忽略取消但仍可由 Tokio abort 丢弃的启动 Future。
pub struct NeverRunner;

impl ApplicationRunner for NeverRunner {
    type Error = Infallible;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        std::future::pending().await
    }
}
