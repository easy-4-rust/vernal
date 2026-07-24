//! 全部 Trait 实现消费者。

use std::sync::Arc;

use crate::trait_component_support::Greeting;

/// 验证 `Vec<Arc<dyn Trait>>` 自动生成多实现依赖。
#[derive(vernal_macros::Component)]
pub struct AllGreetingConsumer {
    /// 按绑定注册顺序注入的全部服务。
    pub greetings: Vec<Arc<dyn Greeting>>,
}
