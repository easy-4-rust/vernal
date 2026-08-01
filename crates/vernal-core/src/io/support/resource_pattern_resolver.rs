//! 资源模式解析器契约。
//!
//! 对标 Spring `org.springframework.core.io.support.ResourcePatternResolver`。

use std::io;

use crate::io::Resource;

/// 资源模式解析器契约。
///
/// 对应 Java: org.springframework.core.io.support.ResourcePatternResolver
pub trait ResourcePatternResolver: Send + Sync {
    /// 按模式解析为多个资源（如 `classpath*:*.xml`）。
    ///
    /// # 错误
    ///
    /// 模式非法或 IO 失败时返回 [`std::io::Error`]。
    fn get_resources(&self, pattern: &str) -> io::Result<Vec<Box<dyn Resource>>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SinglePatternResolver;

    impl ResourcePatternResolver for SinglePatternResolver {
        fn get_resources(&self, pattern: &str) -> io::Result<Vec<Box<dyn Resource>>> {
            if pattern.ends_with("*.txt") {
                Ok(vec![Box::new(crate::io::ByteArrayResource::new(
                    b"x".to_vec(),
                ))])
            } else {
                Ok(Vec::new())
            }
        }
    }

    #[test]
    fn resolves_matching_pattern() {
        // A 类（合同对齐）：对标 Spring 模式匹配
        let resolver = SinglePatternResolver;
        let resources = resolver.get_resources("classpath*:*.txt").unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].read_bytes().unwrap(), b"x");
    }

    #[test]
    fn unmatched_pattern_yields_empty() {
        // B 类（边界行为）
        let resolver = SinglePatternResolver;
        assert!(resolver.get_resources("*.xml").unwrap().is_empty());
    }
}
