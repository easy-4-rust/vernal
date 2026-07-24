//! 框架中立 Web 失败对象。

use std::{error::Error, fmt, sync::Arc};

use vernal_core::SharedError;

use crate::ProblemDetails;

/// 同时携带脱敏协议问题与服务端原始错误链的失败边界。
///
/// 安全、限流、事务等 AOP 拦截器可把该对象包装进
/// [`vernal_aop::InvocationError`]。Axum、Tonic 等 Adapter 只读取
/// [`ProblemDetails`] 生成客户端响应，日志与追踪系统仍可沿 [`Error::source`]
/// 获取原始错误；两条信息通道不会互相泄漏。
#[derive(Clone, Debug)]
pub struct WebFailure {
    problem: ProblemDetails,
    source: Option<SharedError>,
}

impl WebFailure {
    /// 创建没有内部错误源的协议失败。
    #[must_use]
    pub const fn new(problem: ProblemDetails) -> Self {
        Self {
            problem,
            source: None,
        }
    }

    /// 创建同时保留服务端错误链的协议失败。
    #[must_use]
    pub fn with_source<E>(problem: ProblemDetails, source: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        Self {
            problem,
            source: Some(Arc::new(source)),
        }
    }

    /// 返回可安全映射到 HTTP/gRPC 响应的问题描述。
    #[must_use]
    pub const fn problem(&self) -> &ProblemDetails {
        &self.problem
    }
}

impl fmt::Display for WebFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            self.problem
                .detail()
                .unwrap_or_else(|| self.problem.title()),
        )
    }
}

impl Error for WebFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

impl From<ProblemDetails> for WebFailure {
    fn from(problem: ProblemDetails) -> Self {
        Self::new(problem)
    }
}
