//! 初始化失败生命周期组件对象。

use vernal_context::{Lifecycle, LifecycleFuture};

/// 模拟 refresh 在提交 `Refreshed` 状态前失败。
pub struct FailingInitializeLifecycle;

impl Lifecycle for FailingInitializeLifecycle {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            Err(
                Box::new(std::io::Error::other("expected initialize failure"))
                    as vernal_core::BoxError,
            )
        })
    }
}
