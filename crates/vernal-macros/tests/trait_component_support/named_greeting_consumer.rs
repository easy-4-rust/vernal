//! 命名 Trait 实现消费者。

use std::sync::Arc;

use crate::trait_component_support::Greeting;

/// 验证字段 qualifier 自动生成精确命名 Trait 依赖。
#[derive(vernal_macros::Component)]
pub struct NamedGreetingConsumer {
    /// 明确选择英文绑定。
    #[component(qualifier = "english")]
    pub greeting: Arc<dyn Greeting>,
}
