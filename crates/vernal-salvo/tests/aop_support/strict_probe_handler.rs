//! 测试用 Salvo 严格 AOP 目标 Handler 对象。

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use salvo::{
    Depot, FlowCtrl, Handler, Request, Response,
    http::{Method, StatusCode},
    writing::Text,
};
use vernal_http::HttpRequestSnapshot;
use vernal_salvo::VernalSalvoDepotExt;

/// 验证类型化 RequestContext、owned 快照与原生响应保持。
pub struct StrictProbeHandler {
    called: Arc<AtomicBool>,
    verify_snapshot: bool,
    status: StatusCode,
    body: &'static str,
}

impl StrictProbeHandler {
    /// 创建严格 AOP 测试 Handler。
    pub const fn new(
        called: Arc<AtomicBool>,
        verify_snapshot: bool,
        status: StatusCode,
        body: &'static str,
    ) -> Self {
        Self {
            called,
            verify_snapshot,
            status,
            body,
        }
    }
}

#[salvo::async_trait]
impl Handler for StrictProbeHandler {
    async fn handle(
        &self,
        _request: &mut Request,
        depot: &mut Depot,
        response: &mut Response,
        _control: &mut FlowCtrl,
    ) {
        self.called.store(true, Ordering::SeqCst);
        if self.verify_snapshot {
            let context = depot
                .vernal_request_context()
                .expect("strict request context");
            let snapshot = context
                .extensions()
                .get::<HttpRequestSnapshot>()
                .await
                .expect("owned HTTP snapshot");

            assert_eq!(context.route().handler(), "/orders/{id}");
            assert_eq!(context.route().operation_name(), "GET");
            assert_eq!(context.route().path_template(), "/orders/{id}");
            assert_eq!(snapshot.method(), Method::GET);
            assert_eq!(snapshot.uri().path(), "/orders/42");
        }
        response.status_code = Some(self.status);
        response.render(Text::Plain(self.body));
    }
}
