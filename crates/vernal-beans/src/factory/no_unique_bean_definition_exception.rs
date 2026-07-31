//! NoUniqueBeanDefinitionException — 对应 Spring `org.springframework.beans.factory.NoUniqueBeanDefinitionException`。
//!
//! 当有多个 Bean 匹配但期望唯一时抛出的异常。

use std::fmt;

/// 当有多个 Bean 匹配但期望唯一时抛出的异常。
///
/// 对应 Java 类：`org.springframework.beans.factory.NoUniqueBeanDefinitionException`。
#[derive(Debug)]
pub struct NoUniqueBeanDefinitionException {
    type_name: String,
    message: String,
    number_found: usize,
}

impl NoUniqueBeanDefinitionException {
    /// 创建一个新的 NoUniqueBeanDefinitionException。
    pub fn new(type_name: impl Into<String>, number_found: usize) -> Self {
        let type_name = type_name.into();
        Self {
            message: format!(
                "No qualifying bean of type '{}' available: expected single matching bean but found {}",
                type_name, number_found
            ),
            type_name,
            number_found,
        }
    }

    /// 获取类型名称。
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// 获取找到的 Bean 数量。
    pub fn number_found(&self) -> usize {
        self.number_found
    }
}

impl fmt::Display for NoUniqueBeanDefinitionException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for NoUniqueBeanDefinitionException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let e = NoUniqueBeanDefinitionException::new("String", 3);
        assert_eq!(e.type_name(), "String");
        assert_eq!(e.number_found(), 3);
        assert!(format!("{}", e).contains("3"));
    }
}
