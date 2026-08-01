//! 简单值样式器。
//!
//! 对标 Spring `org.springframework.core.style.SimpleValueStyler`。

use super::ValueStyler;

/// 简单值样式器。
///
/// 对应 Java: org.springframework.core.style.SimpleValueStyler
///
/// Spring 语义：最简单形态——直接输出 `toString()`（Rust 中为 `Debug`）。
pub struct SimpleValueStyler;

impl ValueStyler for SimpleValueStyler {
    fn style(&self, value: &dyn std::fmt::Debug) -> String {
        format!("{value:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outputs_debug_directly() {
        // A 类（合同对齐）：对标 Spring toString 直出
        let styler = SimpleValueStyler;
        assert_eq!(styler.style(&"x"), "\"x\"");
        assert_eq!(styler.style(&7_u8), "7");
    }
}
