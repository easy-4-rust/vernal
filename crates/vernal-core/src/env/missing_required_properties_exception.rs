//! 必填属性缺失异常。
//!
//! 对标 Spring `org.springframework.core.env.MissingRequiredPropertiesException`。

use std::fmt;

/// 必填属性缺失异常。
///
/// 对应 Java: org.springframework.core.env.MissingRequiredPropertiesException
///
/// Spring 语义：`validateRequiredProperties` 校验失败时抛出，列出全部缺失
/// 属性名。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingRequiredPropertiesException {
    missing_properties: Vec<String>,
}

impl MissingRequiredPropertiesException {
    /// 创建异常。
    #[must_use]
    pub fn new(missing_properties: Vec<String>) -> Self {
        Self { missing_properties }
    }

    /// 返回缺失属性列表。
    #[must_use]
    pub fn missing_properties(&self) -> &[String] {
        &self.missing_properties
    }

    /// 是否为空（无缺失）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.missing_properties.is_empty()
    }
}

impl fmt::Display for MissingRequiredPropertiesException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "以下必填属性缺失: {}",
            self.missing_properties.join(", ")
        )
    }
}

impl std::error::Error for MissingRequiredPropertiesException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_missing_properties() {
        // A 类（合同对齐）：对标 Spring 缺失属性消息
        let err = MissingRequiredPropertiesException::new(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(
            err.missing_properties(),
            &["a".to_string(), "b".to_string()]
        );
        assert!(err.to_string().contains('a'));
        assert!(err.to_string().contains('b'));
    }

    #[test]
    fn empty_is_not_missing() {
        // B 类（边界行为）
        let err = MissingRequiredPropertiesException::new(Vec::new());
        assert!(err.is_empty());
    }

    #[test]
    fn implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<MissingRequiredPropertiesException>();
    }
}
