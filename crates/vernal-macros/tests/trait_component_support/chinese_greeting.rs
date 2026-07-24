//! 中文问候组件。

use crate::trait_component_support::Greeting;

/// 通过 Component 宏定义的中文实现。
#[derive(vernal_macros::Component)]
pub struct ChineseGreeting;

impl Greeting for ChineseGreeting {
    fn message(&self) -> &'static str {
        "你好"
    }
}
