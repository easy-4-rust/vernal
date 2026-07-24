//! Gotham AOP 原生响应信封对象。

use gotham::helpers::http::Body;
use http::Response;
use tokio::sync::Mutex;

use crate::GothamAopError;

/// 在类型擦除的 AOP 返回值中保存 Gotham 原生响应。
///
/// Gotham 的 Body 满足 `Send` 但不要求 `Sync`。信封用 Tokio Mutex 提供独占
/// 所有权转移，因此既能进入 `InvocationValue`，又不会复制或重建原生 Body。
pub struct GothamResponse {
    response: Mutex<Option<Response<Body>>>,
}

impl GothamResponse {
    /// 用一个原生响应创建可供拦截器短路返回的信封。
    #[must_use]
    pub const fn new(response: Response<Body>) -> Self {
        Self {
            response: Mutex::const_new(Some(response)),
        }
    }

    /// 创建等待最终 Gotham Handler 写入的空信封。
    pub(crate) const fn empty() -> Self {
        Self {
            response: Mutex::const_new(None),
        }
    }

    /// 写入最终 Handler 返回的原生响应。
    pub(crate) async fn store(&self, response: Response<Body>) -> Result<(), GothamAopError> {
        let mut slot = self.response.lock().await;
        if slot.is_some() {
            return Err(GothamAopError::ResponseAlreadyStored);
        }
        *slot = Some(response);
        Ok(())
    }

    /// 取出原生响应；同一信封只能成功消费一次。
    pub async fn take(&self) -> Option<Response<Body>> {
        self.response.lock().await.take()
    }
}
