//! 对应 aspect-rs：aspect-core/src/pointcut/parser.rs（parse_pointcut）
//! 语义参照 spring-aop：AspectJ Weaver 解析器
//!
//! 切点表达式解析器（改名 parse_pointcut → parse_pointcut_expr）。
//! 移植自 aspect-rs 的 parser，扩展 tag()/qualifier() 语法。

use super::ast::PointcutExpr;
use super::matcher::{QualifierPattern, TagPattern};
use super::pattern::{ExecutionPattern, ModulePattern, NamePattern, Visibility};

/// 切点表达式解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointcutParseError {
    /// 错误描述。
    pub message: String,
    /// 可选的错误位置。
    pub position: Option<usize>,
}

impl std::fmt::Display for PointcutParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.position {
            Some(pos) => write!(f, "Pointcut parse error at {}: {}", pos, self.message),
            None => write!(f, "Pointcut parse error: {}", self.message),
        }
    }
}

impl std::error::Error for PointcutParseError {}

/// 解析切点表达式字符串。
///
/// 对应 aspect-rs `parse_pointcut`（改名 `parse_pointcut_expr`）。
///
/// # 支持的语法
///
/// | 语法 | 示例 |
/// |---|---|
/// | `execution(pattern)` | `execution(pub fn *(..))` |
/// | `within(module)` | `within(crate::api)` |
/// | `tag(name)` | `tag(secured)` |
/// | `qualifier(name)` | `qualifier(primary)` |
/// | `&&` | `execution(...) && within(...)` |
/// | `\|\|` | `execution(save) \|\| execution(update)` |
/// | `!` | `!within(crate::internal)` |
/// | `()` 分组 | `(execution(a) \|\| execution(b)) && within(api)` |
pub fn parse_pointcut_expr(input: &str) -> Result<PointcutExpr, PointcutParseError> {
    let input = input.trim();

    // 处理括号分组
    if input.starts_with('(') && input.ends_with(')') {
        if let Some(inner) = strip_outer_parens(input) {
            return parse_pointcut_expr(inner);
        }
    }

    // 处理 NOT 运算符（最高优先级）
    if input.starts_with('!') {
        let inner = parse_pointcut_expr(input[1..].trim())?;
        return Ok(PointcutExpr::Not(Box::new(inner)));
    }

    // 处理 OR 运算符（最低优先级）
    if let Some(or_pos) = find_operator(input, " || ") {
        let left = parse_pointcut_expr(&input[..or_pos])?;
        let right = parse_pointcut_expr(&input[or_pos + 4..])?;
        return Ok(PointcutExpr::Or(Box::new(left), Box::new(right)));
    }

    // 处理 AND 运算符（中等优先级）
    if let Some(and_pos) = find_operator(input, " && ") {
        let left = parse_pointcut_expr(&input[..and_pos])?;
        let right = parse_pointcut_expr(&input[and_pos + 4..])?;
        return Ok(PointcutExpr::And(Box::new(left), Box::new(right)));
    }

    // 解析基本切点
    if input.starts_with("execution(") {
        parse_execution(input)
    } else if input.starts_with("within(") {
        parse_within(input)
    } else if input.starts_with("tag(") {
        parse_tag(input)
    } else if input.starts_with("qualifier(") {
        parse_qualifier(input)
    } else {
        Err(PointcutParseError {
            message: format!("Unknown pointcut type: {input}"),
            position: None,
        })
    }
}

/// 去除外层平衡括号。
fn strip_outer_parens(input: &str) -> Option<&str> {
    if !input.starts_with('(') || !input.ends_with(')') {
        return None;
    }

    let inner = &input[1..input.len() - 1];
    let mut depth = 0;
    for ch in inner.chars() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            _ => {}
        }
    }

    if depth == 0 { Some(inner) } else { None }
}

/// 在括号外查找运算符位置。
fn find_operator(input: &str, operator: &str) -> Option<usize> {
    let mut depth = 0;
    let op_len = operator.len();
    let chars: Vec<char> = input.chars().collect();

    for i in 0..chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {
                if depth == 0 && i + op_len <= chars.len() {
                    let slice: String = chars[i..i + op_len].iter().collect();
                    if slice == operator {
                        return Some(i);
                    }
                }
            }
        }
    }

    None
}

/// 解析 execution 切点：`execution(pub fn save(..))`
fn parse_execution(input: &str) -> Result<PointcutExpr, PointcutParseError> {
    if !input.starts_with("execution(") || !input.ends_with(')') {
        return Err(PointcutParseError {
            message: "Invalid execution syntax".to_string(),
            position: None,
        });
    }

    let content = &input[10..input.len() - 1].trim();

    // 解析可见性
    let (visibility, rest) = parse_visibility(content);

    // 期望 "fn" 关键字
    let rest = rest.trim();
    if !rest.starts_with("fn ") {
        return Err(PointcutParseError {
            message: "Expected 'fn' keyword".to_string(),
            position: None,
        });
    }
    let rest = &rest[3..].trim();

    // 解析函数名模式
    let name = if let Some(paren_pos) = rest.find('(') {
        &rest[..paren_pos].trim()
    } else {
        return Err(PointcutParseError {
            message: "Expected function signature".to_string(),
            position: None,
        });
    };

    let name_pattern = parse_name_pattern(name);

    Ok(PointcutExpr::Execution(ExecutionPattern {
        visibility,
        name: name_pattern,
        return_type: None,
    }))
}

