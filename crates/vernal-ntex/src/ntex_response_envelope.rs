//! Ntex Local-AOP 响应信封对象。

use std::cell::RefCell;

use ntex::web::WebResponse;

use crate::NtexAopError;

/// 在不可克隆的 Ntex Request/Response 所有权与 AOP 擦除值之间传递响应。
pub(crate) struct NtexResponseEnvelope {
    response: RefCell<Option<WebResponse>>,
}

impl NtexResponseEnvelope {
    /// 创建尚未写入响应的信封。
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            response: RefCell::new(None),
        }
    }

    /// 写入目标产生的唯一响应。
    ///
    /// # Errors
    ///
    /// 信封已经包含响应时返回结构化错误，不覆盖第一次业务结果。
    pub(crate) fn store(&self, response: WebResponse) -> Result<(), NtexAopError> {
        let mut slot = self.response.borrow_mut();
        if slot.is_some() {
            return Err(NtexAopError::ResponseAlreadyStored);
        }
        *slot = Some(response);
        Ok(())
    }

    /// 取出目标生成的响应。
    pub(crate) fn take(&self) -> Option<WebResponse> {
        self.response.borrow_mut().take()
    }
}
