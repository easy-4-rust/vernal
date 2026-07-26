//! 词法分析器。
//!
//! 对标 Spring 的 `Tokenizer`。

use super::token::Token;

/// 词法分析器。
///
/// 将表达式字符串分解为词法单元序列。
/// 对标 Spring 的 `org.springframework.expression.spel.standard.Tokenizer`。
pub struct Tokenizer {
    expression: String,
    pos: usize,
}

impl Tokenizer {
    /// 创建词法分析器。
    #[must_use]
    pub fn new(expression: String) -> Self {
        Self { expression, pos: 0 }
    }

    /// 词法分析，返回词法单元序列。
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = self.expression.chars().collect();
        let len = chars.len();

        while self.pos < len {
            let ch = chars[self.pos];
            let start = self.pos;

            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    self.pos += 1;
                }
                '+' => {
                    tokens.push(Token::new(
                        "PLUS".to_string(),
                        "+".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '-' => {
                    tokens.push(Token::new(
                        "MINUS".to_string(),
                        "-".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '*' => {
                    tokens.push(Token::new(
                        "MULTIPLY".to_string(),
                        "*".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '/' => {
                    tokens.push(Token::new(
                        "DIVIDE".to_string(),
                        "/".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '%' => {
                    tokens.push(Token::new(
                        "MODULUS".to_string(),
                        "%".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '^' => {
                    tokens.push(Token::new(
                        "POWER".to_string(),
                        "^".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '(' => {
                    tokens.push(Token::new(
                        "LPAREN".to_string(),
                        "(".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                ')' => {
                    tokens.push(Token::new(
                        "RPAREN".to_string(),
                        ")".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '[' => {
                    tokens.push(Token::new(
                        "LSQUARE".to_string(),
                        "[".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                ']' => {
                    tokens.push(Token::new(
                        "RSQUARE".to_string(),
                        "]".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '{' => {
                    tokens.push(Token::new(
                        "LCURLY".to_string(),
                        "{".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '}' => {
                    tokens.push(Token::new(
                        "RCURLY".to_string(),
                        "}".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                ',' => {
                    tokens.push(Token::new(
                        "COMMA".to_string(),
                        ",".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '.' => {
                    tokens.push(Token::new(
                        "DOT".to_string(),
                        ".".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                ':' => {
                    tokens.push(Token::new(
                        "COLON".to_string(),
                        ":".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '#' => {
                    tokens.push(Token::new(
                        "HASH".to_string(),
                        "#".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '@' => {
                    tokens.push(Token::new(
                        "AT".to_string(),
                        "@".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }
                '\'' => {
                    // 字符串字面量
                    self.pos += 1;
                    let mut s = String::new();
                    while self.pos < len && chars[self.pos] != '\'' {
                        s.push(chars[self.pos]);
                        self.pos += 1;
                    }
                    if self.pos < len {
                        self.pos += 1; // 跳过结束引号
                    }
                    tokens.push(Token::new("STRING".to_string(), s, start, self.pos));
                }
                _ if ch.is_ascii_digit() => {
                    // 数字字面量
                    let mut num = String::new();
                    while self.pos < len
                        && (chars[self.pos].is_ascii_digit() || chars[self.pos] == '.')
                    {
                        num.push(chars[self.pos]);
                        self.pos += 1;
                    }
                    let kind = if num.contains('.') { "REAL" } else { "INT" };
                    tokens.push(Token::new(kind.to_string(), num, start, self.pos));
                }
                _ if ch.is_alphabetic() || ch == '_' => {
                    // 标识符
                    let mut ident = String::new();
                    while self.pos < len
                        && (chars[self.pos].is_alphanumeric() || chars[self.pos] == '_')
                    {
                        ident.push(chars[self.pos]);
                        self.pos += 1;
                    }
                    tokens.push(Token::new("IDENTIFIER".to_string(), ident, start, self.pos));
                }
                _ => {
                    return Err(format!("未知字符 '{}' at position {}", ch, start));
                }
            }
        }

        Ok(tokens)
    }
}
