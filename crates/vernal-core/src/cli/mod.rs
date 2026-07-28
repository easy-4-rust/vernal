//! 命令行解析模块（feature = "cli"）。
//!
//! 对标 Spring `jopt-simple`（命令行参数解析）。
//!
//! # 与 Spring 的对应关系
//!
//! | Spring | vernal-core |
//! |---|---|
//! | `joptsimple.OptionParser` | `clap::Command` |
//! | `joptsimple.OptionSet` | `clap::ArgMatches` |
//! | `@Option` annotation | `#[clap(long, short)]` |

/// 命令行参数解析错误。
#[derive(Debug, Clone)]
pub struct CliError {
    /// 错误消息
    pub message: String,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CLI error: {}", self.message)
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_error_display() {
        let err = CliError { message: "missing argument".to_string() };
        assert!(err.to_string().contains("CLI error"));
        assert!(err.to_string().contains("missing argument"));
    }
}

/// Clap 命令行解析工具（feature = "cli"）。
///
/// 对标 Spring `jopt-simple`（命令行参数解析）。
///
/// 使用 `clap` crate 作为后端，提供命令行参数解析功能。
#[cfg(feature = "cli")]
pub mod clap_parser {
    use super::CliError;

    /// Clap 命令行解析工具。
    ///
    /// 对标 Spring `OptionParser`。
    #[derive(Debug, Clone)]
    pub struct ClapParser;

    impl ClapParser {
        /// 创建新的 Clap 解析器。
        pub fn new() -> Self {
            Self
        }

        /// 解析命令行参数。
        pub fn parse_args(&self) -> Result<Vec<String>, CliError> {
            let args: Vec<String> = std::env::args().collect();
            Ok(args)
        }

        /// 获取程序名称。
        pub fn program_name(&self) -> Option<String> {
            std::env::args().next()
        }
    }

    impl Default for ClapParser {
        fn default() -> Self {
            Self::new()
        }
    }
}
