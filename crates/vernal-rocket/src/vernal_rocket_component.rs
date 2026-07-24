//! Rocket `IoC` 组件 Request Guard 对象。

use std::{any::Any, ops::Deref, sync::Arc};

use rocket::{
    Request,
    http::Status,
    request::{FromRequest, Outcome},
};
use vernal_context::ApplicationContext;

use crate::RocketRejection;

/// 从 Rocket Managed `ApplicationContext` 解析类型化组件。
pub struct VernalRocketComponent<T>(pub Arc<T>);

impl<T> Deref for VernalRocketComponent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r, T> FromRequest<'r> for VernalRocketComponent<T>
where
    T: Any + Send + Sync,
{
    type Error = RocketRejection;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(context) = request.rocket().state::<Arc<ApplicationContext>>() else {
            return Outcome::Error((Status::InternalServerError, RocketRejection::MissingContext));
        };
        match context.container().resolve::<T>() {
            Ok(component) => Outcome::Success(Self(component)),
            Err(error) => Outcome::Error((
                Status::InternalServerError,
                RocketRejection::component_resolution(error),
            )),
        }
    }
}
