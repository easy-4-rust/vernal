//! 值样式契约。
//!
//! 对标 Spring `org.springframework.core.style.ValueStyler`。

/// 值样式契约。
///
/// 对应 Java: org.springframework.core.style.ValueStyler
///
/// Spring 语义：把任意值格式化为诊断文本（对标 `ToStringCreator` 的
/// 值格式化钩子）。
pub trait ValueStyler: Send + Sync {
    /// 样式化值。
    ///
    /// 对应 Java: `ValueStyler#style(Object)`
    fn style(&self, value: &dyn std::fmt::Debug) -> String;
}

/// 便捷：对实现了 `Debug` 的具体值做样式化。
pub fn style_value<T: std::fmt::Debug>(styler: &dyn ValueStyler, value: &T) -> String {
    styler.style(&value as &dyn std::fmt::Debug)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::DefaultValueStyler;

    #[test]
    fn default_styler_satisfies_contract() {
        // D 类（重构安全）：`DefaultValueStyler` 实现该契约
        fn assert_styler<T: ValueStyler>() {}
        assert_styler::<DefaultValueStyler>();
    }
}
