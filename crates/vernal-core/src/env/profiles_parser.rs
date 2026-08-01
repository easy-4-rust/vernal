//! Profile 表达式解析器。
//!
//! 对标 Spring `org.springframework.core.env.ProfilesParser`。

use super::profiles::{Profiles, SingleProfile};

/// Profile 表达式解析器。
///
/// 对应 Java: org.springframework.core.env.ProfilesParser
///
/// Spring 语义：解析 `dev & prod`（AND）、`dev | test`（OR）、`!test`（NOT）、
/// `(dev & prod) | test`（括号）表达式为 [`Profiles`] 匹配器。
pub struct ProfilesParser;

/// 组合匹配器（表达式内部节点）。
enum ProfileExpr {
    And(Box<ProfileExpr>, Box<ProfileExpr>),
    Or(Box<ProfileExpr>, Box<ProfileExpr>),
    Not(Box<ProfileExpr>),
    Name(String),
}

impl ProfileExpr {
    fn eval(&self, active: &[&str]) -> bool {
        match self {
            ProfileExpr::And(l, r) => l.eval(active) && r.eval(active),
            ProfileExpr::Or(l, r) => l.eval(active) || r.eval(active),
            ProfileExpr::Not(inner) => !inner.eval(active),
            ProfileExpr::Name(name) => active.iter().any(|p| *p == name),
        }
    }
}

/// 表达式 AST 的 [`Profiles`] 适配器。
struct ExpressionProfiles {
    expr: ProfileExpr,
}

impl Profiles for ExpressionProfiles {
    fn matches(&self, active_profiles: &[&str]) -> bool {
        self.expr.eval(active_profiles)
    }
}

impl ProfilesParser {
    /// 解析 profile 表达式。
    ///
    /// # 错误
    ///
    /// 语法非法（括号不匹配、非法字符、空表达式）时返回错误文本。
    pub fn parse(expression: &str) -> Result<Box<dyn Profiles>, String> {
        let mut cursor = Cursor::new(expression);
        let expr = parse_or(&mut cursor)?;
        cursor.skip_whitespace();
        if !cursor.is_end() {
            return Err(format!("非法字符 '{}'", cursor.peek_char().unwrap_or('?')));
        }
        Ok(Box::new(ExpressionProfiles { expr }))
    }
}

/// 递归下降游标。
struct Cursor<'a> {
    chars: Vec<char>,
    pos: usize,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            chars: text.chars().collect(),
            pos: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn is_end(&self) -> bool {
        self.pos >= self.chars.len()
    }
}

/// 解析 `or` 层：`term ('|' term)*`。
fn parse_or(cursor: &mut Cursor<'_>) -> Result<ProfileExpr, String> {
    let mut left = parse_and(cursor)?;
    loop {
        cursor.skip_whitespace();
        if cursor.peek_char() == Some('|') {
            cursor.pos += 1;
            let right = parse_and(cursor)?;
            left = ProfileExpr::Or(Box::new(left), Box::new(right));
        } else {
            break;
        }
    }
    Ok(left)
}

/// 解析 `and` 层：`factor ('&' factor)*`。
fn parse_and(cursor: &mut Cursor<'_>) -> Result<ProfileExpr, String> {
    let mut left = parse_factor(cursor)?;
    loop {
        cursor.skip_whitespace();
        if cursor.peek_char() == Some('&') {
            cursor.pos += 1;
            let right = parse_factor(cursor)?;
            left = ProfileExpr::And(Box::new(left), Box::new(right));
        } else {
            break;
        }
    }
    Ok(left)
}

/// 解析因子：`!factor` | `(expr)` | profile 名。
fn parse_factor(cursor: &mut Cursor<'_>) -> Result<ProfileExpr, String> {
    cursor.skip_whitespace();
    match cursor.peek_char() {
        Some('!') => {
            cursor.pos += 1;
            Ok(ProfileExpr::Not(Box::new(parse_factor(cursor)?)))
        }
        Some('(') => {
            cursor.pos += 1;
            let inner = parse_or(cursor)?;
            cursor.skip_whitespace();
            if cursor.peek_char() != Some(')') {
                return Err("缺少右括号 ')'".to_string());
            }
            cursor.pos += 1;
            Ok(inner)
        }
        Some(c) if is_name_char(c) => {
            let mut name = String::new();
            while let Some(c) = cursor.peek_char() {
                if is_name_char(c) {
                    name.push(c);
                    cursor.pos += 1;
                } else {
                    break;
                }
            }
            Ok(ProfileExpr::Name(name))
        }
        _ => Err("空表达式或非法字符".to_string()),
    }
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-'
}

/// 便捷 API：从表达式构造匹配器（对标 Spring `Profiles.of(...)` 形态）。
///
/// # 错误
///
/// 语法非法时返回错误文本。
pub fn of(expression: &str) -> Result<Box<dyn Profiles>, String> {
    ProfilesParser::parse(expression)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(expression: &str, active: &[&str]) -> bool {
        ProfilesParser::parse(expression).unwrap().matches(active)
    }

    #[test]
    fn single_name() {
        // A 类（合同对齐）：对标 Spring `Profiles.of("dev")`
        assert!(matches("dev", &["dev"]));
        assert!(!matches("dev", &["prod"]));
    }

    #[test]
    fn and_expression() {
        // A 类（合同对齐）：对标 Spring `"dev & cloud"`
        assert!(matches("dev & cloud", &["dev", "cloud"]));
        assert!(!matches("dev & cloud", &["dev"]));
    }

    #[test]
    fn or_expression() {
        // A 类（合同对齐）：对标 Spring `"dev | test"`
        assert!(matches("dev | test", &["test"]));
        assert!(!matches("dev | test", &["prod"]));
    }

    #[test]
    fn not_expression() {
        // A 类（合同对齐）：对标 Spring `"!prod"`
        assert!(matches("!prod", &["dev"]));
        assert!(!matches("!prod", &["prod"]));
    }

    #[test]
    fn parenthesized_expression() {
        // B 类（边界行为）：对标 Spring `"(dev & cloud) | test"`
        assert!(matches("(dev & cloud) | test", &["test"]));
        assert!(matches("(dev & cloud) | test", &["dev", "cloud"]));
        assert!(!matches("(dev & cloud) | test", &["dev"]));
    }

    #[test]
    fn invalid_expression_returns_error() {
        // C 类（错误路径）：对标 Spring `IllegalArgumentException`
        assert!(ProfilesParser::parse("dev &").is_err());
        assert!(ProfilesParser::parse("(dev").is_err());
        assert!(ProfilesParser::parse("").is_err());
    }

    #[test]
    fn of_helper_matches() {
        // D 类（重构安全）：便捷 API 与解析器等价
        let profiles = of("dev | prod").unwrap();
        assert!(profiles.matches(&["prod"]));
    }
}
