//! 统一启动测试探针对象。

use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Mutex, Semaphore},
    time::{Instant, sleep},
};

/// 记录生命周期事件并控制阻塞 initialize 的共享测试探针。
///
/// 探针只服务公开合同测试，不参与生产启动实现。事件使用异步互斥保护，释放门使用
/// Semaphore 保存许可，避免 `Notify` 在测试线程与生命周期任务交接时丢失通知。
pub struct LaunchProbe {
    events: Mutex<Vec<&'static str>>,
    initialize_release: Semaphore,
}

impl LaunchProbe {
    /// 创建尚未释放阻塞 initialize 的空探针。
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            events: Mutex::new(Vec::new()),
            initialize_release: Semaphore::new(0),
        })
    }

    /// 追加一个稳定生命周期事件。
    pub async fn record(&self, event: &'static str) {
        self.events.lock().await.push(event);
    }

    /// 返回当前事件序列快照。
    pub async fn events(&self) -> Vec<&'static str> {
        self.events.lock().await.clone()
    }

    /// 等待指定事件在预算内出现。
    pub async fn wait_for(&self, expected: &'static str) {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if self.events.lock().await.contains(&expected) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for launch event {expected}"
            );
            sleep(Duration::from_millis(1)).await;
        }
    }

    /// 等待测试线程允许阻塞 initialize 继续。
    pub async fn wait_for_initialize_release(&self) {
        let permit = self
            .initialize_release
            .acquire()
            .await
            .expect("test probe semaphore remains open");
        permit.forget();
    }

    /// 允许一个阻塞 initialize 调用继续。
    pub fn release_initialize(&self) {
        self.initialize_release.add_permits(1);
    }
}
