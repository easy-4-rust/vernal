//! Rocket 严格 AOP 请求上下文 Request Guard 对象。

use std::sync::Arc;

use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};
use vernal_web::RequestContext;

use crate::{RocketRejection, rocket_request_state::RocketRequestState};

/// 从请求本地状态读取当前匹配 Route 的类型化请求上下文。
pub struct VernalRocketRequestContext(pub Arc<RequestContext>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VernalRocketRequestContext {
    type Error = RocketRejection;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let state = request.local_cache(RocketRequestState::missing);
        match state.request_context() {
            Some(context) => Outcome::Success(Self(context)),
            None => Outcome::Error((
                Status::InternalServerError,
                RocketRejection::MissingRequestContext,
            )),
        }
    }
}
