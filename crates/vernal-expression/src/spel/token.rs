//! Token 数据结构（对标 Spring `org.springframework.expression.spel.standard.Token`）。
//!
//! 词法单元 = 类型 + 字符串数据 + 起止位置。

use super::token_kind::TokenKind;

/// 词法单元（对标 Spring `Token`）。
#[derive(Debug, Clone)]
pub struct Token {
    /// 词法单元类型。
    pub kind: TokenKind,
    /// 关联字符串数据（标识符名/字面量值/关键字）。
    pub data: Option<String>,
    /// 起始位置（字节偏移）。
    pub start_pos: usize,
    /// 结束位置（exclusive，字节偏移）。
    pub end_pos: usize,
}

impl Token {
    /// 创建无 payload 的 token（如 `+`, `[`）。
    #[must_use]
    pub fn empty(kind: TokenKind, start_pos: usize, end_pos: usize) -> Self {
        Self {
            kind,
            data: None,
            start_pos,
            end_pos,
        }
    }

    /// 创建带 payload 的 token（标识符 / 字面量 / 关键字）。
    #[must_use]
    pub fn with_data(kind: TokenKind, data: impl Into<String>, start_pos: usize, end_pos: usize) -> Self {
        Self {
            kind,
            data: Some(data.into()),
            start_pos,
            end_pos,
        }
    }

    /// 是否为标识符 token。
    #[must_use]
    pub fn is_identifier(&self) -> bool {
        self.kind == TokenKind::Identifier
    }

    /// 是否为数字关系运算符（`> >= < <= == !=`）。
    #[must_use]
    pub fn is_numeric_relational_operator(&self) -> bool {
        self.kind.is_numeric_relational()
    }

    /// 获取字符串值（无 payload 时返回空串）。
    #[must_use]
    pub fn string_value(&self) -> String {
        self.data.clone().unwrap_or_default()
    }

    /// 转换 `matches`/`instanceof`/`between` 为对应 TokenKind，保留位置与 payload。
    ///
    /// Spring 在 relational 阶段把标识符 token 改写为对应关键字。
    #[must_use]
    pub fn as_instanceof_token(&self) -> Token {
        let mut t = self.clone();
        t.kind = TokenKind::Instanceof;
        t.data = Some("instanceof".to_string());
        t
    }

    /// 转换为 `matches` token。
    #[must_use]
    pub fn as_matches_token(&self) -> Token {
        let mut t = self.clone();
        t.kind = TokenKind::Matches;
        t.data = Some("matches".to_string());
        t
    }

    /// 转换为 `between` token。
    #[must_use]
    pub fn as_between_token(&self) -> Token {
        let mut t = self.clone();
        t.kind = TokenKind::Between;
        t.data = Some("between".to_string());
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_token() {
        let t = Token::empty(TokenKind::Plus, 0, 1);
        assert_eq!(t.kind, TokenKind::Plus);
        assert!(t.data.is_none());
        assert_eq!(t.string_value(), "");
    }

    #[test]
    fn data_token() {
        let t = Token::with_data(TokenKind::Identifier, "foo", 0, 3);
        assert!(t.is_identifier());
        assert_eq!(t.string_value(), "foo");
    }

    #[test]
    fn as_instanceof_converts_payload() {
        let t = Token::with_data(TokenKind::Identifier, "instanceof", 5, 15);
        let t2 = t.as_instanceof_token();
        assert_eq!(t2.kind, TokenKind::Instanceof);
        assert_eq!(t2.string_value(), "instanceof");
    }
}
