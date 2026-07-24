//! Salvo 请求生命周期 Hoop 对象。

use std::{pin::Pin, sync::Arc};

use salvo::{Depot, FlowCtrl, Handler, Request, Response, http::ResBody};
use tokio_util::sync::CancellationToken;
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::SalvoScopedBody;

/// 向 Salvo `Depot` 注入 Context，并让请求 Scope 跟随响应 Body。
#[derive(Clone)]
pub struct VernalSalvoHoop {
    context: Arc<ApplicationContext>,
}

impl VernalSalvoHoop {
    /// 创建 Salvo Hoop。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

#[salvo::async_trait]
impl Handler for VernalSalvoHoop {
    async fn handle(
        &self,
        request: &mut Request,
        depot: &mut Depot,
        response: &mut Response,
        control: &mut FlowCtrl,
    ) {
        depot.inject(Arc::clone(&self.context));
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        depot.inject(Arc::clone(&scope));

        // Handler Future 或响应 Body 被丢弃时，DropGuard 只负责同步发出取消信号；
        // 预先启动的 Tokio 任务负责执行真正的异步 Scope 关闭。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.drop_guard();
        control.call_next(request, depot, response).await;

        // Salvo Handler 不用 Result 表达下游错误；无论状态码为何，都让 Scope
        // 跟随原生 ResBody 的 Frame 生命周期，而不是在 Handler 返回时提前关闭。
        let cancellation = request_guard.disarm();
        let body = response.take_body();
        response.body(ResBody::Boxed(Pin::from(Box::new(SalvoScopedBody::new(
            body,
            scope,
            cancellation,
        )))));
    }
}
