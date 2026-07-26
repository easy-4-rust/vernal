//! SpEL 解析器配置。
//!
//! 对标 Spring 的 `SpelParserConfiguration`。

/// SpEL 解析器配置。
///
/// 对标 Spring 的 `org.springframework.expression.spel.SpelParserConfiguration`。
#[derive(Debug, Clone)]
pub struct SpelParserConfiguration {
    /// 最大表达式长度
    max_expression_length: usize,
    /// 最大运算次数
    max_operations: usize,
    /// 是否自动增长空引用
    auto_grow_null_references: bool,
    /// 是否自动增长集合
    auto_grow_collections: bool,
    /// 最大自动增长大小
    maximum_auto_grow_size: usize,
}

impl SpelParserConfiguration {
    /// 创建默认配置。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取最大表达式长度。
    #[must_use]
    pub fn max_expression_length(&self) -> usize {
        self.max_expression_length
    }

    /// 获取最大运算次数。
    #[must_use]
    pub fn max_operations(&self) -> usize {
        self.max_operations
    }

    /// 是否自动增长空引用。
    #[must_use]
    pub fn auto_grow_null_references(&self) -> bool {
        self.auto_grow_null_references
    }
}

impl Default for SpelParserConfiguration {
    fn default() -> Self {
        Self {
            max_expression_length: 10_000,
            max_operations: 10_000,
            auto_grow_null_references: false,
            auto_grow_collections: false,
            maximum_auto_grow_size: 256,
        }
    }
}
