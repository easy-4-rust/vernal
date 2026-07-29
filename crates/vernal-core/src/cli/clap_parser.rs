//! CLI 参数解析器。
//!
//! 对标 Spring `org.springframework.boot.cli.parser.OptionParsingCommand`。
//! 底层使用 `clap`（对标 jopt-simple）。

#![cfg(feature = "cli")]

use std::env;

use super::cli_error::CliError;

/// CLI 参数解析器。
///
/// 对应 Java: `org.springframework.boot.cli.parser.OptionParsingCommand`
pub struct ClapParser;

impl ClapParser {
    /// 创建新的 CLI 解析器。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 解析命令行参数。
    ///
    /// 对应 Java: `OptionParsingCommand.parseOptions(String[])`
    /// 输入是程序接收到的参数（不含程序名）。
    ///
    /// # Errors
    ///
    /// 参数解析失败时返回 `CliError`。
    pub fn parse_args(&self, args: &[String]) -> Result<Vec<String>, CliError> {
        // 简单实现：将 `--key=value` 拆为 `key=value` 标记
        let mut result = Vec::new();
        for arg in args {
            if let Some(stripped) = arg.strip_prefix("--") {
                result.push(stripped.to_string());
            } else {
                result.push(arg.clone());
            }
        }
        Ok(result)
    }

    /// 获取程序名（`argv[0]`）。
    ///
    /// 对应 Java: `mainClass.getName()` 或 `argv[0]`
    #[must_use]
    pub fn program_name(&self) -> Option<String> {
        env::args().next()
    }
}

impl Default for ClapParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_long_options() {
        let parser = ClapParser::new();
        let args = vec!["--name=test".to_string(), "--value=42".to_string()];
        let result = parser.parse_args(&args).unwrap();
        assert_eq!(result, vec!["name=test", "value=42"]);
    }

    #[test]
    fn parse_short_options_are_kept_as_is() {
        let parser = ClapParser::new();
        let args = vec!["-x".to_string()];
        let result = parser.parse_args(&args).unwrap();
        assert_eq!(result, vec!["-x"]);
    }

    #[test]
    fn parse_empty_args() {
        let parser = ClapParser::new();
        let result = parser.parse_args(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_positional_args() {
        let parser = ClapParser::new();
        let args = vec!["positional1".to_string(), "positional2".to_string()];
        let result = parser.parse_args(&args).unwrap();
        assert_eq!(result, vec!["positional1", "positional2"]);
    }

    #[test]
    fn default_impl_works() {
        let _parser: ClapParser = ClapParser::default();
    }
}
