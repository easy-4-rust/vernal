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
    /// 从已经拆分出的标准 HTTP 元数据创建 owned 快照。
    ///
    /// Poem、Actix Web 等框架虽然使用标准 HTTP 类型，但不直接暴露
    /// `http::Request<B>`；该构造器避免适配器为了捕获元数据而伪造请求 Body。
    #[must_use]
    pub const fn from_parts(
        method: Method,
        uri: Uri,
        version: Version,
        headers: HeaderMap,
    ) -> Self {
        Self {
            method,
            uri,
            version,
            headers,
        }
    }

    /// 从任意标准请求捕获元数据。
    #[must_use]
    pub fn capture<B>(request: &Request<B>) -> Self {
        Self::from_parts(
            request.method().clone(),
            request.uri().clone(),
            request.version(),
            request.headers().clone(),
        )
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
