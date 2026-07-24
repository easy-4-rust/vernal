//! Registry 诊断测试使用的消费组件。

use std::sync::Arc;

use super::{Database, Greeting};

/// 同时依赖具体组件与 Trait Object 的测试消费方。
pub struct GreetingConsumer {
    pub _database: Arc<Database>,
    pub greeting: Arc<dyn Greeting>,
}
