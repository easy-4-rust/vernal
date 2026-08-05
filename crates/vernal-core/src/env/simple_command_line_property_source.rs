//! 简单命令行属性源。
//!
//! 对标 Spring `org.springframework.core.env.SimpleCommandLinePropertySource`。

use super::PropertySource;
use super::command_line_args::CommandLineArgs;
use super::command_line_property_source::CommandLinePropertySource;
use super::enumerable_property_source::EnumerablePropertySource;
use super::simple_command_line_args_parser::SimpleCommandLineArgsParser;

/// 简单命令行属性源。
///
/// 对应 Java: org.springframework.core.env.SimpleCommandLinePropertySource
///
/// Spring 语义：`CommandLinePropertySource` 的 `CommandLineArgs` 实现——
/// `getProperty("name")` 返回选项首值，`getProperty("name[index]")` 返回
/// 第 index 个值（越界返回 `None`）。
pub struct SimpleCommandLinePropertySource {
    name: String,
    args: CommandLineArgs,
}

impl SimpleCommandLinePropertySource {
    /// 从解析后的参数创建命名源。
    #[must_use]
    pub fn new(name: impl Into<String>, args: CommandLineArgs) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }

    /// 从原始参数解析并创建（名称默认 `"commandLineArgs"`）。
    ///
    /// 解析失败时退化为空参数集（对标 Spring 的宽松解析）。
    #[must_use]
    pub fn from_args(args: &[String]) -> Self {
        let parser = SimpleCommandLineArgsParser;
        let parsed = parser.parse(args).unwrap_or_default();
        Self {
            name: "commandLineArgs".to_string(),
            args: parsed,
        }
    }

    /// 解析 `name` 或 `name[index]` 形态的属性键。
    fn lookup(&self, key: &str) -> Option<String> {
        if let Some((name, index)) = key.rsplit_once('[') {
            let name = name.to_string();
            let index = index.trim_end_matches(']');
            let index: usize = index.parse().ok()?;
            return self.args.option_values(&name)?.get(index).cloned();
        }
        self.args
            .option_values(key)
            .and_then(|values| values.first().cloned())
    }
}

impl PropertySource for SimpleCommandLinePropertySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_property(&self, key: &str) -> Option<String> {
        self.lookup(key)
    }

    fn contains_property(&self, key: &str) -> bool {
        self.lookup(key).is_some()
    }
}

impl EnumerablePropertySource for SimpleCommandLinePropertySource {
    fn property_names(&self) -> Vec<String> {
        self.args.option_names()
    }
}

impl CommandLinePropertySource for SimpleCommandLinePropertySource {
    fn get_option_values(&self, name: &str) -> Option<&Vec<String>> {
        self.args.option_values(name)
    }

    fn get_non_option_args(&self) -> &[String] {
        self.args.non_option_args()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_with(args: &[&str]) -> SimpleCommandLinePropertySource {
        let owned: Vec<String> = args.iter().map(ToString::to_string).collect();
        SimpleCommandLinePropertySource::from_args(&owned)
    }

    #[test]
    fn option_first_value_as_property() {
        // A 类（合同对齐）：对标 Spring `name` → 首值
        let source = source_with(&["--port=8080", "--port=9090"]);
        assert_eq!(source.get_property("port").as_deref(), Some("8080"));
        assert!(source.contains_property("port"));
    }

    #[test]
    fn indexed_value_access() {
        // A 类（合同对齐）：对标 Spring `name[index]`
        let source = source_with(&["--port=8080", "--port=9090"]);
        assert_eq!(source.get_property("port[1]").as_deref(), Some("9090"));
        assert_eq!(source.get_property("port[5]"), None);
    }

    #[test]
    fn raw_option_and_non_option_access() {
        // A 类（合同对齐）：对标 Spring getOptionValues / getNonOptionArgs
        let source = source_with(&["file.txt", "--debug"]);
        assert_eq!(source.get_option_values("debug").unwrap()[0], "");
        assert_eq!(source.get_non_option_args(), &["file.txt".to_string()]);
    }

    #[test]
    fn enumerates_option_names() {
        // B 类（边界行为）：可枚举能力
        let source = source_with(&["--a=1", "--b=2"]);
        let mut names = source.property_names();
        names.sort();
        assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
    }
}
