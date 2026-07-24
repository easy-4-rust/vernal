//! Primary Trait 实现消费者。

use std::sync::Arc;

use crate::trait_component_support::Greeting;

/// 验证 `Arc<dyn Trait>` 自动生成单值绑定依赖。
#[derive(vernal_macros::Component)]
pub struct PrimaryGreetingConsumer {
    /// 由唯一 Primary 规则选择的服务。
    pub greeting: Arc<dyn Greeting>,
}