/// 解析 within 切点：`within(crate::api)`
fn parse_within(input: &str) -> Result<PointcutExpr, PointcutParseError> {
    if !input.starts_with("within(") || !input.ends_with(')') {
        return Err(PointcutParseError {
            message: "Invalid within syntax".to_string(),
            position: None,
        });
    }

    let module_path = input[7..input.len() - 1].trim();

    Ok(PointcutExpr::Within(ModulePattern {
        path: module_path.to_string(),
    }))
}

/// 解析 tag 切点：`tag(secured)`
fn parse_tag(input: &str) -> Result<PointcutExpr, PointcutParseError> {
    if !input.starts_with("tag(") || !input.ends_with(')') {
        return Err(PointcutParseError {
            message: "Invalid tag syntax".to_string(),
            position: None,
        });
    }

    let tag_name = input[4..input.len() - 1].trim();
    if tag_name.is_empty() {
        return Err(PointcutParseError {
            message: "Tag name cannot be empty".to_string(),
            position: None,
        });
    }

    Ok(PointcutExpr::Tag(TagPattern {
        tags: vec![tag_name.to_string()],
    }))
}

/// 解析 qualifier 切点：`qualifier(primary)`
fn parse_qualifier(input: &str) -> Result<PointcutExpr, PointcutParseError> {
    if !input.starts_with("qualifier(") || !input.ends_with(')') {
        return Err(PointcutParseError {
            message: "Invalid qualifier syntax".to_string(),
            position: None,
        });
    }

    let qualifier_name = input[10..input.len() - 1].trim();
    if qualifier_name.is_empty() {
        return Err(PointcutParseError {
            message: "Qualifier name cannot be empty".to_string(),
            position: None,
        });
    }

    Ok(PointcutExpr::Qualifier(QualifierPattern {
        qualifier: qualifier_name.to_string(),
    }))
}

/// 从字符串开头解析可见性。
fn parse_visibility(input: &str) -> (Option<Visibility>, &str) {
    if input.starts_with("pub(crate) ") {
        (Some(Visibility::Crate), &input[11..])
    } else if input.starts_with("pub(super) ") {
        (Some(Visibility::Super), &input[11..])
    } else if input.starts_with("pub ") {
        (Some(Visibility::Public), &input[4..])
    } else {
        (None, input)
    }
}

