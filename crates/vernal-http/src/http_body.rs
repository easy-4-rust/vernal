//! 有限与流式 HTTP Body 对象。

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use http_body::{Body, SizeHint};
use http_body_util::BodyExt;
use tokio_util::sync::CancellationToken;

use crate::{CollectedBody, HttpBodyError};

pub use http_body::Frame;

type FrameStream =
    Pin<Box<dyn Stream<Item = Result<Frame<Bytes>, HttpBodyError>> + Send + 'static>>;
type CancellationFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// 统一表示空、有限和流式 HTTP Body。
///
/// 流式变体直接转发 `http_body::Frame`，因此数据帧、Trailer、上游唤醒和背压
/// 都由原始 Stream 保持。只有调用 [`HttpBody::collect_limited`] 时才会显式缓冲。
pub enum HttpBody {
    /// 不包含任何帧。
    Empty {
        /// 可选取消等待 Future。
        cancellation: Option<CancellationFuture>,
        /// 是否已经被轮询到结束。
        completed: bool,
    },
    /// 只产生一个数据帧。
    Full {
        /// 尚未发送的数据。
        bytes: Option<Bytes>,
        /// 可选取消等待 Future。
        cancellation: Option<CancellationFuture>,
    },
    /// 按上游背压逐帧产生数据或 Trailer。
    Stream {
        /// 上游 Frame Stream。
        frames: FrameStream,
        /// 可选取消等待 Future。
        cancellation: Option<CancellationFuture>,
        /// 上游是否已经结束。
        completed: bool,
    },
}

impl HttpBody {
    /// 创建空 Body。
    #[must_use]
    pub fn empty() -> Self {
        Self::Empty {
            cancellation: None,
            completed: false,
        }
    }

    /// 创建单数据帧 Body。
    #[must_use]
    pub fn full(bytes: impl Into<Bytes>) -> Self {
        Self::Full {
            bytes: Some(bytes.into()),
            cancellation: None,
        }
    }

    /// 从产生标准 HTTP Frame 的 Stream 创建流式 Body。
    #[must_use]
    pub fn from_stream<S>(frames: S) -> Self
    where
        S: Stream<Item = Result<Frame<Bytes>, HttpBodyError>> + Send + 'static,
    {
        Self::Stream {
            frames: Box::pin(frames),
            cancellation: None,
            completed: false,
        }
    }

    /// 绑定取消令牌。
    ///
    /// 取消 Future 会注册当前 Body task 的 Waker，即使上游 Stream 没有新帧，
    /// 取消也能立即唤醒 `poll_frame`。
    #[must_use]
    pub fn with_cancellation(mut self, cancellation: CancellationToken) -> Self {
        let future: CancellationFuture = Box::pin(cancellation.cancelled_owned());
        match &mut self {
            Self::Empty { cancellation, .. }
            | Self::Full { cancellation, .. }
            | Self::Stream { cancellation, .. } => *cancellation = Some(future),
        }
        self
    }

    /// 在明确的字节上限内收集全部数据帧，同时保留 Trailer。
    ///
    /// # Errors
    ///
    /// Body 被取消、上游传输失败或累计数据超过 `limit` 时返回
    /// [`HttpBodyError`]。
    pub async fn collect_limited(mut self, limit: usize) -> Result<CollectedBody, HttpBodyError> {
        let mut bytes = BytesMut::new();
        let mut trailers = None;

        while let Some(frame) = self.frame().await {
            let frame = frame?;
            match frame.into_data() {
                Ok(data) => {
                    let observed = bytes.len().saturating_add(data.len());
                    if observed > limit {
                        return Err(HttpBodyError::LimitExceeded { limit, observed });
                    }
                    bytes.extend_from_slice(&data);
                }
                Err(frame) => {
                    if let Ok(frame_trailers) = frame.into_trailers() {
                        trailers
                            .get_or_insert_with(http::HeaderMap::new)
                            .extend(frame_trailers);
                    }
                }
            }
        }

        Ok(CollectedBody::new(bytes.freeze(), trailers))
    }

    /// 轮询并消费取消信号。
    fn poll_cancellation(&mut self, context: &mut Context<'_>) -> bool {
        let cancellation = match self {
            Self::Empty { cancellation, .. }
            | Self::Full { cancellation, .. }
            | Self::Stream { cancellation, .. } => cancellation,
        };
        cancellation
            .as_mut()
            .is_some_and(|future| future.as_mut().poll(context).is_ready())
    }
}

impl Body for HttpBody {
    type Data = Bytes;
    type Error = HttpBodyError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let body = self.as_mut().get_mut();
        if body.poll_cancellation(context) {
            *body = Self::Empty {
                cancellation: None,
                completed: true,
            };
            return Poll::Ready(Some(Err(HttpBodyError::Cancelled)));
        }

        match body {
            Self::Empty { completed, .. } => {
                *completed = true;
                Poll::Ready(None)
            }
            Self::Full { bytes, .. } => {
                Poll::Ready(bytes.take().map(|bytes| Ok(Frame::data(bytes))))
            }
            Self::Stream {
                frames, completed, ..
            } => match frames.as_mut().poll_next(context) {
                Poll::Ready(None) => {
                    *completed = true;
                    Poll::Ready(None)
                }
                frame => frame,
            },
        }
    }

    fn is_end_stream(&self) -> bool {
        match self {
            Self::Empty { completed, .. } | Self::Stream { completed, .. } => *completed,
            Self::Full { bytes, .. } => bytes.is_none(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Empty { .. } => SizeHint::with_exact(0),
            Self::Full { bytes, .. } => {
                SizeHint::with_exact(bytes.as_ref().map_or(0, |bytes| bytes.len() as u64))
            }
            Self::Stream { .. } => SizeHint::default(),
        }
    }
}

impl Default for HttpBody {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<Bytes> for HttpBody {
    fn from(bytes: Bytes) -> Self {
        Self::full(bytes)
    }
}

impl From<Vec<u8>> for HttpBody {
    fn from(bytes: Vec<u8>) -> Self {
        Self::full(bytes)
    }
}

impl From<String> for HttpBody {
    fn from(value: String) -> Self {
        Self::full(value)
    }
}

impl From<&'static str> for HttpBody {
    fn from(value: &'static str) -> Self {
        Self::full(value)
    }
}
