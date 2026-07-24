//! Trait 组件宏测试对象集合。

mod all_greeting_consumer;
mod chinese_greeting;
mod english_greeting;
mod greeting;
mod named_greeting_consumer;
mod primary_greeting_consumer;

pub use all_greeting_consumer::AllGreetingConsumer;
pub use chinese_greeting::ChineseGreeting;
pub use english_greeting::EnglishGreeting;
pub use greeting::Greeting;
pub use named_greeting_consumer::NamedGreetingConsumer;
pub use primary_greeting_consumer::PrimaryGreetingConsumer;
