//! 中文问候服务测试实现。

use crate::trait_support::Greeting;

/// 用于验证命名和 Primary 选择规则的中文实现。
pub struct ChineseGreeting;

impl Greeting for ChineseGreeting {
    fn message(&self) -> &'static str {
        "你好"
    }
}
