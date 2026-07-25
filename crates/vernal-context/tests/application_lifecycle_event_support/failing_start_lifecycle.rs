//! 启动失败生命周期组件对象。

use vernal_context::{Lifecycle, LifecycleFuture};

/// 模拟 start 在提交 `Ready` 状态前失败。
pub struct FailingStartLifecycle;

impl Lifecycle for FailingStartLifecycle {
    fn start(&self, _cancellation: tokio_util::sync::CancellationToken) -> LifecycleFuture<'_> {
        Box::pin(async {
            Err(Box::new(std::io::Error::other("expected start failure")) as vernal_core::BoxError)
        })
    }
}
