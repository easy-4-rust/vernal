//! Trait Binding 合同测试使用的最小对象集合。

mod chinese_greeting;
mod english_greeting;
mod greeting;
mod greeting_consumer;

pub use chinese_greeting::ChineseGreeting;
pub use english_greeting::EnglishGreeting;
pub use greeting::Greeting;
pub use greeting_consumer::GreetingConsumer;
