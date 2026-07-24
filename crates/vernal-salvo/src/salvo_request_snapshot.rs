//! Salvo 请求元数据快照转换对象。

use salvo::Request;
use vernal_http::HttpRequestSnapshot;

/// 把 Salvo 原生请求的 HTTP 元数据复制成 Vernal owned 快照。
pub(crate) struct SalvoRequestSnapshot;

impl SalvoRequestSnapshot {
    /// 捕获 Method、URI、Version 与 Headers，不读取或缓冲请求 Body。
    pub(crate) fn capture(request: &Request) -> HttpRequestSnapshot {
        HttpRequestSnapshot::from_parts(
            request.method().clone(),
            request.uri().clone(),
            request.version(),
            request.headers().clone(),
        )
    }
}
