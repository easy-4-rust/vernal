//! 关闭等待者取消安全合同使用的阻塞生命周期组件。

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::sync::Notify;
use vernal_context::{Lifecycle, LifecycleFuture};

/// 在 stop 阶段等待测试显式放行，并记录停止是否最终完成。
///
/// 该对象让测试能够在 Context 已进入后台关闭、但组件尚未释放的确定时刻取消首个
/// 等待者，从而证明完成 stop 的所有权属于协调器，而不是属于某个 `close()` Future。
pub struct CancellationSafeLifecycle {
    stop_entered: Arc<Notify>,
    stop_release: Arc<Notify>,
    stopped: AtomicBool,
}

impl CancellationSafeLifecycle {
    /// 创建一个尚未进入 stop 的测试组件。
    #[must_use]
    pub fn new(stop_entered: Arc<Notify>, stop_release: Arc<Notify>) -> Self {
        Self {
            stop_entered,
            stop_release,
            stopped: AtomicBool::new(false),
        }
    }

    /// 返回 stop Future 是否已经越过测试放行点并正常完成。
    #[must_use]
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}

impl Lifecycle for CancellationSafeLifecycle {
    /// 阻塞到测试放行，随后以 Release 顺序发布完成状态。
    fn stop(&self) -> LifecycleFuture<'_> {
        Box::pin(async {
            self.stop_entered.notify_one();
            self.stop_release.notified().await;
            self.stopped.store(true, Ordering::Release);
            Ok(())
        })
    }
}
