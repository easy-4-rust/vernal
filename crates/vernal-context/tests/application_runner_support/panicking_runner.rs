//! 发生 panic 的应用 Runner 对象。

use std::convert::Infallible;

use vernal_context::ApplicationRunner;

/// 模拟用户启动逻辑 panic。
pub struct PanickingRunner;

impl ApplicationRunner for PanickingRunner {
    type Error = Infallible;

    async fn run(
        &self,
        _cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<(), Self::Error> {
        panic!("secret-runner-panic-payload")
    }
}
