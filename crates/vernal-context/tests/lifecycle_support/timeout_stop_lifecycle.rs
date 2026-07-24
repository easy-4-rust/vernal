//! 生命周期 stop 超时与继续回滚合同使用的组件对象。

use std::{future::pending, sync::Arc};

use tokio::sync::Mutex;
use vernal_context::{Lifecycle, LifecycleFuture};

/// 记录 stop 顺序，并可按实例配置永久等待到 Context 超时。
///
/// 测试使用限定符注册同一类型的依赖组件与被依赖组件，既保持一文件一对象，又
/// 验证逆序关闭遇到超时后仍会继续释放剩余组件。
pub struct TimeoutStopLifecycle {
    name: &'static str,
    events: Arc<Mutex<Vec<&'static str>>>,
    blocks: bool,
}

impl TimeoutStopLifecycle {
    /// 创建立即完成 stop 的顺序探针。
    #[must_use]
    pub fn completing(name: &'static str, events: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self {
            name,
            events,
            blocks: false,
        }
    }

    /// 创建记录后永久等待、由 Context 执行预算 abort 的顺序探针。
    #[must_use]
    pub fn blocking(name: &'static str, events: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self {
            name,
            events,
            blocks: true,
        }
    }
}

impl Lifecycle for TimeoutStopLifecycle {
    /// 返回测试指定的稳定低基数组件名。
    fn name(&self) -> &'static str {
        self.name
    }

    /// 先记录逆序调用证据，再按配置完成或永久等待。
    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            self.events.lock().await.push(self.name);
            if self.blocks {
                pending::<()>().await;
            }
            Ok(())
        })
    }
}
