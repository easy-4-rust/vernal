//! 简化拦截器的调用上下文。
//!
//! 对标 tx_di 的 `CallContext`：携带方法名，不含参数序列化。
//! 适用于日志、指标、鉴权等不需要方法参数的简单场景。

/// 简化拦截器的调用上下文。
///
/// 携带被拦截方法的基本信息，供 `before` / `after` / `around` 钩子使用。
/// 不包含方法参数（与完整 AOP 的 `Invocation` 不同），保持零开销。
///
/// # 设计来源
///
/// 对标 tx_di 的 `CallContext`，但不包含参数序列化（vernal 的完整 AOP
/// 通过 `Invocation` 提供参数访问）。
#[derive(Debug, Clone)]
pub struct SimpleInvocationContext {
    /// 被拦截方法的静态名称
    method: &'static str,
}

impl SimpleInvocationContext {
    /// 创建调用上下文。
    ///
    /// # 参数
    /// - `method`：被拦截方法的静态名称（通常由 `#[simple_intercept]` 宏自动填充）
    #[must_use]
    pub const fn new(method: &'static str) -> Self {
        Self { method }
    }

    /// 返回被拦截方法的名称。
    #[must_use]
    pub fn method(&self) -> &'static str {
        self.method
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_new() {
        let ctx = SimpleInvocationContext::new("test_method");
        assert_eq!(ctx.method(), "test_method");
    }

    #[test]
    fn context_clone() {
        let ctx = SimpleInvocationContext::new("test_method");
        let cloned = ctx.clone();
        assert_eq!(cloned.method(), "test_method");
    }

    #[test]
    fn context_debug() {
        let ctx = SimpleInvocationContext::new("test_method");
        let debug = format!("{:?}", ctx);
        assert!(debug.contains("test_method"));
    }
}
