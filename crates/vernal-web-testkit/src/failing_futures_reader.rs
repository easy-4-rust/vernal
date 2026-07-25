//! Futures IO 字节读取错误测试对象。

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::io::{AsyncBufRead, AsyncRead};

/// 在第一次 Futures IO 读取时返回确定性错误的测试 Reader。
///
/// Tide 使用 `futures_lite::io::AsyncRead` 而不是 Tokio Reader；独立对象避免
/// 测试通过兼容层改变 Tide 原生的读取与错误语义。
#[derive(Default)]
pub struct FailingFuturesReader {
    emitted: bool,
}

impl FailingFuturesReader {
    /// 创建尚未发出错误的 Reader。
    #[must_use]
    pub const fn new() -> Self {
        Self { emitted: false }
    }

    /// 返回合同测试使用的稳定错误文本。
    #[must_use]
    pub const fn error_message() -> &'static str {
        "synthetic upstream Futures reader failure"
    }
}

impl AsyncRead for FailingFuturesReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        _buffer: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.emitted {
            return Poll::Ready(Ok(0));
        }
        self.emitted = true;
        Poll::Ready(Err(io::Error::other(Self::error_message())))
    }
}

impl AsyncBufRead for FailingFuturesReader {
    fn poll_fill_buf(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
    ) -> Poll<io::Result<&[u8]>> {
        if self.emitted {
            return Poll::Ready(Ok(&[]));
        }
        self.emitted = true;
        Poll::Ready(Err(io::Error::other(Self::error_message())))
    }

    fn consume(self: Pin<&mut Self>, _amount: usize) {}
}
