//! Actix 请求元数据快照转换对象。

use std::io;

use actix_web::dev::ServiceRequest;
use vernal_http::{HeaderMap, HeaderName, HeaderValue, HttpRequestSnapshot, Method, Uri, Version};

use crate::ActixRequestSnapshotError;

/// 把 Actix HTTP 0.2 原生元数据转换成 Vernal HTTP 1.x owned 快照。
pub(crate) struct ActixRequestSnapshot;

impl ActixRequestSnapshot {
    /// 捕获不包含 Body 的请求元数据。
    ///
    /// # Errors
    ///
    /// 标准 Method、URI、Version 或 Header 无法转换时返回字段化错误。
    pub(crate) fn capture(
        request: &ServiceRequest,
    ) -> Result<HttpRequestSnapshot, ActixRequestSnapshotError> {
        let method = Method::from_bytes(request.method().as_str().as_bytes())
            .map_err(|source| ActixRequestSnapshotError::new("method", source))?;
        let uri = request
            .uri()
            .to_string()
            .parse::<Uri>()
            .map_err(|source| ActixRequestSnapshotError::new("URI", source))?;
        let version = Self::version(request.version())?;
        let mut headers = HeaderMap::new();
        for (name, value) in request.headers() {
            let name = HeaderName::from_bytes(name.as_str().as_bytes())
                .map_err(|source| ActixRequestSnapshotError::new("header name", source))?;
            let value = HeaderValue::from_bytes(value.as_bytes())
                .map_err(|source| ActixRequestSnapshotError::new("header value", source))?;
            headers.append(name, value);
        }
        Ok(HttpRequestSnapshot::from_parts(
            method, uri, version, headers,
        ))
    }

    /// 显式映射两个 `http` 主版本的协议版本常量。
    fn version(version: actix_web::http::Version) -> Result<Version, ActixRequestSnapshotError> {
        if version == actix_web::http::Version::HTTP_09 {
            Ok(Version::HTTP_09)
        } else if version == actix_web::http::Version::HTTP_10 {
            Ok(Version::HTTP_10)
        } else if version == actix_web::http::Version::HTTP_11 {
            Ok(Version::HTTP_11)
        } else if version == actix_web::http::Version::HTTP_2 {
            Ok(Version::HTTP_2)
        } else if version == actix_web::http::Version::HTTP_3 {
            Ok(Version::HTTP_3)
        } else {
            Err(ActixRequestSnapshotError::new(
                "version",
                io::Error::new(io::ErrorKind::InvalidData, "unsupported HTTP version"),
            ))
        }
    }
}
