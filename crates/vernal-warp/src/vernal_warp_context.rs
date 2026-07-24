//! Warp 应用上下文 Filter 对象。

use std::sync::Arc;

use vernal_context::ApplicationContext;
use warp::{Filter, Rejection};

use crate::WarpRejection;

/// 从 Warp Request Extensions 提取当前 `ApplicationContext`。
#[derive(Clone)]
pub struct VernalWarpContext(pub Arc<ApplicationContext>);

impl VernalWarpContext {
    /// 创建应用上下文 Filter。
    ///
    /// Filter 从 Tower Service 写入的 Request Extension 读取 Context，不创建
    /// 第二个容器，也不访问进程全局 Service Locator。
    #[must_use]
    pub fn filter() -> impl Filter<Extract = (Self,), Error = Rejection> + Clone {
        warp::ext::optional::<Arc<ApplicationContext>>().and_then(
            |context: Option<Arc<ApplicationContext>>| async move {
                context
                    .map(Self)
                    .ok_or_else(|| warp::reject::custom(WarpRejection::MissingContext))
            },
        )
    }
}
