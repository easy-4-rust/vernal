//! Gotham 请求 Scope `StateData` 对象。

use std::sync::Arc;

use gotham::state::StateData;
use vernal_web::WebRequestScope;

/// 在 Gotham `State` 中保存当前 `WebRequestScope`。
#[derive(Clone)]
pub struct VernalGothamRequestScope(pub Arc<WebRequestScope>);

impl StateData for VernalGothamRequestScope {}
