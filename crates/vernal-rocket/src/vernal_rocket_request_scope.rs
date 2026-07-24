//! Rocket 请求 Scope Request Guard 对象。

use std::sync::Arc;

use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};
use vernal_web::WebRequestScope;

use crate::{RocketRejection, rocket_request_state::RocketRequestState};

/// 从 Rocket Request Local Cache 提取当前 `WebRequestScope`。
pub struct VernalRocketRequestScope(pub Arc<WebRequestScope>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VernalRocketRequestScope {
    type Error = RocketRejection;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request
            .local_cache(RocketRequestState::missing)
            .scope()
            .map(Self)
            .map_or_else(
                || {
                    Outcome::Error((
                        Status::InternalServerError,
                        RocketRejection::MissingRequestScope,
                    ))
                },
                Outcome::Success,
            )
    }
}
