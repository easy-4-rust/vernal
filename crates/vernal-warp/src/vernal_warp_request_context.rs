//! Warp 请求上下文 Filter 对象。

use std::sync::Arc;

use vernal_web::RequestContext;
use warp::{Filter, Rejection};

use crate::WarpRejection;

/// 从 Warp Request Extensions 提取严格 AOP 创建的请求上下文。
#[derive(Clone)]
pub struct VernalWarpRequestContext(pub Arc<RequestContext>);

impl VernalWarpRequestContext {
    /// 创建请求上下文 Filter。
    ///
    /// 普通 [`crate::VernalWarpLayer`] 只建立应用 Context 与请求 Scope；只有
    /// [`crate::VernalWarpAopLayer`] 会在执行 Filter 前写入该对象。
    #[must_use]
    pub fn filter() -> impl Filter<Extract = (Self,), Error = Rejection> + Clone {
        warp::ext::optional::<Arc<RequestContext>>().and_then(
            |context: Option<Arc<RequestContext>>| async move {
                context
                    .map(Self)
                    .ok_or_else(|| warp::reject::custom(WarpRejection::MissingRequestContext))
            },
        )
    }
}
