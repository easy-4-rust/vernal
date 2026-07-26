//! 词法分析器。
//!
//! 对标 Spring 的 `Tokenizer`。
//! 支持多字符运算符：`==`、`!=`、`<=`、`>=`、`&&`、`||`。

use super::token::Token;

/// 词法分析器。
///
/// 将表达式字符串分解为词法单元序列。
/// 对标 Spring 的 `org.springframework.expression.spel.standard.Tokenizer`。
///
/// 支持的运算符：
/// - 单字符：`+`、`-`、`*`、`/`、`%`、`^`、`(`、`)`、`[`、`]`、`{`、`}`、`,`、`.`、`:`、`#`、`@`、`!`、`=`、`<`、`>`、`?`
/// - 双字符：`==`、`!=`、`<=`、`>=`、`&&`、`||`、`++`、`--`、`?:`、`?.`、`^`、`$`
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
                // 空白字符：跳过
                ' ' | '\t' | '\n' | '\r' => {
                    self.pos += 1;
                }

                // 等号：`=` 或 `==`
                '=' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '=' {
                        tokens.push(Token::new(
                            "EQ".to_string(),
                            "==".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::new(
                            "ASSIGN".to_string(),
                            "=".to_string(),
                            start,
                            start + 1,
                        ));
                        self.pos += 1;
                    }
                }

                // 感叹号：`!` 或 `!=`
                '!' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '=' {
                        tokens.push(Token::new(
                            "NE".to_string(),
                            "!=".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::new(
                            "NOT".to_string(),
                            "!".to_string(),
                            start,
                            start + 1,
                        ));
                        self.pos += 1;
                    }
                }

                // 小于号：`<` 或 `<=`
                '<' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '=' {
                        tokens.push(Token::new(
                            "LE".to_string(),
                            "<=".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::new(
                            "LT".to_string(),
                            "<".to_string(),
                            start,
                            start + 1,
                        ));
                        self.pos += 1;
                    }
                }

                // 大于号：`>` 或 `>=`
                '>' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '=' {
                        tokens.push(Token::new(
                            "GE".to_string(),
                            ">=".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::new(
                            "GT".to_string(),
                            ">".to_string(),
                            start,
                            start + 1,
                        ));
                        self.pos += 1;
                    }
                }

                // 与号：`&&`
                '&' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '&' {
                        tokens.push(Token::new(
                            "AND".to_string(),
                            "&&".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        return Err(format!("未知字符 '&' at position {}", start));
                    }
                }

                // 或号：`||`
                '|' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '|' {
                        tokens.push(Token::new(
                            "OR".to_string(),
                            "||".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        return Err(format!("未知字符 '|' at position {}", start));
                    }
                }

                // 加号：`+` 或 `++`
                '+' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '+' {
                        tokens.push(Token::new(
                            "INC".to_string(),
                            "++".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::new(
                            "PLUS".to_string(),
                            "+".to_string(),
                            start,
                            start + 1,
                        ));
                        self.pos += 1;
                    }
                }

                // 减号：`-` 或 `--`
                '-' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == '-' {
                        tokens.push(Token::new(
                            "DEC".to_string(),
                            "--".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::new(
                            "MINUS".to_string(),
                            "-".to_string(),
                            start,
                            start + 1,
                        ));
                        self.pos += 1;
                    }
                }

                // 乘号
                '*' => {
                    tokens.push(Token::new(
                        "MULTIPLY".to_string(),
                        "*".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }

                // 除号
                '/' => {
                    tokens.push(Token::new(
                        "DIVIDE".to_string(),
                        "/".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }

                // 取模
                '%' => {
                    tokens.push(Token::new(
                        "MODULUS".to_string(),
                        "%".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }

                // 幂运算
                '^' => {
                    tokens.push(Token::new(
                        "POWER".to_string(),
                        "^".to_string(),
                        start,
                        start + 1,
                    ));
                    self.pos += 1;
                }

                // 问号：`?` 或 `?:` 或 `?.`
                '?' => {
                    if self.pos + 1 < len && chars[self.pos + 1] == ':' {
                        tokens.push(Token::new(
                            "ELVIS".to_string(),
                            "?:".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else if self.pos + 1 < len && chars[self.pos + 1] == '.' {
                        tokens.push(Token::new(
                            "SAFE_NAVI".to_string(),
                            "?.".to_string(),
                            start,
                            start + 2,
                        ));
                        self.pos += 2;
                    } else {
                        tokens.push(Token::new(
                            "QMARK".to_string(),
                            "?".to_string(),
                            start,
                            start + 1,
                        ));
                        self.pos += 1;
                    }
                }

                // 括号
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

                // 分隔符
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

                // 特殊符号
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

                // 字符串字面量
                '\'' => {
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

                // 数字字面量
                _ if ch.is_ascii_digit() => {
                    let mut num = String::new();
                    let mut has_dot = false;
                    while self.pos < len
                        && (chars[self.pos].is_ascii_digit()
                            || (chars[self.pos] == '.' && !has_dot))
                    {
                        if chars[self.pos] == '.' {
                            has_dot = true;
                        }
                        num.push(chars[self.pos]);
                        self.pos += 1;
                    }
                    // 检查长整数后缀
                    if self.pos < len && (chars[self.pos] == 'L' || chars[self.pos] == 'l') {
                        num.push(chars[self.pos]);
                        self.pos += 1;
                    }
                    let kind = if has_dot { "REAL" } else { "INT" };
                    tokens.push(Token::new(kind.to_string(), num, start, self.pos));
                }

                // 标识符
                _ if ch.is_alphabetic() || ch == '_' => {
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
