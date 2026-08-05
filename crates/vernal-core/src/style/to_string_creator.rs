//! 字符串化构建器。
//!
//! 对标 Spring `org.springframework.core.style.ToStringCreator`。

use std::fmt::Write;

use super::ToStringStyler;
use super::default_to_string_styler::DefaultToStringStyler;

/// 字符串化构建器。
///
/// 对应 Java: org.springframework.core.style.ToStringCreator
///
/// Spring 语义：链式 `append(field, value)` 构建 `ClassName [f=v, ...]`
/// 风格的 `toString` 输出。
pub struct ToStringCreator<'a> {
    buffer: String,
    styler: &'a dyn ToStringStyler,
    first_field: bool,
}

impl<'a> ToStringCreator<'a> {
    /// 以指定样式器开始构建对象字符串。
    #[must_use]
    pub fn with_styler(object: &str, styler: &'a dyn ToStringStyler) -> Self {
        let mut creator = Self {
            buffer: String::default(),
            styler,
            first_field: true,
        };
        styler.style_start(&mut creator.buffer, object);
        creator
    }

    /// 使用默认样式器构建。
    #[must_use]
    pub fn new(object: &str) -> Self {
        Self::with_styler(object, &DefaultToStringStyler)
    }

    /// 追加字段（字段值按 `Debug` 样式化）。
    pub fn append<T: std::fmt::Debug>(&mut self, field_name: &str, value: &T) -> &mut Self {
        let value_text = format!("{value:?}");
        if !self.first_field {
            self.buffer.push(',');
        }
        self.first_field = false;
        self.styler
            .style_field(&mut self.buffer, field_name, &value_text);
        self
    }

    /// 完成构建并返回结果字符串。
    #[must_use]
    pub fn build(mut self) -> String {
        self.styler.style_end(&mut self.buffer, "");
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_to_string_with_fields() {
        // A 类（合同对齐）：对标 Spring `ToStringCreator` 输出
        let mut creator = ToStringCreator::new("Point");
        creator.append("x", &1_i32).append("y", &2_i32);
        let result = creator.build();
        assert!(result.starts_with("Point ["));
        assert!(result.contains("x=1"));
        assert!(result.contains("y=2"));
        assert!(result.ends_with(']'));
    }

    #[test]
    fn no_fields_yields_empty_brackets() {
        // B 类（边界行为）
        let creator = ToStringCreator::new("Empty");
        let result = creator.build();
        assert!(result.starts_with("Empty ["));
    }
}
