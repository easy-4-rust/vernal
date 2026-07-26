//! 词法单元。
//!
//! 对标 Spring 的 `Token`。

/// 词法单元。
///
/// 对标 Spring 的 `org.springframework.expression.spel.standard.Token`。
#[derive(Debug, Clone)]
pub struct Token {
    /// 词法单元类型
    kind: String,
    /// 原始文本
    text: String,
    /// 起始位置
    start_pos: usize,
    /// 结束位置
    end_pos: usize,
}

impl Token {
    /// 创建词法单元。
    #[must_use]
    pub fn new(kind: String, text: String, start_pos: usize, end_pos: usize) -> Self {
        Self {
            kind,
            text,
            start_pos,
            end_pos,
        }
    }

    /// 获取词法单元类型。
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// 获取原始文本。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
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
