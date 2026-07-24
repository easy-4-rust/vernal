//! 组件停止 panic 隔离合同使用的生命周期对象。

use std::sync::Arc;

use tokio::sync::Mutex;
use vernal_context::{Lifecycle, LifecycleFuture};

/// 记录 stop 顺序，并可按实例配置在记录后主动 panic。
///
/// 测试通过限定符注册同一 Rust 类型的两个实例，既维持“一文件一对象”，又能验证
/// 逆序关闭遇到用户 panic 后仍继续处理剩余组件。
pub struct StopProbeLifecycle {
    name: &'static str,
    events: Arc<Mutex<Vec<&'static str>>>,
    panics: bool,
}

impl StopProbeLifecycle {
    /// 创建一个带稳定诊断名称的停止探针。
    #[must_use]
    pub fn new(name: &'static str, events: Arc<Mutex<Vec<&'static str>>>, panics: bool) -> Self {
        Self {
            name,
            events,
            panics,
        }
    }
}

impl Lifecycle for StopProbeLifecycle {
    /// 返回测试指定的低基数稳定组件名。
    fn name(&self) -> &'static str {
        self.name
    }

    /// 记录调用后按配置 panic，模拟不可信用户生命周期钩子。
    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            self.events.lock().await.push(self.name);
            assert!(!self.panics, "intentional stop panic");
            Ok(())
        })
    }
}
