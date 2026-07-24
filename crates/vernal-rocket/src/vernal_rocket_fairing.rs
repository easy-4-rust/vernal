//! Rocket Context 与请求 Scope 生命周期 Fairing 对象。

use std::sync::Arc;

use rocket::{
    Build, Data, Request, Response, Rocket,
    fairing::{Fairing, Info, Kind},
};
use vernal_context::ApplicationContext;
use vernal_web::WebRequestScope;

use crate::{RocketScopedReader, rocket_request_state::RocketRequestState};

/// 注册 Managed Context，并让每个 Rocket 请求 Scope 跟随响应 Body。
#[derive(Clone)]
pub struct VernalRocketFairing {
    context: Arc<ApplicationContext>,
}

impl VernalRocketFairing {
    /// 创建 Rocket Fairing。
    #[must_use]
    pub fn new(context: Arc<ApplicationContext>) -> Self {
        Self { context }
    }
}

#[rocket::async_trait]
impl Fairing for VernalRocketFairing {
    fn info(&self) -> Info {
        Info {
            name: "Vernal Application Context and Request Scope",
            kind: Kind::Ignite | Kind::Request | Kind::Response,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        if rocket.state::<Arc<ApplicationContext>>().is_some() {
            Ok(rocket)
        } else {
            Ok(rocket.manage(Arc::clone(&self.context)))
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        // Ignite 可能发现应用已经注册了 Context。请求阶段必须绑定 Rocket 最终
        // Managed State 中的权威实例，不能继续使用 Fairing 构造时传入但未安装的
        // 另一个 Arc，否则组件提取器与请求 Scope 会落到不同 Container。
        let Some(context) = request.rocket().state::<Arc<ApplicationContext>>().cloned() else {
            request.local_cache(RocketRequestState::missing);
            return;
        };
        let scope = Arc::new(WebRequestScope::from_application_context(context));
        let cancellation = scope.cancellation().clone();

        // 请求或响应 Body 被丢弃时，DropGuard 发出取消；后台 Tokio 任务完成
        // 真正的异步 Scope 关闭。
        let cleanup_scope = Arc::clone(&scope);
        let cleanup_cancellation = cancellation.clone();
        tokio::spawn(async move {
            cleanup_cancellation.cancelled().await;
            let _ = cleanup_scope.close().await;
        });

        request.local_cache(|| RocketRequestState::installed(scope, cancellation.drop_guard()));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let state = request.local_cache(RocketRequestState::missing);
        let Some((scope, cancellation)) = state.take_response_parts() else {
            return;
        };

        let max_chunk_size = response.body().max_chunk_size();
        let body = response.body_mut().take();
        response.set_streamed_body(RocketScopedReader::new(body, scope, cancellation));
        response.set_max_chunk_size(max_chunk_size);
    }
}
