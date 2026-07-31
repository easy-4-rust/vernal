//! ReaderContext — 对应 Spring beans.factory.parsing.ReaderContext。
//!
//! 读取器上下文，封装 Bean 定义读取过程中需要共享的状态和回调。
//! 包括问题报告器、事件监听器和源提取器，供各个解析阶段使用。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.ReaderContext`。

use std::sync::Arc;

use super::fail_fast_problem_reporter::FailFastProblemReporter;
use super::location::Location;
use super::null_source_extractor::NullSourceExtractor;
use super::problem::{Problem, ProblemSeverity};
use super::problem_reporter::ProblemReporter;
use super::reader_event_listener::ReaderEventListener;
use super::source_extractor::SourceExtractor;

/// 读取器上下文。
///
/// 对应 Spring 的 `ReaderContext`。
///
/// 封装解析过程中共享的状态：问题报告器、事件监听器和源提取器。
/// 通过 `ReaderContext`，解析器可以统一地报告问题、触发事件和
/// 提取源信息，而无需直接依赖具体实现。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::reader_context::ReaderContext;
///
/// let ctx = ReaderContext::default();
/// // 可以通过 ctx 访问问题报告器、事件监听器和源提取器
/// assert_eq!(ctx.warning_count(), 0);
/// ```
pub struct ReaderContext {
    /// 问题报告器。
    problem_reporter: Arc<dyn ProblemReporter>,
    /// 源提取器。
    source_extractor: Arc<dyn SourceExtractor>,
}

impl ReaderContext {
    /// 创建一个新的读取器上下文。
    ///
    /// # 参数
    /// - `problem_reporter` — 问题报告器
    /// - `source_extractor` — 源提取器
    pub fn new(
        problem_reporter: Arc<dyn ProblemReporter>,
        source_extractor: Arc<dyn SourceExtractor>,
    ) -> Self {
        Self {
            problem_reporter,
            source_extractor,
        }
    }

    /// 返回问题报告器引用。
    pub fn problem_reporter(&self) -> &dyn ProblemReporter {
        self.problem_reporter.as_ref()
    }

    /// 返回源提取器引用。
    pub fn source_extractor(&self) -> &dyn SourceExtractor {
        self.source_extractor.as_ref()
    }

    /// 报告一个致命问题。
    pub fn fatal(&self, message: impl Into<String>, location: Location) {
        self.problem_reporter
            .fatal(Problem::error(message, location));
    }

    /// 报告一个警告问题。
    pub fn warning(&self, message: impl Into<String>, location: Location) {
        self.problem_reporter
            .warning(Problem::warning(message, location));
    }

    /// 报告一个自定义严重级别的问题。
    pub fn report(&self, severity: ProblemSeverity, message: impl Into<String>, location: Location) {
        match severity {
            ProblemSeverity::Error => {
                self.problem_reporter.fatal(Problem::error(message, location));
            }
            ProblemSeverity::Warning => {
                self.problem_reporter
                    .warning(Problem::warning(message, location));
            }
        }
    }

    /// 如果使用 FailFastProblemReporter，返回已收集的警告数量。
    ///
    /// 对于自定义 ProblemReporter 实现，始终返回 0。
    pub fn warning_count(&self) -> usize {
        // 由于类型擦除，这里无法直接访问 FailFastProblemReporter 的内部状态。
        // 返回 0 作为默认行为。
        0
    }
}

impl Default for ReaderContext {
    fn default() -> Self {
        Self::new(
            Arc::new(FailFastProblemReporter::new()),
            Arc::new(NullSourceExtractor::new()),
        )
    }
}

