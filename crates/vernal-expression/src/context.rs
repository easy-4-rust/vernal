//! 表达式求值上下文。

/// 表达式求值上下文 trait。
///
/// 对标 Spring 的 `EvaluationContext`。
pub trait EvaluationContext: Send + Sync {
    /// 获取变量值。
    fn get_variable(&self, name: &str) -> Option<&dyn std::any::Any>;

    /// 设置变量值。
    fn set_variable(&mut self, name: String, value: Box<dyn std::any::Any + Send + Sync>);
}
