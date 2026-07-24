//! 有限 HTTP Body 收集结果对象。

use bytes::Bytes;
use http::HeaderMap;

/// 在显式大小上限内收集的 Body 数据与 Trailer。
///
/// Trailer 与普通 Header 分开保存，避免桥接 Hyper 或 Tonic 时丢失帧语义。
#[derive(Debug)]
pub struct CollectedBody {
    bytes: Bytes,
    trailers: Option<HeaderMap>,
}

impl CollectedBody {
    /// 创建收集结果。
    pub(crate) fn new(bytes: Bytes, trailers: Option<HeaderMap>) -> Self {
        Self { bytes, trailers }
    }

    /// 返回完整数据字节。
    #[must_use]
    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    /// 消耗对象并返回完整数据字节。
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }

    /// 返回可选 Trailer。
    #[must_use]
    pub fn trailers(&self) -> Option<&HeaderMap> {
        self.trailers.as_ref()
    }

    /// 将数据与 Trailer 拆分。
    #[must_use]
    pub fn into_parts(self) -> (Bytes, Option<HeaderMap>) {
        (self.bytes, self.trailers)
    }
}
