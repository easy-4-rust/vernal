//! Poem 原生响应信封对象。

use poem::Response;
use tokio::sync::Mutex;

/// 在类型擦除的 Vernal AOP 调用链中安全传递 Poem 原生响应。
///
/// Poem `Response` 只需要满足 `Send`，Vernal `InvocationValue` 则要求
/// `Send + Sync`。Tokio Mutex 只负责一次性移交所有权，不读取、缓冲或重建 Body，
/// 因此响应状态、Header、扩展数据和流式 Body 都保持原样。
pub struct PoemResponse {
    response: Mutex<Option<Response>>,
}

impl PoemResponse {
    /// 包装一个尚未被消费的 Poem 响应。
    #[must_use]
    pub fn new(response: Response) -> Self {
        Self {
            response: Mutex::new(Some(response)),
        }
    }

    /// 原子取出响应；同一信封只能成功消费一次。
    pub async fn take(&self) -> Option<Response> {
        self.response.lock().await.take()
    }
}
