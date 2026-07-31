//! ParseState — 对应 Spring beans.factory.parsing.ParseState。
//!
//! 解析状态栈，跟踪 XML/Properties 等配置文件的解析层次。
//! 当遇到嵌套元素时推入栈，解析完成后弹出。用于错误报告时
//! 提供当前解析上下文（如 "in bean 'myBean', in property 'name'"）。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.ParseState`。

use std::fmt;

/// 解析状态栈。
///
/// 对应 Spring 的 `ParseState`。
///
/// 在 Bean 定义解析过程中维护一个状态栈。每进入一个新的嵌套
/// 层级（如 `<bean>`、`<property>`），推入一个状态条目；离开时
/// 弹出。当发生错误时，可以生成当前解析路径的描述。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::parse_state::ParseState;
///
/// let mut state = ParseState::new();
/// state.push("bean 'myService'");
/// state.push("property 'dataSource'");
///
/// let path = state.to_string();
/// assert!(path.contains("bean 'myService'"));
/// assert!(path.contains("property 'dataSource'"));
///
/// state.pop();
/// assert!(!state.to_string().contains("dataSource"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct ParseState {
    /// 状态栈。
    stack: Vec<String>,
}

impl ParseState {
    /// 创建一个空的解析状态。
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// 推入一个状态条目。
    ///
    /// # 参数
    /// - `entry` — 当前解析层次的描述
    pub fn push(&mut self, entry: impl Into<String>) {
        self.stack.push(entry.into());
    }

    /// 弹出最近的状态条目。
    ///
    /// 如果栈为空则不做任何操作。
    pub fn pop(&mut self) {
        self.stack.pop();
    }

    /// 返回当前栈深度。
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// 栈是否为空。
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// 返回栈顶条目（最近推入的）。
    pub fn peek(&self) -> Option<&str> {
        self.stack.last().map(|s| s.as_str())
    }

    /// 生成当前解析路径的描述。
    ///
    /// 格式：`"in <entry1>, in <entry2>, ..."`。
    pub fn path(&self) -> String {
        if self.stack.is_empty() {
            return String::new();
        }
        self.stack
            .iter()
            .map(|entry| format!("in {}", entry))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl fmt::Display for ParseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path();
        if path.is_empty() {
            write!(f, "(empty parse state)")
        } else {
            write!(f, "{}", path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut state = ParseState::new();
        assert!(state.is_empty());
        assert_eq!(state.depth(), 0);

        state.push("bean");
        assert_eq!(state.depth(), 1);
        assert_eq!(state.peek(), Some("bean"));

        state.push("property");
        assert_eq!(state.depth(), 2);
        assert_eq!(state.peek(), Some("property"));

        state.pop();
        assert_eq!(state.depth(), 1);
        assert_eq!(state.peek(), Some("bean"));
    }

    #[test]
    fn test_path_format() {
        let mut state = ParseState::new();
        state.push("bean 'myService'");
        state.push("property 'dataSource'");
        state.push("constructor-arg");

        let path = state.path();
        assert_eq!(
            path,
            "in bean 'myService', in property 'dataSource', in constructor-arg"
        );
    }

    #[test]
    fn test_display_empty() {
        let state = ParseState::new();
        assert_eq!(format!("{}", state), "(empty parse state)");
    }

    #[test]
    fn test_pop_on_empty_is_safe() {
        let mut state = ParseState::new();
        state.pop(); // 不应 panic
        assert!(state.is_empty());
    }
}
