//! BeanNotOfRequiredTypeException — 对应 Spring `org.springframework.beans.factory.BeanNotOfRequiredTypeException`。
//!
//! Bean 类型不匹配时抛出的异常。

use std::any::TypeId;
use std::fmt;

/// Bean 类型不匹配时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.BeanNotOfRequiredTypeException`。
#[derive(Debug)]
pub struct BeanNotOfRequiredTypeException {
    bean_name: String,
    required_type_name: String,
    actual_type_name: String,
}

impl BeanNotOfRequiredTypeException {
    /// 创建一个新的 BeanNotOfRequiredTypeException。
    pub fn new(
        bean_name: impl Into<String>,
        required_type: impl Into<String>,
        actual_type: impl Into<String>,
    ) -> Self {
        Self {
            bean_name: bean_name.into(),
            required_type_name: required_type.into(),
            actual_type_name: actual_type.into(),
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 获取期望的类型名称。
    pub fn required_type_name(&self) -> &str {
        &self.required_type_name
    }

    /// 获取实际的类型名称。
    pub fn actual_type_name(&self) -> &str {
        &self.actual_type_name
    }
}

impl fmt::Display for BeanNotOfRequiredTypeException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bean named '{}' is expected to be of type '{}' but was actually of type '{}'",
            self.bean_name, self.required_type_name, self.actual_type_name
        )
    }
}

impl std::error::Error for BeanNotOfRequiredTypeException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = BeanNotOfRequiredTypeException::new("myBean", "String", "i32");
        assert_eq!(e.bean_name(), "myBean");
        assert_eq!(e.required_type_name(), "String");
        assert_eq!(e.actual_type_name(), "i32");
    }

    #[test]
    fn test_display() {
        let e = BeanNotOfRequiredTypeException::new("myBean", "String", "i32");
        assert!(format!("{}", e).contains("myBean"));
        assert!(format!("{}", e).contains("String"));
        assert!(format!("{}", e).contains("i32"));
    }
}
