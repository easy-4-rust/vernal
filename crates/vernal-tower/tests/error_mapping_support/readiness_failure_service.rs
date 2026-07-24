//! 测试用 readiness 失败 Service 对象。

use std::{
    future::{Ready, ready},
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

use http::{Request, Response};
use tower::Service;

/// 在 `poll_ready` 返回错误，并记录 `call` 是否被错误推进的测试 Service。
#[derive(Clone)]
pub struct ReadinessFailureService {
    called: Arc<AtomicBool>,
}

impl ReadinessFailureService {
    /// 创建共享调用记录的失败 Service。
    pub fn new(called: Arc<AtomicBool>) -> Self {
        Self { called }
    }
}

impl Service<Request<()>> for ReadinessFailureService {
    type Response = Response<u8>;
    type Error = io::Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Err(io::Error::other("not ready")))
    }

    fn call(&mut self, _request: Request<()>) -> Self::Future {
        self.called.store(true, Ordering::SeqCst);
        ready(Ok(Response::new(1)))
    }
}
