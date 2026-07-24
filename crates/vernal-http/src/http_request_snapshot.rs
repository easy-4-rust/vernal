//! Owned HTTP 请求元数据快照对象。

use http::{HeaderMap, Method, Request, Uri, Version};

/// 可安全跨越 `.await` 和 Tokio task 的请求元数据副本。
///
/// 该模型吸收 Sa-Token-Rust 在 Axum/Rocket 集成中的关键经验：不要让对原生
/// Request 或非 `Sync` Body 的借用跨越异步边界。快照默认不复制 Body，也不解析
/// Authorization、Cookie 等敏感值。
#[derive(Clone, Debug)]
pub struct HttpRequestSnapshot {
    method: Method,
    uri: Uri,
    version: Version,
    headers: HeaderMap,
}

impl HttpRequestSnapshot {
    /// 从任意标准请求捕获元数据。
    #[must_use]
    pub fn capture<B>(request: &Request<B>) -> Self {
        Self {
            method: request.method().clone(),
            uri: request.uri().clone(),
            version: request.version(),
            headers: request.headers().clone(),
        }
    }

    /// 返回 HTTP 方法。
    #[must_use]
    pub const fn method(&self) -> &Method {
        &self.method
    }

    /// 返回完整 URI。
    #[must_use]
    pub const fn uri(&self) -> &Uri {
        &self.uri
    }

    /// 返回协议版本。
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// 返回请求 Header。
    #[must_use]
    pub const fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
