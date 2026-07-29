//! StringValueResolver — Spring 风格的字符串值解析器 trait。
//!
//! 对应 Java 类：`org.springframework.util.StringValueResolver`。
//!
//! 将一个字符串值解析为另一个字符串（例如占位符解析、表达式求值）。

/// Spring 风格的字符串值解析器 trait。
///
/// 对应 Spring 的 `StringValueResolver`。
///
/// 实现此 trait 的类型将输入字符串解析为输出字符串。
/// 典型用途是 `EmbeddedValueResolver`（基于 `Environment` 解析占位符）。
pub trait StringValueResolver: Send + Sync {
    /// 解析给定的字符串值。
    ///
    /// 对应 Spring 的 `StringValueResolver.resolveStringValue(String strVal)`。
    ///
    /// # 参数
    ///
    /// * `value` — 待解析的字符串（可能含占位符或表达式）
    ///
    /// # 返回
    ///
    /// 解析后的字符串。解析失败时通常原样返回或返回 `Err`。
    fn resolve_string_value(&self, value: &str) -> String;
}

impl<F> StringValueResolver for F
where
    F: Fn(&str) -> String + Send + Sync,
{
    fn resolve_string_value(&self, value: &str) -> String {
        (self)(value)
    }
}

/// 恒等解析器：原样返回输入字符串。
#[derive(Debug, Default, Clone, Copy)]
pub struct IdentityStringValueResolver;

impl StringValueResolver for IdentityStringValueResolver {
    fn resolve_string_value(&self, value: &str) -> String {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_resolver() {
        let resolver = IdentityStringValueResolver;
        assert_eq!(resolver.resolve_string_value("hello"), "hello");
    }

    #[test]
    fn test_closure_resolver() {
        let resolver = |v: &str| format!("[{v}]") as String;
        assert_eq!(resolver.resolve_string_value("x"), "[x]");
    }
}
