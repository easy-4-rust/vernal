//! Tokio 字节读取错误测试对象。

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncRead, ReadBuf};

/// 在第一次 Tokio 读取时返回确定性错误的测试 Reader。
///
/// Rocket 的公开响应 Body 以 Tokio `AsyncRead` 暴露字节流，该对象让测试无需
/// 网络连接即可触发真实 Reader 错误路径。
#[derive(Default)]
pub struct FailingTokioReader {
    emitted: bool,
}

impl FailingTokioReader {
    /// 创建尚未发出错误的 Reader。
    #[must_use]
    pub const fn new() -> Self {
        Self { emitted: false }
    }

    /// 返回合同测试使用的稳定错误文本。
    #[must_use]
    pub const fn error_message() -> &'static str {
        "synthetic upstream Tokio reader failure"
    }
}

impl AsyncRead for FailingTokioReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        _buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.emitted {
            return Poll::Ready(Ok(()));
        }
        self.emitted = true;
        Poll::Ready(Err(io::Error::other(Self::error_message())))
    }
}
