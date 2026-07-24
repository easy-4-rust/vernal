//! Rocket 请求元数据快照转换对象。

use http::{HeaderMap, HeaderName, HeaderValue, Method, Uri, Version};
use rocket::Request;
use vernal_http::HttpRequestSnapshot;

use crate::RocketAopError;

/// 把 Rocket 自有 HTTP 模型转换成 Vernal 使用的 owned 标准快照。
pub(crate) struct RocketRequestSnapshot;

impl RocketRequestSnapshot {
    /// 捕获方法、URI、协议版本与全部 Header，不读取请求 Body。
    pub(crate) fn capture(request: &Request<'_>) -> Result<HttpRequestSnapshot, RocketAopError> {
        let method = Method::from_bytes(request.method().as_str().as_bytes())
            .map_err(|_| RocketAopError::snapshot_conversion("method"))?;
        let uri = request
            .uri()
            .to_string()
            .parse::<Uri>()
            .map_err(|_| RocketAopError::snapshot_conversion("uri"))?;
        let mut headers = HeaderMap::new();
        for header in request.headers().iter() {
            let name = HeaderName::from_bytes(header.name().as_str().as_bytes())
                .map_err(|_| RocketAopError::snapshot_conversion("header-name"))?;
            let value = HeaderValue::from_str(header.value())
                .map_err(|_| RocketAopError::snapshot_conversion("header-value"))?;
            headers.append(name, value);
        }

        // Rocket 的 Request 公共 API 不暴露传输协议版本；0.5 的默认 HTTP 服务
        // 使用 HTTP/1.1 语义。快照显式记录该适配边界，不伪造请求 Body。
        Ok(HttpRequestSnapshot::from_parts(
            method,
            uri,
            Version::HTTP_11,
            headers,
        ))
    }
}
