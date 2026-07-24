//! Tower 原生响应信封对象。

use tokio::sync::Mutex;

/// 在类型擦除的 AOP 调用链中安全传递 Tower 原生响应。
///
/// `InvocationValue` 必须同时满足 `Send + Sync`，而很多响应 Body 只保证
/// `Send`。Tokio 互斥锁让信封本身可在线程间共享，因此无需给原生响应额外增加
/// `Sync` 约束，也不会读取、缓冲或重建响应 Body。
pub struct TowerResponse<R> {
    response: Mutex<Option<R>>,
}

impl<R> TowerResponse<R> {
    /// 包装一个尚未被消费的原生响应。
    #[must_use]
    pub fn new(response: R) -> Self {
        Self {
            response: Mutex::new(Some(response)),
        }
    }

    /// 原子取出响应；同一信封只能成功消费一次。
    pub async fn take(&self) -> Option<R> {
        self.response.lock().await.take()
    }
}
