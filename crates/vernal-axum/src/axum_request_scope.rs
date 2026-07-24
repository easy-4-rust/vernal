//! Axum 请求作用域中间件对象。

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tokio_util::sync::CancellationToken;
use vernal_tower::ScopedBody;
use vernal_web::WebRequestScope;

/// 在 Axum 原生 Middleware 边界创建并释放 `WebRequestScope`。
pub struct AxumRequestScope;

impl AxumRequestScope {
    /// 执行请求作用域中间件。
    ///
    /// `Next` 返回后仍不关闭 Scope；Scope 生命周期会延伸到响应 Body 完成、
    /// 出错或被客户端丢弃。请求 Future 被取消时，`DropGuard` 唤醒后台关闭任务。
    pub async fn handle(State(()): State<()>, mut request: Request, next: Next) -> Response {
        let cancellation = CancellationToken::new();
        let scope = Arc::new(WebRequestScope::new(cancellation.clone()));
        request.extensions_mut().insert(Arc::clone(&scope));

        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        let request_guard = cancellation.drop_guard();
        let response = next.run(request).await;
        let cancellation = request_guard.disarm();
        let (parts, body) = response.into_parts();
        let body = Body::new(ScopedBody::new(body, scope, cancellation));
        Response::from_parts(parts, body)
    }
}
