//! 命令行属性源契约。
//!
//! 对标 Spring `org.springframework.core.env.CommandLinePropertySource`。

use super::EnumerablePropertySource;

/// 命令行属性源契约。
///
/// 对应 Java: org.springframework.core.env.CommandLinePropertySource
///
/// Spring 语义：基于 [`CommandLineArgs`](crate::env::CommandLineArgs) 的抽象属性源——属性 `name` 映射到
/// 选项首值，`name[index]` 映射到第 index 个值；同时暴露选项与非选项参数
/// 的原始访问。
pub trait CommandLinePropertySource: EnumerablePropertySource {
    /// 返回选项全部值。
    ///
    /// 对应 Java: `CommandLinePropertySource#getOptionValues(String)`
    fn get_option_values(&self, name: &str) -> Option<&Vec<String>>;

    /// 返回非选项参数列表。
    ///
    /// 对应 Java: `CommandLinePropertySource#getNonOptionArgs()`
    fn get_non_option_args(&self) -> &[String];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::SimpleCommandLinePropertySource;

    #[test]
    fn simple_source_satisfies_contract() {
        // D 类（重构安全）：`SimpleCommandLinePropertySource` 实现该契约
        fn assert_source<T: CommandLinePropertySource>() {}
        assert_source::<SimpleCommandLinePropertySource>();
    }
}
