//! BeanDefinitionParsingException — Spring 风格的 Bean 定义解析异常。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.BeanDefinitionParsingException`。
//!
//! 当解析 Bean 定义配置时遇到问题抛出，包含解析问题的详情。

use std::error::Error;
use std::fmt;

/// Spring 风格的 Bean 定义解析异常。
///
/// 对应 Spring 的 `BeanDefinitionParsingException`。
///
/// 表示在解析 Bean 定义配置（如 XML、注解）过程中遇到问题。
#[derive(Clone, Debug)]
pub struct BeanDefinitionParsingException {
    /// 解析问题描述。
    problem: String,
    /// 配置位置描述。
    location: Option<String>,
}

impl BeanDefinitionParsingException {
    /// 创建新的 BeanDefinitionParsingException。
    pub fn new(problem: impl Into<String>) -> Self {
        Self {
            problem: problem.into(),
            location: None,
        }
    }

    /// 创建带配置位置的 BeanDefinitionParsingException。
    pub fn with_location(problem: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            problem: problem.into(),
            location: Some(location.into()),
        }
    }

    /// 获取解析问题描述。
    pub fn problem(&self) -> &str {
        &self.problem
    }

    /// 获取配置位置。
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }
}

impl fmt::Display for BeanDefinitionParsingException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse bean definition: {}", self.problem)?;
        if let Some(ref loc) = self.location {
            write!(f, " (at {loc})")?;
        }
        Ok(())
    }
}

impl Error for BeanDefinitionParsingException {}
