//! 字节流错误测试对象。

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_core::Stream;

/// 在第一次轮询时返回确定性 `io::Error` 的字节流。
///
/// Actix Web、Ntex 和 Poem 等框架可把该对象转换为自己的原生响应 Body，从而
/// 使用同一个失败事实验证各自的请求 Scope 清理边界。
#[derive(Default)]
pub struct FailingByteStream {
    emitted: bool,
}

impl FailingByteStream {
    /// 创建尚未发出错误的字节流。
    #[must_use]
    pub const fn new() -> Self {
        Self { emitted: false }
    }

    /// 返回合同测试使用的稳定错误文本。
    #[must_use]
    pub const fn error_message() -> &'static str {
        "synthetic upstream byte stream failure"
    }
}

impl Stream for FailingByteStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.emitted {
            return Poll::Ready(None);
        }
        self.emitted = true;
        Poll::Ready(Some(Err(io::Error::other(Self::error_message()))))
    }
}
