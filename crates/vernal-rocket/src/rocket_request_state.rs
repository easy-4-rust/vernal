//! Rocket 请求本地生命周期状态对象。

use std::sync::{Arc, Mutex};

use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::{RequestContext, WebRequestScope};

/// 在 Request Fairing 与 Response Fairing 之间安全移交 Scope 所有权。
pub(crate) struct RocketRequestState {
    scope: Option<Arc<WebRequestScope>>,
    drop_guard: Mutex<Option<DropGuard>>,
    request_context: Mutex<Option<Arc<RequestContext>>>,
}

impl RocketRequestState {
    /// 创建已安装 Fairing 的请求状态。
    pub(crate) fn installed(scope: Arc<WebRequestScope>, drop_guard: DropGuard) -> Self {
        Self {
            scope: Some(scope),
            drop_guard: Mutex::new(Some(drop_guard)),
            request_context: Mutex::new(None),
        }
    }

    /// 创建用于识别“未安装 Fairing”的缺失状态。
    pub(crate) fn missing() -> Self {
        Self {
            scope: None,
            drop_guard: Mutex::new(None),
            request_context: Mutex::new(None),
        }
    }

    /// 读取当前 Scope。
    pub(crate) fn scope(&self) -> Option<Arc<WebRequestScope>> {
        self.scope.clone()
    }

    /// 写入当前匹配 Route 的请求上下文。
    ///
    /// Rocket 允许 Handler 返回 Forward 后尝试下一条 Route，因此同一请求的后续
    /// 包装 Handler 可以用新的低基数路由身份替换旧上下文。
    pub(crate) fn set_request_context(&self, context: Arc<RequestContext>) {
        *self
            .request_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(context);
    }

    /// 读取当前 Route 的请求上下文。
    pub(crate) fn request_context(&self) -> Option<Arc<RequestContext>> {
        self.request_context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// 把 Drop 兜底转换为响应 Body 持有的取消令牌。
    pub(crate) fn take_response_parts(&self) -> Option<(Arc<WebRequestScope>, CancellationToken)> {
        let scope = self.scope()?;
        let drop_guard = self
            .drop_guard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()?;
        Some((scope, drop_guard.disarm()))
    }
}