/// 解析函数名模式（精确、通配符、前缀、后缀、包含）。
fn parse_name_pattern(name: &str) -> NamePattern {
    if name == "*" {
        NamePattern::Wildcard
    } else if name.starts_with('*') && name.ends_with('*') && name.len() > 2 {
        NamePattern::Contains(name[1..name.len() - 1].to_string())
    } else if name.starts_with('*') {
        NamePattern::Suffix(name[1..].to_string())
    } else if name.ends_with('*') {
        NamePattern::Prefix(name[..name.len() - 1].to_string())
    } else {
        NamePattern::Exact(name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_execution_wildcard() {
        let pc = parse_pointcut_expr("execution(pub fn *(..))").unwrap();
        match pc {
            PointcutExpr::Execution(pattern) => {
                assert_eq!(pattern.visibility, Some(Visibility::Public));
                assert_eq!(pattern.name, NamePattern::Wildcard);
            }
            _ => panic!("Expected Execution pointcut"),
        }
    }

    #[test]
    fn parse_execution_exact_name() {
        let pc = parse_pointcut_expr("execution(fn save_user(..))").unwrap();
        match pc {
            PointcutExpr::Execution(pattern) => {
                assert_eq!(pattern.visibility, None);
                assert_eq!(pattern.name, NamePattern::Exact("save_user".to_string()));
            }
            _ => panic!("Expected Execution pointcut"),
        }
    }

    #[test]
    fn parse_execution_prefix() {
        let pc = parse_pointcut_expr("execution(pub fn save*(..))").unwrap();
        match pc {
            PointcutExpr::Execution(pattern) => {
                assert_eq!(pattern.name, NamePattern::Prefix("save".to_string()));
            }
            _ => panic!("Expected Execution pointcut"),
        }
    }

    #[test]
    fn parse_execution_suffix() {
        let pc = parse_pointcut_expr("execution(fn *_user(..))").unwrap();
        match pc {
            PointcutExpr::Execution(pattern) => {
                assert_eq!(pattern.name, NamePattern::Suffix("_user".to_string()));
            }
            _ => panic!("Expected Execution pointcut"),
        }
    }

    #[test]
    fn parse_execution_contains() {
        let pc = parse_pointcut_expr("execution(fn *save*(..))").unwrap();
        match pc {
            PointcutExpr::Execution(pattern) => {
                assert_eq!(pattern.name, NamePattern::Contains("save".to_string()));
            }
            _ => panic!("Expected Execution pointcut"),
        }
    }

    #[test]
    fn parse_execution_pub_crate() {
        let pc = parse_pointcut_expr("execution(pub(crate) fn save(..))").unwrap();
        match pc {
            PointcutExpr::Execution(pattern) => {
                assert_eq!(pattern.visibility, Some(Visibility::Crate));
            }
            _ => panic!("Expected Execution pointcut"),
        }
    }

    #[test]
    fn parse_within() {
        let pc = parse_pointcut_expr("within(crate::api)").unwrap();
        match pc {
            PointcutExpr::Within(pattern) => {
                assert_eq!(pattern.path, "crate::api");
            }
            _ => panic!("Expected Within pointcut"),
        }
    }

    #[test]
    fn parse_tag() {
        let pc = parse_pointcut_expr("tag(secured)").unwrap();
        match pc {
            PointcutExpr::Tag(pattern) => {
                assert_eq!(pattern.tags, vec!["secured".to_string()]);
            }
            _ => panic!("Expected Tag pointcut"),
        }
    }

    #[test]
    fn parse_qualifier() {
        let pc = parse_pointcut_expr("qualifier(primary)").unwrap();
        match pc {
            PointcutExpr::Qualifier(pattern) => {
                assert_eq!(pattern.qualifier, "primary".to_string());
            }
            _ => panic!("Expected Qualifier pointcut"),
        }
    }

    #[test]
    fn parse_and() {
        let pc = parse_pointcut_expr("execution(pub fn *(..)) && within(crate::api)").unwrap();
        match pc {
            PointcutExpr::And(left, right) => {
                assert!(matches!(*left, PointcutExpr::Execution(_)));
                assert!(matches!(*right, PointcutExpr::Within(_)));
            }
            _ => panic!("Expected And pointcut"),
        }
    }

    #[test]
    fn parse_or() {
        let pc = parse_pointcut_expr("execution(fn save(..)) || execution(fn update(..))").unwrap();
        match pc {
            PointcutExpr::Or(_, _) => {}
            _ => panic!("Expected Or pointcut"),
        }
    }

    #[test]
    fn parse_not() {
        let pc = parse_pointcut_expr("!within(crate::internal)").unwrap();
        match pc {
            PointcutExpr::Not(inner) => {
                assert!(matches!(*inner, PointcutExpr::Within(_)));
            }
            _ => panic!("Expected Not pointcut"),
        }
    }

    #[test]
    fn parse_parentheses() {
        let pc = parse_pointcut_expr("(execution(pub fn *(..)))").unwrap();
        assert!(matches!(pc, PointcutExpr::Execution(_)));
    }

    #[test]
    fn parse_complex_with_parentheses() {
        let pc = parse_pointcut_expr(
            "(execution(pub fn *(..)) || within(crate::admin)) && within(crate::api)",
        )
        .unwrap();

        match pc {
            PointcutExpr::And(left, right) => {
                assert!(matches!(*left, PointcutExpr::Or(_, _)));
                assert!(matches!(*right, PointcutExpr::Within(_)));
            }
            _ => panic!("Expected And with Or on left"),
        }
    }

    #[test]
    fn parse_operator_precedence() {
        // Without parens: A || B && C should parse as A || (B && C)
        let pc =
            parse_pointcut_expr("execution(fn a(..)) || execution(fn b(..)) && within(crate::api)")
                .unwrap();

        match pc {
            PointcutExpr::Or(left, right) => {
                assert!(matches!(*left, PointcutExpr::Execution(_)));
                assert!(matches!(*right, PointcutExpr::And(_, _)));
            }
            _ => panic!("Expected Or with And on right"),
        }
    }

    #[test]
    fn parse_nested_parentheses() {
        let pc = parse_pointcut_expr("((execution(pub fn *(..))))").unwrap();
        assert!(matches!(pc, PointcutExpr::Execution(_)));
    }

    #[test]
    fn parse_tag_with_and() {
        let pc = parse_pointcut_expr("execution(pub fn *(..)) && tag(secured)").unwrap();
        match pc {
            PointcutExpr::And(left, right) => {
                assert!(matches!(*left, PointcutExpr::Execution(_)));
                assert!(matches!(*right, PointcutExpr::Tag(_)));
            }
            _ => panic!("Expected And pointcut"),
        }
    }

    #[test]
    fn parse_qualifier_with_or() {
        let pc = parse_pointcut_expr("qualifier(primary) || qualifier(secondary)").unwrap();
        match pc {
            PointcutExpr::Or(left, right) => {
                assert!(matches!(*left, PointcutExpr::Qualifier(_)));
                assert!(matches!(*right, PointcutExpr::Qualifier(_)));
            }
            _ => panic!("Expected Or pointcut"),
        }
    }

    #[test]
    fn parse_error_unknown_type() {
        let result = parse_pointcut_expr("unknown(expr)");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_empty_tag() {
        let result = parse_pointcut_expr("tag()");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_empty_qualifier() {
        let result = parse_pointcut_expr("qualifier()");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_missing_fn_keyword() {
        let result = parse_pointcut_expr("execution(pub save(..))");
        assert!(result.is_err());
    }
}
