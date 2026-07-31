//! FailFastProblemReporter — 对应 Spring FailFastProblemReporter。
//!
//! 快速失败的问题报告器。遇到致命问题时立即 panic，警告则打印日志。
//! 这是 Spring 默认的 ProblemReporter 实现策略。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.FailFastProblemReporter`。

use super::problem::{Problem, ProblemSeverity};
use super::problem_reporter::ProblemReporter;

/// 快速失败的问题报告器。
///
/// 对应 Spring 的 `FailFastProblemReporter`。
///
/// - **Fatal 问题**：立即 panic（生产中可通过自定义策略改为 Err 返回）
/// - **Warning 问题**：记录到内部缓冲区供后续查询
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::fail_fast_problem_reporter::FailFastProblemReporter;
/// use vernal_beans::factory::parsing::problem_reporter::ProblemReporter;
/// use vernal_beans::factory::parsing::problem::Problem;
/// use vernal_beans::factory::parsing::location::Location;
///
/// let reporter = FailFastProblemReporter::new();
/// let warn = Problem::warning("deprecated attribute", Location::from_resource("cfg.xml"));
/// reporter.warning(warn);
/// assert_eq!(reporter.warning_count(), 1);
/// ```
pub struct FailFastProblemReporter {
    warnings: std::sync::Mutex<Vec<Problem>>,
}

impl FailFastProblemReporter {
    /// 创建一个新的快速失败报告器。
    pub fn new() -> Self {
        Self {
            warnings: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// 返回已收集的警告数量。
    pub fn warning_count(&self) -> usize {
        self.warnings.lock().unwrap().len()
    }

    /// 返回所有警告的快照。
    pub fn warnings(&self) -> Vec<Problem> {
        self.warnings.lock().unwrap().clone()
    }

    /// 是否有警告。
    pub fn has_warnings(&self) -> bool {
        !self.warnings.lock().unwrap().is_empty()
    }
}

impl Default for FailFastProblemReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProblemReporter for FailFastProblemReporter {
    fn fatal(&self, problem: Problem) {
        panic!(
            "Bean definition parsing failed: {} [{}]",
            problem.message(),
            problem.location()
        );
    }

    fn warning(&self, problem: Problem) {
        self.warnings.lock().unwrap().push(problem);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::parsing::location::Location;
    use crate::factory::parsing::problem::Problem;

    #[test]
    fn test_warning_collection() {
        let reporter = FailFastProblemReporter::new();
        assert!(!reporter.has_warnings());
        assert_eq!(reporter.warning_count(), 0);

        let loc = Location::from_resource("test.xml");
        reporter.warning(Problem::warning("warn1", loc.clone()));
        reporter.warning(Problem::warning("warn2", loc));

        assert!(reporter.has_warnings());
        assert_eq!(reporter.warning_count(), 2);
    }

    #[test]
    fn test_warnings_snapshot() {
        let reporter = FailFastProblemReporter::new();
        let loc = Location::from_resource("app.xml");
        reporter.warning(Problem::warning("deprecated", loc));

        let warnings = reporter.warnings();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message(), "deprecated");
    }

    #[test]
    #[should_panic(expected = "Bean definition parsing failed")]
    fn test_fatal_panics() {
        let reporter = FailFastProblemReporter::new();
        let loc = Location::new("bad.xml", 1, 1);
        reporter.fatal(Problem::error("fatal error", loc));
    }
}
