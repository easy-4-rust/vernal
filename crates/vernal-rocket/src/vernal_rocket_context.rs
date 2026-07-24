//! Rocket 应用上下文 Request Guard 对象。

use std::sync::Arc;

use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};
use vernal_context::ApplicationContext;

use crate::RocketRejection;

/// 从 Rocket Managed State 提取当前 `ApplicationContext`。
pub struct VernalRocketContext(pub Arc<ApplicationContext>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VernalRocketContext {
    type Error = RocketRejection;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request
            .rocket()
            .state::<Arc<ApplicationContext>>()
            .cloned()
            .map(Self)
            .map_or_else(
                || Outcome::Error((Status::InternalServerError, RocketRejection::MissingContext)),
                Outcome::Success,
            )
    }
}
