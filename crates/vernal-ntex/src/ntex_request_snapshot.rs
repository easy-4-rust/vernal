//! Ntex 请求元数据快照转换对象。

use ntex::web::{ErrorRenderer, WebRequest};
use vernal_http::{HeaderMap, HeaderValue, HttpRequestSnapshot};

/// 把 Ntex 原生 HTTP 1.x 元数据复制成 Vernal owned 快照。
pub(crate) struct NtexRequestSnapshot;

impl NtexRequestSnapshot {
    /// 捕获 Method、URI、Version 与 Headers，不读取或缓冲请求 Body。
    pub(crate) fn capture<Err>(request: &WebRequest<Err>) -> HttpRequestSnapshot
    where
        Err: ErrorRenderer,
    {
        let mut headers = HeaderMap::new();
        for (name, value) in request.headers() {
            headers.append(name.clone(), HeaderValue::from(value.clone()));
        }
        HttpRequestSnapshot::from_parts(
            request.method().clone(),
            request.uri().clone(),
            request.version(),
            headers,
        )
    }
}
