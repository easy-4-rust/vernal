//! Gotham 请求上下文 `StateData` 对象。

use std::sync::Arc;

use gotham::state::StateData;
use vernal_web::RequestContext;

/// 在 Gotham `State` 中保存严格 AOP 创建的请求上下文。
#[derive(Clone)]
pub struct VernalGothamRequestContext(pub Arc<RequestContext>);

impl StateData for VernalGothamRequestContext {}
