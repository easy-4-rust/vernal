//! 依赖问候服务 Trait 的测试组件。

use std::sync::Arc;

use crate::trait_support::Greeting;

/// 验证受限 Resolver 能注入 Trait Object 的业务消费者。
pub struct GreetingConsumer {
    /// 由唯一候选或 Primary 规则选出的服务。
    pub greeting: Arc<dyn Greeting>,
}
