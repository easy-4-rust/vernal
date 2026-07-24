//! Warp 请求 Scope Filter 对象。

use std::sync::Arc;

use vernal_web::WebRequestScope;
use warp::{Filter, Rejection};

use crate::WarpRejection;

/// 从 Warp Request Extensions 提取当前 `WebRequestScope`。
#[derive(Clone)]
pub struct VernalWarpRequestScope(pub Arc<WebRequestScope>);

impl VernalWarpRequestScope {
    /// 创建请求 Scope Filter。
    #[must_use]
    pub fn filter() -> impl Filter<Extract = (Self,), Error = Rejection> + Clone {
        warp::ext::optional::<Arc<WebRequestScope>>().and_then(
            |scope: Option<Arc<WebRequestScope>>| async move {
                scope
                    .map(Self)
                    .ok_or_else(|| warp::reject::custom(WarpRejection::MissingRequestScope))
            },
        )
    }
}
