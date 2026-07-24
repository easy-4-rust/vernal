//! Registry 诊断测试使用的英文问候实现。

use super::Greeting;

/// `Greeting` 端口的测试实现。
pub struct EnglishGreeting;

impl Greeting for EnglishGreeting {
    fn greet(&self) -> &'static str {
        "hello"
    }
}
