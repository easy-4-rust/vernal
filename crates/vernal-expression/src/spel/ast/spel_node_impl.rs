//! AST 节点基类。
//!
//! 对标 Spring 的 `SpelNodeImpl`。

/// AST 节点基类。
///
/// 所有 AST 节点的公共基类。
/// 对标 Spring 的 `org.springframework.expression.spel.ast.SpelNodeImpl`。
pub struct SpelNodeImpl {
    /// 起始位置
    start_pos: usize,
    /// 结束位置
    end_pos: usize,
}

impl SpelNodeImpl {
    /// 创建 AST 节点基类。
    #[must_use]
    pub fn new(start_pos: usize, end_pos: usize) -> Self {
        Self { start_pos, end_pos }
    }

    /// 获取起始位置。
    #[must_use]
    pub fn start_position(&self) -> usize {
        self.start_pos
    }

    /// 获取结束位置。
    #[must_use]
    pub fn end_position(&self) -> usize {
        self.end_pos
    }
}
