//! Rocket 请求本地生命周期状态对象。

use std::sync::{Arc, Mutex};

use tokio_util::sync::{CancellationToken, DropGuard};
use vernal_web::WebRequestScope;

/// 在 Request Fairing 与 Response Fairing 之间安全移交 Scope 所有权。
pub(crate) struct RocketRequestState {
    scope: Option<Arc<WebRequestScope>>,
    drop_guard: Mutex<Option<DropGuard>>,
}

impl RocketRequestState {
    /// 创建已安装 Fairing 的请求状态。
    pub(crate) fn installed(scope: Arc<WebRequestScope>, drop_guard: DropGuard) -> Self {
        Self {
            scope: Some(scope),
            drop_guard: Mutex::new(Some(drop_guard)),
        }
    }

    /// 创建用于识别“未安装 Fairing”的缺失状态。
    pub(crate) fn missing() -> Self {
        Self {
            scope: None,
            drop_guard: Mutex::new(None),
        }
    }

    /// 读取当前 Scope。
    pub(crate) fn scope(&self) -> Option<Arc<WebRequestScope>> {
        self.scope.clone()
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
