//! 默认值样式器。
//!
//! 对标 Spring `org.springframework.core.style.DefaultValueStyler`。

use super::ValueStyler;

/// 默认值样式器。
///
/// 对应 Java: org.springframework.core.style.DefaultValueStyler
///
/// Spring 语义：字符串加引号、数字/布尔原样、`null` → `<null>`、数组/集合
/// 展开为 `[a, b]`（对标 Spring 的默认值格式化规则）。
pub struct DefaultValueStyler;

impl ValueStyler for DefaultValueStyler {
    fn style(&self, value: &dyn std::fmt::Debug) -> String {
        let text = format!("{value:?}");
        if text == "()" {
            return "<null>".to_string();
        }
        // Debug 输出已是 Rust 风格；对标 Spring 的字符串引号规则
        if text.starts_with('"') {
            return text;
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_strings_with_quotes() {
        // A 类（合同对齐）：对标 Spring 字符串引号
        let styler = DefaultValueStyler;
        assert_eq!(styler.style(&"hello"), "\"hello\"");
    }

    #[test]
    fn styles_numbers_plain() {
        // B 类（边界行为）：数字原样
        let styler = DefaultValueStyler;
        assert_eq!(styler.style(&42_i32), "42");
        assert_eq!(styler.style(&3.5_f64), "3.5");
    }

    #[test]
    fn styles_booleans() {
        let styler = DefaultValueStyler;
        assert_eq!(styler.style(&true), "true");
    }
}