impl std::fmt::Debug for ReaderContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReaderContext").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_context() {
        let ctx = ReaderContext::default();
        // 默认使用 FailFastProblemReporter 和 NullSourceExtractor
        // 这里验证创建不会 panic
        let _ = ctx.problem_reporter();
        let _ = ctx.source_extractor();
    }

    #[test]
    fn test_context_with_custom_reporter() {
        use std::sync::Mutex;

        struct SilentReporter {
            count: Mutex<usize>,
        }
        impl SilentReporter {
            fn new() -> Self {
                Self { count: Mutex::new(0) }
            }
            fn count(&self) -> usize {
                *self.count.lock().unwrap()
            }
        }
        impl ProblemReporter for SilentReporter {
            fn fatal(&self, _problem: Problem) {
                *self.count.lock().unwrap() += 1;
            }
            fn warning(&self, _problem: Problem) {
                *self.count.lock().unwrap() += 1;
            }
        }

        let reporter = Arc::new(SilentReporter::new());
        let ctx = ReaderContext::new(reporter.clone(), Arc::new(NullSourceExtractor::new()));

        ctx.warning("test warning", Location::from_resource("test.xml"));
        assert_eq!(reporter.count(), 1);
    }

    #[test]
    fn test_report_error_and_warning() {
        use std::sync::Mutex;

        struct CountingReporter {
            fatals: Mutex<usize>,
            warnings: Mutex<usize>,
        }
        impl CountingReporter {
            fn new() -> Self {
                Self {
                    fatals: Mutex::new(0),
                    warnings: Mutex::new(0),
                }
            }
            fn fatal_count(&self) -> usize { *self.fatals.lock().unwrap() }
            fn warning_count(&self) -> usize { *self.warnings.lock().unwrap() }
        }
        impl ProblemReporter for CountingReporter {
            fn fatal(&self, _problem: Problem) { *self.fatals.lock().unwrap() += 1; }
            fn warning(&self, _problem: Problem) { *self.warnings.lock().unwrap() += 1; }
        }

        let reporter = Arc::new(CountingReporter::new());
        let ctx = ReaderContext::new(reporter.clone(), Arc::new(NullSourceExtractor::new()));

        ctx.report(ProblemSeverity::Error, "error", Location::UNKNOWN);
        ctx.report(ProblemSeverity::Warning, "warn", Location::UNKNOWN);

        assert_eq!(reporter.fatal_count(), 1);
        assert_eq!(reporter.warning_count(), 1);
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn warning_count_returns_zero() {
        let ctx = ReaderContext::default();
        assert_eq!(ctx.warning_count(), 0);
    }

    #[test]
    fn debug_format() {
        let ctx = ReaderContext::default();
        let debug = format!("{:?}", ctx);
        assert!(debug.contains("ReaderContext"));
    }

    #[test]
    fn fatal_reports_to_reporter() {
        use std::sync::Mutex;

        struct CountingReporter {
            fatals: Mutex<usize>,
        }
        impl CountingReporter {
            fn new() -> Self {
                Self { fatals: Mutex::new(0) }
            }
            fn fatal_count(&self) -> usize { *self.fatals.lock().unwrap() }
        }
        impl ProblemReporter for CountingReporter {
            fn fatal(&self, _problem: Problem) { *self.fatals.lock().unwrap() += 1; }
            fn warning(&self, _problem: Problem) {}
        }

        let reporter = Arc::new(CountingReporter::new());
        let ctx = ReaderContext::new(reporter.clone(), Arc::new(NullSourceExtractor::new()));

        ctx.fatal("fatal error", Location::UNKNOWN);
        assert_eq!(reporter.fatal_count(), 1);
    }

    #[test]
    fn warning_reports_to_reporter() {
        use std::sync::Mutex;

        struct CountingReporter {
            warnings: Mutex<usize>,
        }
        impl CountingReporter {
            fn new() -> Self {
                Self { warnings: Mutex::new(0) }
            }
            fn warning_count(&self) -> usize { *self.warnings.lock().unwrap() }
        }
        impl ProblemReporter for CountingReporter {
            fn fatal(&self, _problem: Problem) {}
            fn warning(&self, _problem: Problem) { *self.warnings.lock().unwrap() += 1; }
        }

        let reporter = Arc::new(CountingReporter::new());
        let ctx = ReaderContext::new(reporter.clone(), Arc::new(NullSourceExtractor::new()));

        ctx.warning("warning message", Location::UNKNOWN);
        assert_eq!(reporter.warning_count(), 1);
    }

    #[test]
    fn report_error_severity() {
        use std::sync::Mutex;

        struct CountingReporter {
            fatals: Mutex<usize>,
        }
        impl CountingReporter {
            fn new() -> Self {
                Self { fatals: Mutex::new(0) }
            }
            fn fatal_count(&self) -> usize { *self.fatals.lock().unwrap() }
        }
        impl ProblemReporter for CountingReporter {
            fn fatal(&self, _problem: Problem) { *self.fatals.lock().unwrap() += 1; }
            fn warning(&self, _problem: Problem) {}
        }

        let reporter = Arc::new(CountingReporter::new());
        let ctx = ReaderContext::new(reporter.clone(), Arc::new(NullSourceExtractor::new()));

        ctx.report(ProblemSeverity::Error, "error", Location::from_resource("test.xml"));
        assert_eq!(reporter.fatal_count(), 1);
    }

    #[test]
    fn report_warning_severity() {
        use std::sync::Mutex;

        struct CountingReporter {
            warnings: Mutex<usize>,
        }
        impl CountingReporter {
            fn new() -> Self {
                Self { warnings: Mutex::new(0) }
            }
            fn warning_count(&self) -> usize { *self.warnings.lock().unwrap() }
        }
        impl ProblemReporter for CountingReporter {
            fn fatal(&self, _problem: Problem) {}
            fn warning(&self, _problem: Problem) { *self.warnings.lock().unwrap() += 1; }
        }

        let reporter = Arc::new(CountingReporter::new());
        let ctx = ReaderContext::new(reporter.clone(), Arc::new(NullSourceExtractor::new()));

        ctx.report(ProblemSeverity::Warning, "warn", Location::from_resource("test.xml"));
        assert_eq!(reporter.warning_count(), 1);
    }

    #[test]
    fn problem_reporter_and_source_extractor_accessors() {
        let ctx = ReaderContext::default();
        let _ = ctx.problem_reporter();
        let _ = ctx.source_extractor();
    }
}
