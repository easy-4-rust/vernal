//! 框架中立 HTTP 响应对象。

use http::{Response, response};

use crate::HttpBody;

/// 使用标准 HTTP Parts 与 Vernal Body 的协议响应。
pub struct HttpResponse {
    inner: Response<HttpBody>,
}

impl HttpResponse {
    /// 创建协议响应。
    #[must_use]
    pub const fn new(inner: Response<HttpBody>) -> Self {
        Self { inner }
    }

    /// 返回标准响应引用。
    #[must_use]
    pub const fn inner(&self) -> &Response<HttpBody> {
        &self.inner
    }

    /// 返回标准响应可变引用。
    #[must_use]
    pub const fn inner_mut(&mut self) -> &mut Response<HttpBody> {
        &mut self.inner
    }

    /// 拆分标准响应 Parts 与 Body。
    #[must_use]
    pub fn into_parts(self) -> (response::Parts, HttpBody) {
        self.inner.into_parts()
    }

    /// 消耗包装对象并返回标准响应。
    #[must_use]
    pub fn into_inner(self) -> Response<HttpBody> {
        self.inner
    }
}

impl From<Response<HttpBody>> for HttpResponse {
    fn from(response: Response<HttpBody>) -> Self {
        Self::new(response)
    }
}

impl From<HttpResponse> for Response<HttpBody> {
    fn from(response: HttpResponse) -> Self {
        response.into_inner()
    }
}
