//! Tide 原生响应信封对象。

use tide::Response;
use tokio::sync::Mutex;

use crate::TideAopError;

/// 在借用型 Send-AOP 链中安全传递 Tide 原生响应。
///
/// Mutex 只负责一次性所有权移交，使只满足 `Send` 的 Response 能进入要求
/// `Send + Sync` 的 `InvocationValue`；Body、状态、Header 与扩展不会被重建。
/// 拦截器也可返回自己创建的信封实现 Tide 原生响应短路。
pub struct TideResponse {
    response: Mutex<Option<Response>>,
}

impl TideResponse {
    /// 包装拦截器生成的原生响应。
    #[must_use]
    pub fn new(response: Response) -> Self {
        Self {
            response: Mutex::new(Some(response)),
        }
    }

    /// 创建等待下游 `Next` 写入的空信封。
    pub(crate) fn empty() -> Self {
        Self {
            response: Mutex::new(None),
        }
    }

    /// 写入下游产生的唯一响应。
    pub(crate) async fn store(&self, response: Response) -> Result<(), TideAopError> {
        let mut slot = self.response.lock().await;
        if slot.is_some() {
            return Err(TideAopError::ResponseAlreadyStored);
        }
        *slot = Some(response);
        Ok(())
    }

    /// 原子取出原生响应。
    pub async fn take(&self) -> Option<Response> {
        self.response.lock().await.take()
    }
}
