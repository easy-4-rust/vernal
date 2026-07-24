//! Web 请求上下文对象。

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use vernal_aop::InvocationContext;

use crate::{RequestId, RouteMetadata, SecurityPrincipal};

/// 框架中立、可跨 `.await` 共享的单请求上下文。
///
/// Adapter 把 owned request snapshot 放入 extensions；安全 Bridge 在异步认证后
/// 写入 principal。上下文只属于当前请求，不通过 task-local 或全局变量猜测。
pub struct RequestContext {
    id: RequestId,
    route: RouteMetadata,
    principal: RwLock<Option<Arc<SecurityPrincipal>>>,
    extensions: InvocationContext,
    cancellation: CancellationToken,
    deadline: Option<Instant>,
}

impl RequestContext {
    /// 创建请求上下文。
    #[must_use]
    pub fn new(route: RouteMetadata, cancellation: CancellationToken) -> Self {
        Self {
            id: RequestId::next(),
            route,
            principal: RwLock::new(None),
            extensions: InvocationContext::new(),
            cancellation,
            deadline: None,
        }
    }

    /// 设置请求绝对截止时间。
    #[must_use]
    pub fn with_deadline(mut self, deadline: Instant) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// 写入或清除安全主体。
    pub async fn set_principal(&self, principal: Option<Arc<SecurityPrincipal>>) {
        *self.principal.write().await = principal;
    }

    /// 返回当前安全主体快照。
    pub async fn principal(&self) -> Option<Arc<SecurityPrincipal>> {
        self.principal.read().await.clone()
    }

    /// 返回请求标识。
    #[must_use]
    pub const fn id(&self) -> RequestId {
        self.id
    }

    /// 返回路由元数据。
    #[must_use]
    pub const fn route(&self) -> &RouteMetadata {
        &self.route
    }

    /// 返回类型化扩展上下文。
    #[must_use]
    pub const fn extensions(&self) -> &InvocationContext {
        &self.extensions
    }

    /// 返回请求取消令牌。
    #[must_use]
    pub const fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }

    /// 返回可选绝对截止时间。
    #[must_use]
    pub const fn deadline(&self) -> Option<Instant> {
        self.deadline
    }
}
