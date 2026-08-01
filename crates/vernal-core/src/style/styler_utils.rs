//! 样式工具。
//!
//! 对标 Spring `org.springframework.core.style.StylerUtils`。

use super::default_value_styler::DefaultValueStyler;
use super::ValueStyler;

/// 样式工具（静态辅助函数）。
///
/// 对应 Java: org.springframework.core.style.StylerUtils
pub struct StylerUtils;

impl StylerUtils {
    /// 使用默认值样式器样式化值。
    ///
    /// 对应 Java: `StylerUtils#style(Object)`
    #[must_use]
    pub fn style<T: std::fmt::Debug>(value: &T) -> String {
        let styler = DefaultValueStyler;
        styler.style(&value as &dyn std::fmt::Debug)
    }

    /// 使用指定样式器样式化值。
    #[must_use]
    pub fn style_with<T: std::fmt::Debug>(styler: &dyn ValueStyler, value: &T) -> String {
        styler.style(&value as &dyn std::fmt::Debug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_with_default_styler() {
        // A 类（合同对齐）：对标 Spring 默认样式化
        assert_eq!(StylerUtils::style(&"hi"), "\"hi\"");
        assert_eq!(StylerUtils::style(&5_u32), "5");
    }

    #[test]
    fn styles_with_custom_styler() {
        // D 类（重构安全）：自定义样式器接入
        struct UpperStyler;
        impl ValueStyler for UpperStyler {
            fn style(&self, value: &dyn std::fmt::Debug) -> String {
                format!("{value:?}").to_uppercase()
            }
        }
        assert_eq!(StylerUtils::style_with(&UpperStyler, &"x"), "\"X\"");
    }
}
