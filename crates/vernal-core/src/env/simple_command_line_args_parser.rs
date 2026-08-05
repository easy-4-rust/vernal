//! 简单命令行参数解析器。
//!
//! 对标 Spring `org.springframework.core.env.SimpleCommandLineArgsParser`。

use super::CommandLineArgs;

/// 简单命令行参数解析器。
///
/// 对应 Java: org.springframework.core.env.SimpleCommandLineArgsParser
///
/// Spring 语义：解析 `--key=value`（等号形式）与 `--key value`（空格形式）
/// 两种选项语法；非 `--` 前缀的参数作为非选项参数收集。
pub struct SimpleCommandLineArgsParser;

impl SimpleCommandLineArgsParser {
    /// 解析命令行参数。
    ///
    /// # 错误
    ///
    /// `--key=value` 形式缺少值（`--key=`）时返回错误。
    pub fn parse(&self, args: &[String]) -> Result<CommandLineArgs, String> {
        let mut parsed = CommandLineArgs::new();
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            if let Some(rest) = arg.strip_prefix("--") {
                if let Some((name, value)) = rest.split_once('=') {
                    if name.is_empty() {
                        return Err("参数名不能为空".to_string());
                    }
                    parsed.add_option_arg(name, value);
                } else {
                    // `--key value` 形式：下一项为值
                    if index + 1 < args.len() {
                        parsed.add_option_arg(rest, &args[index + 1]);
                        index += 1;
                    } else {
                        parsed.add_option_arg(rest, "");
                    }
                }
            } else {
                parsed.add_non_option_arg(arg);
            }
            index += 1;
        }
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_equals_form() {
        // A 类（合同对齐）：对标 Spring `--key=value`
        let parser = SimpleCommandLineArgsParser;
        let args = parser
            .parse(&["--port=8080".to_string(), "--host=localhost".to_string()])
            .unwrap();
        assert_eq!(args.option_values("port").unwrap()[0], "8080");
        assert_eq!(args.option_values("host").unwrap()[0], "localhost");
    }

    #[test]
    fn parses_space_form() {
        // A 类（合同对齐）：对标 Spring `--key value`
        let parser = SimpleCommandLineArgsParser;
        let args = parser
            .parse(&["--port".to_string(), "8080".to_string()])
            .unwrap();
        assert_eq!(args.option_values("port").unwrap()[0], "8080");
    }

    #[test]
    fn collects_non_option_args() {
        // B 类（边界行为）：非 `--` 参数
        let parser = SimpleCommandLineArgsParser;
        let args = parser
            .parse(&["file.txt".to_string(), "--debug".to_string()])
            .unwrap();
        assert_eq!(args.non_option_args(), &["file.txt".to_string()]);
        assert!(args.contains_option("debug"));
    }

    #[test]
    fn repeated_options_accumulate() {
        // B 类（边界行为）：同名选项多值
        let parser = SimpleCommandLineArgsParser;
        let args = parser
            .parse(&["--port=1".to_string(), "--port=2".to_string()])
            .unwrap();
        assert_eq!(args.option_values("port").unwrap().len(), 2);
    }
}
