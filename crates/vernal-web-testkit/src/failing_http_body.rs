//! 标准 `http-body` 流错误测试对象。

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body::{Body, Frame};

/// 在第一次 Frame 轮询时返回确定性上游错误的测试 Body。
///
/// 本对象只制造传输失败，不持有也不关闭 Vernal 请求 Scope。Adapter 合同测试
/// 借此证明真正的响应包装器会在向调用方恢复错误前完成异步 Scope 清理。
#[derive(Default)]
pub struct FailingHttpBody {
    emitted: bool,
}

impl FailingHttpBody {
    /// 创建尚未发出错误的 Body。
    #[must_use]
    pub const fn new() -> Self {
        Self { emitted: false }
    }

    /// 返回合同测试使用的稳定错误文本。
    #[must_use]
    pub const fn error_message() -> &'static str {
        "synthetic upstream HTTP body failure"
    }
}

impl Body for FailingHttpBody {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        if self.emitted {
            return Poll::Ready(None);
        }
        self.emitted = true;
        Poll::Ready(Some(Err(io::Error::other(Self::error_message()))))
    }
}
