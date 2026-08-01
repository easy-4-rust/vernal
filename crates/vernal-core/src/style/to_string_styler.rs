//! 字符串化样式契约。
//!
//! 对标 Spring `org.springframework.core.style.ToStringStyler`。

use std::fmt::Write;

/// 字符串化样式契约。
///
/// 对应 Java: org.springframework.core.style.ToStringStyler
///
/// Spring 语义：控制 `ToStringCreator` 的片段拼接风格（开始/字段/值/结束）。
pub trait ToStringStyler: Send + Sync {
    /// 输出起始片段（如 `ClassName [`）。
    fn style_start(&self, buffer: &mut String, object: &str);

    /// 输出字段片段。
    fn style_field(&self, buffer: &mut String, field_name: &str, field_value: &str);

    /// 输出值片段。
    fn style_value(&self, buffer: &mut String, value: &str);

    /// 输出结束片段（如 `]`）。
    fn style_end(&self, buffer: &mut String, object: &str);
}

/// 便捷：写入字段对（对标 Spring 内部 `StringBuilder` 拼接）。
pub fn append_field(buffer: &mut String, name: &str, value: &str) {
    let _ = write!(buffer, "{name}={value}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::DefaultToStringStyler;

    #[test]
    fn default_styler_satisfies_contract() {
        // D 类（重构安全）：`DefaultToStringStyler` 实现该契约
        fn assert_styler<T: ToStringStyler>() {}
        assert_styler::<DefaultToStringStyler>();
    }
}
