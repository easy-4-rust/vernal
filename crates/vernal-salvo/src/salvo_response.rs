//! Salvo 原生响应信封对象。

use salvo::Response;
use tokio::sync::Mutex;

use crate::SalvoAopError;

/// 在借用型 Send-AOP 链与 Salvo `&mut Response` 之间移交响应所有权。
///
/// `Response` 只需要满足 `Send`，Vernal `InvocationValue` 要求 `Send + Sync`。
/// Tokio Mutex 只负责一次性所有权移交，不读取、缓冲或重建 Body，因此状态、
/// Header、扩展与流式响应保持 Salvo 原生语义。拦截器也可返回自己创建的信封
/// 实现原生响应短路。
pub struct SalvoResponse {
    response: Mutex<Option<Response>>,
}

impl SalvoResponse {
    /// 包装一个由拦截器生成、尚未被消费的原生响应。
    #[must_use]
    pub fn new(response: Response) -> Self {
        Self {
            response: Mutex::new(Some(response)),
        }
    }

    /// 创建等待目标写入的空信封。
    pub(crate) fn empty() -> Self {
        Self {
            response: Mutex::new(None),
        }
    }

    /// 写入目标产生的唯一响应。
    ///
    /// # Errors
    ///
    /// 信封已经包含响应时返回结构化错误，不覆盖第一次业务结果。
    pub(crate) async fn store(&self, response: Response) -> Result<(), SalvoAopError> {
        let mut slot = self.response.lock().await;
        if slot.is_some() {
            return Err(SalvoAopError::ResponseAlreadyStored);
        }
        *slot = Some(response);
        Ok(())
    }

    /// 原子取出原生响应；同一信封只能成功消费一次。
    pub async fn take(&self) -> Option<Response> {
        self.response.lock().await.take()
    }
}
