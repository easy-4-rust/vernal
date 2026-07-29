//! TypeFilter — Spring 风格的类型过滤器。
//!
//! 对应 Java 类：`org.springframework.core.type.filter.TypeFilter`。
//!
//! 在组件扫描过程中，用于判定某个类型是否应当被包含或排除。

use regex::Regex;

/// Spring 风格的类型过滤器 trait。
///
/// 对应 Spring 的 `TypeFilter` 接口。
///
/// 判定一个类全限定名是否匹配过滤条件。`ComponentScan` 使用
/// `TypeFilter` 集合来决定包含/排除哪些候选类型。
pub trait TypeFilter: Send + Sync {
    /// 判定给定类名是否匹配。
    ///
    /// # 参数
    ///
    /// * `class_name` — 类全限定名
    ///
    /// # 返回
    ///
    /// 匹配返回 `true`，否则返回 `false`。
    fn matches(&self, class_name: &str) -> bool;
}

/// 基于正则表达式的类型过滤器。
///
/// 对应 Spring 的 `org.springframework.core.type.filter.RegexPatternTypeFilter`。
///
/// 使用正则表达式匹配类全限定名。
#[derive(Debug, Clone)]
pub struct RegexTypeFilter {
    pattern: Regex,
}

impl RegexTypeFilter {
    /// 创建新的正则类型过滤器。
    ///
    /// # 错误
    ///
    /// 当提供的模式不是合法正则时返回错误。
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
        })
    }

    /// 从已编译的正则创建。
    pub fn from_regex(pattern: Regex) -> Self {
        Self { pattern }
    }

    /// 获取内部正则模式。
    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }

    /// 获取原始模式字符串。
    pub fn pattern_str(&self) -> &str {
        self.pattern.as_str()
    }
}

impl TypeFilter for RegexTypeFilter {
    fn matches(&self, class_name: &str) -> bool {
        self.pattern.is_match(class_name)
    }
}

/// 包含过滤器：仅当类名属于给定前缀集合时返回 `true`。
///
/// 用于组件扫描的默认包含策略。
#[derive(Debug, Clone)]
pub struct PrefixIncludeFilter {
    prefixes: Vec<String>,
}

impl PrefixIncludeFilter {
    /// 创建新的前缀包含过滤器。
    pub fn new<I, S>(prefixes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            prefixes: prefixes.into_iter().map(Into::into).collect(),
        }
    }

    /// 添加一个前缀。
    pub fn add(&mut self, prefix: impl Into<String>) {
        self.prefixes.push(prefix.into());
    }

    /// 当前前缀数量。
    pub fn len(&self) -> usize {
        self.prefixes.len()
    }

    /// 是否没有前缀。
    pub fn is_empty(&self) -> bool {
        self.prefixes.is_empty()
    }
}

impl TypeFilter for PrefixIncludeFilter {
    fn matches(&self, class_name: &str) -> bool {
        if self.prefixes.is_empty() {
            return true;
        }
        self.prefixes
            .iter()
            .any(|p| class_name.starts_with(p.as_str()))
    }
}

/// 取反过滤器：将内部过滤器的判定结果取反。
#[derive(Debug)]
pub struct NotFilter<F> {
    inner: F,
}

impl<F> NotFilter<F> {
    /// 创建取反过滤器。
    pub fn new(inner: F) -> Self {
        Self { inner }
    }
}

impl<F: TypeFilter> TypeFilter for NotFilter<F> {
    fn matches(&self, class_name: &str) -> bool {
        !self.inner.matches(class_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_filter() {
        let filter = RegexTypeFilter::new(r"^com\.example\..*Service$").unwrap();
        assert!(filter.matches("com.example.UserService"));
        assert!(!filter.matches("com.example.Repository"));
        assert!(!filter.matches("org.other.UserService"));
    }

    #[test]
    fn test_prefix_filter() {
        let filter = PrefixIncludeFilter::new(["com.example", "org.demo"]);
        assert!(filter.matches("com.example.Foo"));
        assert!(filter.matches("org.demo.Bar"));
        assert!(!filter.matches("net.other.Baz"));
    }

    #[test]
    fn test_not_filter() {
        let inner = PrefixIncludeFilter::new(["com.example"]);
        let not = NotFilter::new(inner);
        assert!(!not.matches("com.example.Foo"));
        assert!(not.matches("net.other.Baz"));
    }
}
