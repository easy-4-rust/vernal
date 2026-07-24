//! Gotham 应用上下文 `StateData` 对象。

use std::sync::Arc;

use gotham::state::StateData;
use vernal_context::ApplicationContext;

/// 在 Gotham `State` 中保存显式 `ApplicationContext`。
#[derive(Clone)]
pub struct VernalGothamContext(pub Arc<ApplicationContext>);

impl StateData for VernalGothamContext {}
