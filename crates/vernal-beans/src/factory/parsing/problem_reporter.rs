//! ProblemReporter — 对应 Spring beans.factory.parsing.ProblemReporter。
//!
//! 问题报告器接口。在 Bean 定义解析过程中，解析器使用 ProblemReporter
//! 报告遇到的问题。不同实现可以决定是立即失败（Fail-Fast）还是收集
//! 问题后统一报告。
//!
//! 对应 Java 接口：`org.springframework.beans.factory.parsing.ProblemReporter`。

use super::problem::Problem;

/// Bean 定义解析问题报告器。
///
/// 对应 Spring 的 `ProblemReporter`。
///
/// 在解析 Bean 定义时，将发现的问题委托给 ProblemReporter 处理。
/// 框架提供两种标准实现：
///
/// - [`super::fail_fast_problem_reporter::FailFastProblemReporter`] — 遇到
///   错误立即 panic / 返回 Err
/// - 用户可自定义实现，收集所有问题后一次性报告
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::problem_reporter::ProblemReporter;
/// use vernal_beans::factory::parsing::problem::{Problem, ProblemSeverity};
/// use vernal_beans::factory::parsing::location::Location;
///
/// struct CollectingReporter {
///     problems: std::sync::Mutex<Vec<Problem>>,
/// }
///
/// impl ProblemReporter for CollectingReporter {
///     fn fatal(&self, problem: Problem) {
///         self.problems.lock().unwrap().push(problem);
///     }
///     fn warning(&self, problem: Problem) {
///         self.problems.lock().unwrap().push(problem);
///     }
/// }
/// ```
pub trait ProblemReporter: Send + Sync {
    /// 报告致命问题（解析不应继续）。
    ///
    /// 对应 Spring 的 `fatal(Problem)`。实现可以选择立即 panic、
    /// 返回 Err 或将问题入队。
    fn fatal(&self, problem: Problem);

    /// 报告警告问题（解析可以继续，但可能存在潜在问题）。
    ///
    /// 对应 Spring 的 `warning(Problem)`。
    fn warning(&self, problem: Problem);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 用于测试的收集型报告器。
    struct CollectingReporter {
        fatals: Mutex<Vec<String>>,
        warnings: Mutex<Vec<String>>,
    }

    impl CollectingReporter {
        fn new() -> Self {
            Self {
                fatals: Mutex::new(Vec::new()),
                warnings: Mutex::new(Vec::new()),
            }
        }

        fn fatal_count(&self) -> usize {
            self.fatals.lock().unwrap().len()
        }

        fn warning_count(&self) -> usize {
            self.warnings.lock().unwrap().len()
        }
    }

    impl ProblemReporter for CollectingReporter {
        fn fatal(&self, problem: Problem) {
            self.fatals.lock().unwrap().push(problem.message().to_string());
        }

        fn warning(&self, problem: Problem) {
            self.warnings.lock().unwrap().push(problem.message().to_string());
        }
    }

    #[test]
    fn test_collecting_reporter_fatal() {
        use super::super::location::Location;
        use super::super::problem::{Problem, ProblemSeverity};

        let reporter = CollectingReporter::new();
        let loc = Location::new("test.xml", 1, 1);
        let problem = Problem::error("bad bean", loc);
        reporter.fatal(problem);

        assert_eq!(reporter.fatal_count(), 1);
        assert_eq!(reporter.warning_count(), 0);
    }

    #[test]
    fn test_collecting_reporter_warning() {
        use super::super::location::Location;
        use super::super::problem::Problem;

        let reporter = CollectingReporter::new();
        let loc = Location::from_resource("config.xml");
        let problem = Problem::warning("deprecated", loc);
        reporter.warning(problem);

        assert_eq!(reporter.warning_count(), 1);
        assert_eq!(reporter.fatal_count(), 0);
    }

    #[test]
    fn test_collecting_reporter_mixed() {
        use super::super::location::Location;
        use super::super::problem::Problem;

        let reporter = CollectingReporter::new();
        let loc = Location::UNKNOWN;

        reporter.fatal(Problem::error("error1", loc.clone()));
        reporter.fatal(Problem::error("error2", loc.clone()));
        reporter.warning(Problem::warning("warn1", loc));

        assert_eq!(reporter.fatal_count(), 2);
        assert_eq!(reporter.warning_count(), 1);
    }
}
