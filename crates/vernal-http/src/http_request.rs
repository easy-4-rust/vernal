//! 框架中立 HTTP 请求对象。

use http::{Request, request};
use tokio_util::sync::CancellationToken;

use crate::{HttpBody, HttpRequestSnapshot};

/// 将标准 `http::Request` 与请求取消信号组合起来。
///
/// 框架适配器只需转换 Body 类型；Method、URI、Version、Header 和 Extensions
/// 继续使用 Rust HTTP 生态的标准对象，不复制协议模型。
pub struct HttpRequest {
    inner: Request<HttpBody>,
    cancellation: CancellationToken,
}

impl HttpRequest {
    /// 从标准请求和取消令牌创建协议请求。
    #[must_use]
    pub fn new(inner: Request<HttpBody>, cancellation: CancellationToken) -> Self {
        Self {
            inner,
            cancellation,
        }
    }

    /// 为不需要外部取消传播的调用创建请求。
    #[must_use]
    pub fn standalone(inner: Request<HttpBody>) -> Self {
        Self::new(inner, CancellationToken::new())
    }

    /// 捕获可安全跨越 `.await` 的请求元数据快照。
    #[must_use]
    pub fn snapshot(&self) -> HttpRequestSnapshot {
        HttpRequestSnapshot::capture(&self.inner)
    }

    /// 返回标准请求引用。
    #[must_use]
    pub const fn inner(&self) -> &Request<HttpBody> {
        &self.inner
    }

    /// 返回标准请求可变引用。
    #[must_use]
    pub const fn inner_mut(&mut self) -> &mut Request<HttpBody> {
        &mut self.inner
    }

    /// 返回请求取消令牌。
    #[must_use]
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// 拆分标准请求 Parts、Body 和取消令牌。
    #[must_use]
    pub fn into_parts(self) -> (request::Parts, HttpBody, CancellationToken) {
        let (parts, body) = self.inner.into_parts();
        (parts, body, self.cancellation)
    }

    /// 消耗包装对象并返回标准请求。
    #[must_use]
    pub fn into_inner(self) -> Request<HttpBody> {
        self.inner
    }
}
