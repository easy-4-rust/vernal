//! 启动报告脱敏测试使用的失败组件。

use std::io;

use vernal_context::{Lifecycle, LifecycleFuture};
use vernal_core::BoxError;

/// initialize 返回包含模拟密钥错误文本的测试组件。
pub struct FailingLifecycle;

impl Lifecycle for FailingLifecycle {
    fn initialize(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            Err(Box::new(io::Error::other(
                "postgres://admin:super-secret@db/internal initialize failed",
            )) as BoxError)
        })
    }
}
