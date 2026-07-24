//! 英文问候组件。

use crate::trait_component_support::Greeting;

/// 通过 Component 宏定义的英文实现。
#[derive(vernal_macros::Component)]
pub struct EnglishGreeting;

impl Greeting for EnglishGreeting {
    fn message(&self) -> &'static str {
        "hello"
    }
}
