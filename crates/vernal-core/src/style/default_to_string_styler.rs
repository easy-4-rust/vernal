//! 默认字符串化样式器。
//!
//! 对标 Spring `org.springframework.core.style.DefaultToStringStyler`。

use std::fmt::Write;

use super::ToStringStyler;

/// 默认字符串化样式器。
///
/// 对应 Java: org.springframework.core.style.DefaultToStringStyler
///
/// Spring 语义：`ClassName [field1=value1, field2=value2]` 风格。
pub struct DefaultToStringStyler;

impl ToStringStyler for DefaultToStringStyler {
    fn style_start(&self, buffer: &mut String, object: &str) {
        let _ = write!(buffer, "{object} [");
    }

    fn style_field(&self, buffer: &mut String, field_name: &str, field_value: &str) {
        let _ = write!(buffer, " {field_name}={field_value}");
    }

    fn style_value(&self, buffer: &mut String, value: &str) {
        buffer.push_str(value);
    }

    fn style_end(&self, buffer: &mut String, _object: &str) {
        buffer.push_str(" ]");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_default_style() {
        // A 类（合同对齐）：对标 Spring 默认风格
        let styler = DefaultToStringStyler;
        let mut buffer = String::new();
        styler.style_start(&mut buffer, "Point");
        styler.style_field(&mut buffer, "x", "1");
        styler.style_field(&mut buffer, "y", "2");
        styler.style_end(&mut buffer, "Point");
        assert_eq!(buffer, "Point [ x=1 y=2 ]");
    }
}
