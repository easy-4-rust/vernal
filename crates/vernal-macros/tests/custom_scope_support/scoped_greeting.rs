//! 派生宏测试用自定义作用域组件对象。

use std::sync::Arc;

use vernal_macros::Component;

use super::request_scope::RequestScope;

/// 由派生宏生成自定义作用域 Definition 的测试组件。
#[derive(Component)]
#[component(scope = RequestScope)]
pub struct ScopedGreeting {
    /// 从依赖图注入的共享文本。
    pub message: Arc<String>,
}
