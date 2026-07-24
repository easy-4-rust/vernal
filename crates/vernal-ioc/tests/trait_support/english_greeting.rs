//! 英文问候服务测试实现。

use crate::trait_support::Greeting;

/// 用于验证默认 Trait Binding 与目标组件身份的英文实现。
pub struct EnglishGreeting;

impl Greeting for EnglishGreeting {
    fn message(&self) -> &'static str {
        "hello"
    }
}
