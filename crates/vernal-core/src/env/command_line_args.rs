//! 命令行参数。
//!
//! 对标 Spring `org.springframework.core.env.CommandLineArgs`。

use std::collections::HashMap;

/// 命令行参数。
///
/// 对应 Java: org.springframework.core.env.CommandLineArgs
///
/// Spring 语义：`SimpleCommandLineArgsParser` 的解析产物——选项名 → 值列表
/// （同选项可重复出现）与非选项参数列表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLineArgs {
    option_values: HashMap<String, Vec<String>>,
    non_option_args: Vec<String>,
}

impl Default for CommandLineArgs {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandLineArgs {
    /// 创建空参数集。
    #[must_use]
    pub fn new() -> Self {
        Self {
            option_values: HashMap::new(),
            non_option_args: Vec::new(),
        }
    }

    /// 追加选项值。
    pub fn add_option_arg(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.option_values
            .entry(name.into())
            .or_default()
            .push(value.into());
    }

    /// 追加非选项参数。
    pub fn add_non_option_arg(&mut self, value: impl Into<String>) {
        self.non_option_args.push(value.into());
    }

    /// 判断是否包含选项（对标 Spring `containsOption`）。
    #[must_use]
    pub fn contains_option(&self, name: &str) -> bool {
        self.option_values.contains_key(name)
    }

    /// 返回全部选项名集合。
    #[must_use]
    pub fn option_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.option_values.keys().cloned().collect();
        names.sort();
        names
    }

    /// 返回选项的全部值（对标 Spring `getOptionValues`）。
    #[must_use]
    pub fn option_values(&self, name: &str) -> Option<&Vec<String>> {
        self.option_values.get(name)
    }

    /// 返回非选项参数列表。
    #[must_use]
    pub fn non_option_args(&self) -> &[String] {
        &self.non_option_args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_multiple_values_per_option() {
        // A 类（合同对齐）：对标 Spring 同名选项多值
        let mut args = CommandLineArgs::new();
        args.add_option_arg("port", "8080");
        args.add_option_arg("port", "9090");
        assert!(args.contains_option("port"));
        assert_eq!(args.option_values("port").unwrap(), &vec!["8080".to_string(), "9090".to_string()]);
    }

    #[test]
    fn collects_non_option_args() {
        // B 类（边界行为）
        let mut args = CommandLineArgs::new();
        args.add_non_option_arg("file.txt");
        assert_eq!(args.non_option_args(), &["file.txt".to_string()]);
    }

    #[test]
    fn missing_option_returns_none() {
        // B 类（边界行为）
        let args = CommandLineArgs::new();
        assert!(!args.contains_option("nope"));
        assert!(args.option_values("nope").is_none());
    }
}
