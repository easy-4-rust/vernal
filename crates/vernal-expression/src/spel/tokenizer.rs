//! 词法分析器（对标 Spring `org.springframework.expression.spel.standard.Tokenizer`）。
//!
//! 词法单元切分；错误统一返回 `Result<Vec<Token>, InternalParseException>`，
//! 由顶层 `doParseExpression` 捕获并转换为 `SpelParseException`。

use super::internal_parse_exception::InternalParseException;
use super::spel_message::SpelMessage;
use super::spel_parse_exception::SpelParseException;
use super::token::Token;
use super::token_kind::TokenKind;

/// 替代运算符名（按字母序，对应 Spring `ALTERNATIVE_OPERATOR_NAMES`）。
const ALTERNATIVE_OPERATOR_NAMES: &[&str] = &[
    "DIV", "EQ", "GE", "GT", "LE", "LT", "MOD", "NE", "NOT",
];

/// 词法分析器（对标 Spring `Tokenizer`）。
pub struct Tokenizer<'a> {
    expression: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    /// 创建词法分析器。
    #[must_use]
    pub fn new(expression: &'a str) -> Self {
        Self {
            expression,
            pos: 0,
        }
    }

    /// 主入口：分词为完整 token 流。
    pub fn tokenize(&mut self) -> Result<Vec<Token>, InternalParseException> {
        let mut tokens: Vec<Token> = Vec::new();
        let bytes = self.expression.as_bytes();
        let len = bytes.len();

        while self.pos < len {
            let ch = bytes[self.pos];
            let start = self.pos;

            match ch {
                b' ' | b'\t' | b'\n' | b'\r' => {
                    self.pos += 1;
                }

                b'=' => {
                    if self.peek_at(1) == Some(b'=') {
                        tokens.push(Token::empty(TokenKind::Equal, start, start + 2));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::empty(TokenKind::Assign, start, start + 1));
                        self.pos += 1;
                    }
                }

                b'!' => {
                    if self.peek_at(1) == Some(b'=') {
                        tokens.push(Token::empty(TokenKind::NotEqual, start, start + 2));
                        self.pos += 2;
                    } else if self.peek_at(1) == Some(b'[') {
                        tokens.push(Token::empty(TokenKind::Project, start, start + 2));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::empty(TokenKind::Not, start, start + 1));
                        self.pos += 1;
                    }
                }

                b'<' => {
                    if self.peek_at(1) == Some(b'=') {
                        tokens.push(Token::empty(TokenKind::Le, start, start + 2));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::empty(TokenKind::Lt, start, start + 1));
                        self.pos += 1;
                    }
                }

                b'>' => {
                    if self.peek_at(1) == Some(b'=') {
                        tokens.push(Token::empty(TokenKind::Ge, start, start + 2));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::empty(TokenKind::Gt, start, start + 1));
                        self.pos += 1;
                    }
                }

                b'&' => {
                    if self.peek_at(1) == Some(b'&') {
                        tokens.push(Token::empty(TokenKind::SymbolicAnd, start, start + 2));
                        self.pos += 2;
                    } else {
                        // 单 & 表示 FactoryBean 引用
                        tokens.push(Token::empty(TokenKind::FactoryBeanRef, start, start + 1));
                        self.pos += 1;
                    }
                }

                b'|' => {
                    if self.peek_at(1) == Some(b'|') {
                        tokens.push(Token::empty(TokenKind::SymbolicOr, start, start + 2));
                        self.pos += 2;
                    } else {
                        return Err(self.error(start, SpelMessage::MissingCharacter, vec!["||".to_string()]));
                    }
                }

                b'+' => {
                    if self.peek_at(1) == Some(b'+') {
                        tokens.push(Token::empty(TokenKind::Inc, start, start + 2));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::empty(TokenKind::Plus, start, start + 1));
                        self.pos += 1;
                    }
                }

                b'-' => {
                    if self.peek_at(1) == Some(b'-') {
                        tokens.push(Token::empty(TokenKind::Dec, start, start + 2));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::empty(TokenKind::Minus, start, start + 1));
                        self.pos += 1;
                    }
                }

                b'*' => {
                    tokens.push(Token::empty(TokenKind::Star, start, start + 1));
                    self.pos += 1;
                }
                b'/' => {
                    tokens.push(Token::empty(TokenKind::Div, start, start + 1));
                    self.pos += 1;
                }
                b'%' => {
                    tokens.push(Token::empty(TokenKind::Mod, start, start + 1));
                    self.pos += 1;
                }
                b'^' => {
                    if self.peek_at(1) == Some(b'[') {
                        tokens.push(Token::empty(TokenKind::SelectFirst, start, start + 2));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::empty(TokenKind::Power, start, start + 1));
                        self.pos += 1;
                    }
                }
                b'$' => {
                    if self.peek_at(1) == Some(b'[') {
                        tokens.push(Token::empty(TokenKind::SelectLast, start, start + 2));
                        self.pos += 2;
                    } else {
                        let tok = self.lex_identifier(start, true)?;
                        tokens.push(tok);
                    }
                }
                b'?' => match self.peek_at(1) {
                    Some(b':') => {
                        tokens.push(Token::empty(TokenKind::Elvis, start, start + 2));
                        self.pos += 2;
                    }
                    Some(b'.') => {
                        tokens.push(Token::empty(TokenKind::SafeNavi, start, start + 2));
                        self.pos += 2;
                    }
                    Some(b'[') => {
                        tokens.push(Token::empty(TokenKind::Select, start, start + 2));
                        self.pos += 2;
                    }
                    _ => {
                        tokens.push(Token::empty(TokenKind::QMark, start, start + 1));
                        self.pos += 1;
                    }
                },

                b'(' => {
                    tokens.push(Token::empty(TokenKind::LParen, start, start + 1));
                    self.pos += 1;
                }
                b')' => {
                    tokens.push(Token::empty(TokenKind::RParen, start, start + 1));
                    self.pos += 1;
                }
                b'[' => {
                    tokens.push(Token::empty(TokenKind::LSquare, start, start + 1));
                    self.pos += 1;
                }
                b']' => {
                    tokens.push(Token::empty(TokenKind::RSquare, start, start + 1));
                    self.pos += 1;
                }
                b'{' => {
                    tokens.push(Token::empty(TokenKind::LCurly, start, start + 1));
                    self.pos += 1;
                }
                b'}' => {
                    tokens.push(Token::empty(TokenKind::RCurly, start, start + 1));
                    self.pos += 1;
                }
                b',' => {
                    tokens.push(Token::empty(TokenKind::Comma, start, start + 1));
                    self.pos += 1;
                }
                b'.' => {
                    tokens.push(Token::empty(TokenKind::Dot, start, start + 1));
                    self.pos += 1;
                }
                b':' => {
                    tokens.push(Token::empty(TokenKind::Colon, start, start + 1));
                    self.pos += 1;
                }
                b'#' => {
                    tokens.push(Token::empty(TokenKind::Hash, start, start + 1));
                    self.pos += 1;
                }
                b'@' => {
                    tokens.push(Token::empty(TokenKind::BeanRef, start, start + 1));
                    self.pos += 1;
                }

                b'\'' => {
                    let tok = self.lex_quoted_string(start)?;
                    tokens.push(tok);
                }
                b'"' => {
                    let tok = self.lex_double_quoted_string(start)?;
                    tokens.push(tok);
                }
                b'0'..=b'9' => {
                    let tok = self.lex_numeric_literal(start)?;
                    tokens.push(tok);
                }

                b'_' | b'a'..=b'z' | b'A'..=b'Z' => {
                    let tok = self.lex_identifier(start, false)?;
                    tokens.push(tok);
                }

                b'\\' => {
                    return Err(self.error(start, SpelMessage::UnexpectedEscapeChar, vec![]));
                }

                _ => {
                    let ch_str = (ch as char).to_string();
                    return Err(self.error(
                        start,
                        SpelMessage::UnsupportedCharacter,
                        vec![ch_str.clone(), format!("U+{:04X}", ch)],
                    ));
                }
            }

            // 每轮必须消费至少一个字节，防止新增分支遗漏推进位置后无限追加 token。
            if self.pos == start {
                return Err(self.error(start, SpelMessage::InternalError, vec![]));
            }
        }

        Ok(tokens)
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        let bytes = self.expression.as_bytes();
        bytes.get(self.pos + offset).copied()
    }

    fn error(
        &self,
        position: usize,
        code: SpelMessage,
        inserts: Vec<String>,
    ) -> InternalParseException {
        let insert_strs: Vec<&str> = inserts.iter().map(String::as_str).collect();
        let ex = SpelParseException::new(self.expression, position, code, &insert_strs);
        InternalParseException::wrap(ex)
    }

    fn lex_quoted_string(&mut self, start: usize) -> Result<Token, InternalParseException> {
        self.pos += 1; // 跳过起始 '
        let mut s = String::new();
        loop {
            match self.expression.as_bytes().get(self.pos).copied() {
                None => {
                    return Err(self.error(start, SpelMessage::NonTerminatingQuotedString, vec![]));
                }
                Some(b'\'') => {
                    if self.peek_at(1) == Some(b'\'') {
                        s.push('\'');
                        self.pos += 2;
                    } else {
                        let end = self.pos + 1;
                        self.pos = end;
                        return Ok(Token::with_data(TokenKind::LiteralString, s, start, end));
                    }
                }
                Some(c) => {
                    s.push(c as char);
                    self.pos += 1;
                }
            }
        }
    }

    fn lex_double_quoted_string(&mut self, start: usize) -> Result<Token, InternalParseException> {
        self.pos += 1;
        let mut s = String::new();
        loop {
            match self.expression.as_bytes().get(self.pos).copied() {
                None => {
                    return Err(self.error(start, SpelMessage::NonTerminatingDoubleQuotedString, vec![]));
                }
                Some(b'"') => {
                    if self.peek_at(1) == Some(b'"') {
                        s.push('"');
                        self.pos += 2;
                    } else {
                        let end = self.pos + 1;
                        self.pos = end;
                        return Ok(Token::with_data(TokenKind::LiteralString, s, start, end));
                    }
                }
                Some(c) => {
                    s.push(c as char);
                    self.pos += 1;
                }
            }
        }
    }

    fn lex_numeric_literal(&mut self, start: usize) -> Result<Token, InternalParseException> {
        // 0x 十六进制
        if self.pos + 1 < self.expression.len() && self.expression.as_bytes()[self.pos] == b'0'
            && matches!(self.expression.as_bytes()[self.pos + 1], b'x' | b'X')
        {
            self.pos += 2;
            let mut number = String::new();
            while let Some(c) = self.expression.as_bytes().get(self.pos).copied() {
                if c.is_ascii_hexdigit() {
                    number.push(c as char);
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let mut is_long = false;
            if matches!(self.expression.as_bytes().get(self.pos).copied(), Some(b'L') | Some(b'l')) {
                is_long = true;
                number.push(self.expression.as_bytes()[self.pos] as char);
                self.pos += 1;
            }
            if number.is_empty() {
                return Err(self.error(start, SpelMessage::NotAnInteger, vec!["".into()]));
            }
            let end = self.pos;
            let kind = if is_long { TokenKind::LiteralHexLong } else { TokenKind::LiteralHexInt };
            return Ok(Token::with_data(kind, number, start, end));
        }

        // 普通十进制 / 实数
        let mut number = String::new();
        let mut is_real = false;
        while let Some(c) = self.expression.as_bytes().get(self.pos).copied() {
            if c.is_ascii_digit() {
                number.push(c as char);
                self.pos += 1;
            } else if c == b'.' && !is_real {
                if matches!(self.peek_at(1), Some(d) if d.is_ascii_digit()) {
                    is_real = true;
                    number.push('.');
                    self.pos += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // 科学计数法
        if matches!(self.expression.as_bytes().get(self.pos).copied(), Some(b'e') | Some(b'E')) {
            is_real = true;
            number.push(self.expression.as_bytes()[self.pos] as char);
            self.pos += 1;
            if matches!(self.expression.as_bytes().get(self.pos).copied(), Some(b'+') | Some(b'-')) {
                number.push(self.expression.as_bytes()[self.pos] as char);
                self.pos += 1;
            }
            while let Some(c) = self.expression.as_bytes().get(self.pos).copied() {
                if c.is_ascii_digit() {
                    number.push(c as char);
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }

        // 后缀
        match self.expression.as_bytes().get(self.pos).copied() {
            Some(b'L') | Some(b'l') => {
                if is_real {
                    return Err(self.error(
                        start,
                        SpelMessage::RealCannotBeLong,
                        vec![number.clone()],
                    ));
                }
                number.push(self.expression.as_bytes()[self.pos] as char);
                self.pos += 1;
                let end = self.pos;
                Ok(Token::with_data(TokenKind::LiteralLong, number, start, end))
            }
            Some(b'F') | Some(b'f') => {
                number.push(self.expression.as_bytes()[self.pos] as char);
                self.pos += 1;
                let end = self.pos;
                Ok(Token::with_data(TokenKind::LiteralRealFloat, number, start, end))
            }
            Some(b'D') | Some(b'd') => {
                number.push(self.expression.as_bytes()[self.pos] as char);
                self.pos += 1;
                let end = self.pos;
                Ok(Token::with_data(TokenKind::LiteralReal, number, start, end))
            }
            _ => {
                let end = self.pos;
                let kind = if is_real { TokenKind::LiteralReal } else { TokenKind::LiteralInt };
                Ok(Token::with_data(kind, number, start, end))
            }
        }
    }

    fn lex_identifier(&mut self, start: usize, started_with_dollar: bool) -> Result<Token, InternalParseException> {
        let mut ident = String::new();
        if started_with_dollar {
            ident.push('$');
        }
        while let Some(c) = self.expression.as_bytes().get(self.pos).copied() {
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
                ident.push(c as char);
                self.pos += 1;
            } else {
                break;
            }
        }
        let end = self.pos;

        // 替代运算符名（按字母序二分匹配）
        if !started_with_dollar && (2..=3).contains(&ident.len()) {
            let upper = ident.to_ascii_uppercase();
            if let Ok(idx) = ALTERNATIVE_OPERATOR_NAMES.binary_search(&upper.as_str()) {
                let kind = match ALTERNATIVE_OPERATOR_NAMES[idx] {
                    "DIV" => TokenKind::Div,
                    "EQ" => TokenKind::Equal,
                    "GE" => TokenKind::Ge,
                    "GT" => TokenKind::Gt,
                    "LE" => TokenKind::Le,
                    "LT" => TokenKind::Lt,
                    "MOD" => TokenKind::Mod,
                    "NE" => TokenKind::NotEqual,
                    "NOT" => TokenKind::Not,
                    _ => unreachable!(),
                };
                return Ok(Token::empty(kind, start, end));
            }
        }

        // 普通标识符（instanceof/matches/between 由 parser 层通过 as_*_token 重映射）
        Ok(Token::with_data(TokenKind::Identifier, ident, start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(input: &str) -> Vec<Token> {
        let mut tk = Tokenizer::new(input);
        tk.tokenize().unwrap_or_else(|e| panic!("tokenize failed for {input:?}: {e:?}"))
    }

    fn kinds(input: &str) -> Vec<TokenKind> {
        t(input).iter().map(|tk| tk.kind).collect()
    }

    #[test]
    fn integers() {
        assert_eq!(kinds("42"), vec![TokenKind::LiteralInt]);
        assert_eq!(kinds("42L"), vec![TokenKind::LiteralLong]);
    }

    #[test]
    fn hex() {
        assert_eq!(t("0x1A")[0].kind, TokenKind::LiteralHexInt);
        assert_eq!(t("0x1AL")[0].kind, TokenKind::LiteralHexLong);
    }

    #[test]
    fn real() {
        assert_eq!(kinds("3.14"), vec![TokenKind::LiteralReal]);
        assert_eq!(kinds("3.14F"), vec![TokenKind::LiteralRealFloat]);
        assert_eq!(kinds("3.14D"), vec![TokenKind::LiteralReal]);
        assert_eq!(kinds("6.022e23"), vec![TokenKind::LiteralReal]);
    }

    #[test]
    fn strings() {
        assert_eq!(kinds("'hello'"), vec![TokenKind::LiteralString]);
        assert_eq!(t("'it''s'")[0].string_value(), "it's");
        assert_eq!(t(r#""hello""#)[0].string_value(), "hello");
    }

    #[test]
    fn operators() {
        assert_eq!(kinds("=="), vec![TokenKind::Equal]);
        assert_eq!(kinds("!="), vec![TokenKind::NotEqual]);
        assert_eq!(kinds("<="), vec![TokenKind::Le]);
        assert_eq!(kinds(">="), vec![TokenKind::Ge]);
        assert_eq!(kinds("++"), vec![TokenKind::Inc]);
        assert_eq!(kinds("--"), vec![TokenKind::Dec]);
        assert_eq!(kinds("&&"), vec![TokenKind::SymbolicAnd]);
        assert_eq!(kinds("||"), vec![TokenKind::SymbolicOr]);
        assert_eq!(kinds("?:"), vec![TokenKind::Elvis]);
        assert_eq!(kinds("?."), vec![TokenKind::SafeNavi]);
        assert_eq!(kinds("?["), vec![TokenKind::Select]);
        assert_eq!(kinds("!["), vec![TokenKind::Project]);
        assert_eq!(kinds("^["), vec![TokenKind::SelectFirst]);
        assert_eq!(kinds("$["), vec![TokenKind::SelectLast]);
    }

    #[test]
    fn single_character_tokens_advance() {
        for (input, expected) in [
            ("*", TokenKind::Star),
            ("/", TokenKind::Div),
            ("%", TokenKind::Mod),
            ("(", TokenKind::LParen),
            (")", TokenKind::RParen),
            ("[", TokenKind::LSquare),
            ("]", TokenKind::RSquare),
            ("{", TokenKind::LCurly),
            ("}", TokenKind::RCurly),
            (",", TokenKind::Comma),
            (".", TokenKind::Dot),
            (":", TokenKind::Colon),
            ("#", TokenKind::Hash),
            ("@", TokenKind::BeanRef),
        ] {
            assert_eq!(kinds(input), vec![expected], "input: {input}");
        }
    }

    #[test]
    fn alternative_operator_names() {
        assert_eq!(kinds("eq"), vec![TokenKind::Equal]);
        assert_eq!(kinds("ne"), vec![TokenKind::NotEqual]);
        assert_eq!(kinds("div"), vec![TokenKind::Div]);
        assert_eq!(kinds("mod"), vec![TokenKind::Mod]);
        assert_eq!(kinds("not"), vec![TokenKind::Not]);
    }

    #[test]
    fn identifiers_and_keywords() {
        assert_eq!(kinds("foo"), vec![TokenKind::Identifier]);
        let toks = t("foo.bar");
        assert_eq!(toks[0].string_value(), "foo");
        assert_eq!(toks[1].kind, TokenKind::Dot);
        assert_eq!(toks[2].string_value(), "bar");
    }

    #[test]
    fn bean_and_factorybean_refs() {
        assert_eq!(kinds("@bean"), vec![TokenKind::BeanRef, TokenKind::Identifier]);
        assert_eq!(kinds("&factory"), vec![TokenKind::FactoryBeanRef, TokenKind::Identifier]);
    }
}
