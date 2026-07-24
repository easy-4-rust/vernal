//! Tide `http-types` 请求元数据快照转换对象。

use std::str::FromStr;

use tide::{Request, http::Version as TideVersion};
use vernal_http::{HeaderMap, HeaderName, HeaderValue, HttpRequestSnapshot, Method, Uri, Version};

use crate::TideAopError;

/// 把 Tide 2.x HTTP 模型转换成 Vernal 使用的 `http` 1.x owned 快照。
pub(crate) struct TideRequestSnapshot;

impl TideRequestSnapshot {
    /// 捕获 Method、URI、Version 与 Headers，不读取或缓冲请求 Body。
    ///
    /// Tide 的合法类型通常都可无损转换；仍保留显式错误分支，避免版本边界变化
    /// 演变成 panic。合成测试请求没有协议版本时按 HTTP/1.1 记录。
    pub(crate) fn capture<State>(
        request: &Request<State>,
    ) -> Result<HttpRequestSnapshot, TideAopError> {
        let method = Method::from_str(request.method().as_ref())
            .map_err(|_| TideAopError::snapshot_conversion("method"))?;
        let uri = Uri::from_str(request.url().as_str())
            .map_err(|_| TideAopError::snapshot_conversion("URI"))?;
        let version = match request.version() {
            Some(TideVersion::Http0_9) => Version::HTTP_09,
            Some(TideVersion::Http1_0) => Version::HTTP_10,
            Some(TideVersion::Http2_0) => Version::HTTP_2,
            Some(TideVersion::Http3_0) => Version::HTTP_3,
            None | Some(_) => Version::HTTP_11,
        };
        let mut headers = HeaderMap::new();
        for (name, values) in request {
            let name = HeaderName::from_bytes(name.as_str().as_bytes())
                .map_err(|_| TideAopError::snapshot_conversion("header name"))?;
            for value in values {
                let value = HeaderValue::from_str(value.as_str())
                    .map_err(|_| TideAopError::snapshot_conversion("header value"))?;
                headers.append(name.clone(), value);
            }
        }
        Ok(HttpRequestSnapshot::from_parts(
            method, uri, version, headers,
        ))
    }
}
