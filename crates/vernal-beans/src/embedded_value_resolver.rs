//! EmbeddedValueResolver — 基于环境的嵌入式值解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.EmbeddedValueResolver`。
//!
//! 实现 `StringValueResolver`，使用 `Environment` 解析 `${...}` 占位符。

use std::sync::Arc;

use crate::environment::Environment;
use crate::string_value_resolver::StringValueResolver;

/// Spring 风格的嵌入式值解析器。
///
/// 对应 Spring 的 `EmbeddedValueResolver`。
///
/// 包装一个 `Environment`，对每个输入字符串调用
/// `resolve_placeholders`，用于在 Bean 后处理器中解析注解属性、
/// XML 属性等位置的占位符。
pub struct EmbeddedValueResolver {
    environment: Arc<dyn Environment + Send + Sync>,
}

impl std::fmt::Debug for EmbeddedValueResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddedValueResolver")
            .field("environment", &"Environment(...)")
            .finish()
    }
}

impl EmbeddedValueResolver {
    /// 创建嵌入式值解析器。
    pub fn new(environment: Arc<dyn Environment + Send + Sync>) -> Self {
        Self { environment }
    }

    /// 获取内部环境引用。
    pub fn environment(&self) -> &Arc<dyn Environment + Send + Sync> {
        &self.environment
    }
}

impl StringValueResolver for EmbeddedValueResolver {
    fn resolve_string_value(&self, value: &str) -> String {
        // 当输入本身就是占位符（形如 ${name}）时，先尝试解析为属性值，
        // 否则当作普通字符串解析其中的占位符。
        if value.trim_start().starts_with("${") && value.trim_end().ends_with('}') {
            let inner = &value.trim()[2..value.trim().len() - 1];
            if let Some(resolved) = self.environment.get_property(inner.trim()) {
                return resolved;
            }
        }
        self.environment.resolve_placeholders(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::SimpleEnvironment;

    #[test]
    fn test_embedded_resolver_simple_placeholder() {
        let mut env = SimpleEnvironment::new();
        env.set_property("name", "vernal");
        let resolver = EmbeddedValueResolver::new(Arc::new(env));
        assert_eq!(resolver.resolve_string_value("${name}"), "vernal");
    }

    #[test]
    fn test_embedded_resolver_mixed_text() {
        let mut env = SimpleEnvironment::new();
        env.set_property("name", "vernal");
        let resolver = EmbeddedValueResolver::new(Arc::new(env));
        assert_eq!(
            resolver.resolve_string_value("Hello ${name}!"),
            "Hello vernal!"
        );
    }
}
