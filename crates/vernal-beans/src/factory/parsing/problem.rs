//! Problem — 对应 Spring beans.factory.parsing.Problem。
//!
//! 在 Bean 定义解析过程中遇到的问题。Problem 封装了问题的严重级别、
//! 描述信息和发生位置。配合 [`super::ProblemReporter`] 使用，
//! 决定是快速失败还是收集后统一报告。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.Problem`。

use std::fmt;

use super::location::Location;

/// 问题严重级别。
///
/// 对应 Spring 的 `Problem` 中隐含的严重程度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProblemSeverity {
    /// 警告 — 解析可以继续，但可能存在潜在问题。
    Warning,
    /// 错误 — 解析不应继续，必须立即或延迟报告。
    Error,
}

/// Bean 定义解析过程中遇到的问题。
///
/// 对应 Spring 的 `Problem`。
///
/// 包含问题的严重级别、描述信息和源位置。用于配合
/// [`super::ProblemReporter`] 接口进行错误处理。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::problem::{Problem, ProblemSeverity};
/// use vernal_beans::factory::parsing::location::Location;
///
/// let loc = Location::new("config.xml", 10, 5);
/// let problem = Problem::new(
///     ProblemSeverity::Error,
///     "Bean 'myBean' has invalid class attribute",
///     loc,
/// );
/// assert_eq!(problem.severity(), ProblemSeverity::Error);
/// ```
#[derive(Debug, Clone)]
pub struct Problem {
    /// 严重级别。
    severity: ProblemSeverity,
    /// 问题描述。
    message: String,
    /// 问题发生的位置。
    location: Location,
    /// 可选的根本原因描述。
    resource_description: Option<String>,
}

impl Problem {
    /// 创建一个新的问题。
    ///
    /// # 参数
    /// - `severity` — 严重级别
    /// - `message` — 问题描述
    /// - `location` — 问题发生的位置
    pub fn new(severity: ProblemSeverity, message: impl Into<String>, location: Location) -> Self {
        Self {
            severity,
            message: message.into(),
            location,
            resource_description: None,
        }
    }

    /// 创建一个错误级别问题。
    pub fn error(message: impl Into<String>, location: Location) -> Self {
        Self::new(ProblemSeverity::Error, message, location)
    }

    /// 创建一个警告级别问题。
    pub fn warning(message: impl Into<String>, location: Location) -> Self {
        Self::new(ProblemSeverity::Warning, message, location)
    }

    /// 设置资源描述（链式构建）。
    #[must_use]
    pub fn with_resource_description(mut self, desc: impl Into<String>) -> Self {
        self.resource_description = Some(desc.into());
        self
    }

    /// 返回问题严重级别。
    pub fn severity(&self) -> ProblemSeverity {
        self.severity
    }

    /// 返回问题描述。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 返回问题发生的位置。
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// 返回资源描述（如果有）。
    pub fn resource_description(&self) -> Option<&str> {
        self.resource_description.as_deref()
    }

    /// 是否为错误级别。
    pub fn is_error(&self) -> bool {
        self.severity == ProblemSeverity::Error
    }

    /// 是否为警告级别。
    pub fn is_warning(&self) -> bool {
        self.severity == ProblemSeverity::Warning
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.severity, self.message)?;
        if self.location.is_known() {
            write!(f, " ({})", self.location)?;
        }
        Ok(())
    }
}

impl std::error::Error for Problem {}

impl fmt::Display for ProblemSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProblemSeverity::Warning => write!(f, "WARNING"),
            ProblemSeverity::Error => write!(f, "ERROR"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_creation_and_accessors() {
        let loc = Location::new("test.xml", 5, 10);
        let p = Problem::new(ProblemSeverity::Error, "Invalid bean name", loc);
        assert_eq!(p.severity(), ProblemSeverity::Error);
        assert_eq!(p.message(), "Invalid bean name");
        assert!(p.is_error());
        assert!(!p.is_warning());
        assert!(p.location().is_known());
    }

    #[test]
    fn test_problem_factory_methods() {
        let loc = Location::from_resource("config.xml");

        let err = Problem::error("bad class", loc.clone());
        assert!(err.is_error());

        let warn = Problem::warning("deprecated syntax", loc);
        assert!(warn.is_warning());
    }

    #[test]
    fn test_problem_display() {
        let loc = Location::new("ctx.xml", 3, 1);
        let p = Problem::error("missing attribute", loc);
        let display = format!("{}", p);
        assert!(display.contains("ERROR"));
        assert!(display.contains("missing attribute"));
        assert!(display.contains("ctx.xml"));
    }

    #[test]
    fn test_problem_with_resource_description() {
        let loc = Location::UNKNOWN;
        let p = Problem::error("fail", loc).with_resource_description("BeanDefinition");
        assert_eq!(p.resource_description(), Some("BeanDefinition"));
    }
}
