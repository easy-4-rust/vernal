//! 对应 aspect-rs：aspect-core/src/pointcut/ast.rs（Pointcut enum）
//! 语义参照 spring-aop：AspectJ 表达式 AST
//!
//! 切点表达式抽象语法树（改名 Pointcut → PointcutExpr，扩展 Tag/Qualifier）。
//! 移植自 aspect-rs 的 Pointcut enum，改名避免与现有 Pointcut trait 冲突。

use super::matcher::{QualifierPattern, TagPattern};
use super::pattern::{ExecutionPattern, ModulePattern};

/// 切点表达式（Rust 原生 DSL）。
///
/// 对应 aspect-rs `Pointcut` enum（改名 `PointcutExpr`），
/// 扩展 `Tag`/`Qualifier` 变体以对接 vernal-aop 现有切点。
///
/// # 示例
///
/// ```rust
/// use vernal_aop::pointcut::dsl::{PointcutExpr, parse_pointcut_expr};
///
/// let pc = parse_pointcut_expr("execution(pub fn *(..))").unwrap();
/// let pc = parse_pointcut_expr("within(crate::api)").unwrap();
/// let pc = parse_pointcut_expr("execution(pub fn *(..)) && within(crate::api)").unwrap();
/// let pc = parse_pointcut_expr("tag(secured)").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum PointcutExpr {
    /// 匹配函数执行：`execution(pub fn save(..))`
    Execution(ExecutionPattern),

    /// 匹配模块内函数：`within(crate::api)`
    Within(ModulePattern),

    /// 逻辑与：两个切点都必须匹配
    And(Box<PointcutExpr>, Box<PointcutExpr>),

    /// 逻辑或：任一切点匹配即可
    Or(Box<PointcutExpr>, Box<PointcutExpr>),

    /// 逻辑非：切点必须不匹配
    Not(Box<PointcutExpr>),

    /// 标签匹配（vernal 扩展，aspect-rs 无对偶）：`tag(secured)`
    Tag(TagPattern),

    /// 限定符匹配（vernal 扩展，aspect-rs 无对偶）：`qualifier(primary)`
    Qualifier(QualifierPattern),
}

impl PointcutExpr {
    /// 从字符串解析切点表达式。
    pub fn parse(input: &str) -> Result<Self, super::parser::PointcutParseError> {
        super::parser::parse_pointcut_expr(input)
    }

    /// 创建逻辑与组合。
    pub fn and(self, other: PointcutExpr) -> Self {
        PointcutExpr::And(Box::new(self), Box::new(other))
    }

    /// 创建逻辑或组合。
    pub fn or(self, other: PointcutExpr) -> Self {
        PointcutExpr::Or(Box::new(self), Box::new(other))
    }

    /// 创建逻辑非。
    pub fn not(self) -> Self {
        PointcutExpr::Not(Box::new(self))
    }

    /// 便捷方法：创建匹配所有公开函数的切点。
    pub fn public_functions() -> Self {
        PointcutExpr::Execution(ExecutionPattern::public())
    }

    /// 便捷方法：创建匹配所有函数的切点。
    pub fn all_functions() -> Self {
        PointcutExpr::Execution(ExecutionPattern::any())
    }

    /// 便捷方法：创建匹配指定模块的切点。
    pub fn within_module(module_path: impl Into<String>) -> Self {
        PointcutExpr::Within(ModulePattern::new(module_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointcut::dsl::pattern::{NamePattern, Visibility};

    #[test]
    fn pointcut_expr_combinators() {
        let pc1 = PointcutExpr::Execution(ExecutionPattern {
            visibility: Some(Visibility::Public),
            name: NamePattern::Wildcard,
            return_type: None,
        });

        let pc2 = PointcutExpr::Within(ModulePattern {
            path: "crate::api".to_string(),
        });

        let and_pc = pc1.clone().and(pc2.clone());
        assert!(matches!(and_pc, PointcutExpr::And(_, _)));

        let or_pc = pc1.clone().or(pc2.clone());
        assert!(matches!(or_pc, PointcutExpr::Or(_, _)));

        let not_pc = pc1.not();
        assert!(matches!(not_pc, PointcutExpr::Not(_)));
    }

    #[test]
    fn pointcut_expr_tag_variant() {
        let pc = PointcutExpr::Tag(TagPattern {
            tags: vec!["secured".to_string()],
        });
        assert!(matches!(pc, PointcutExpr::Tag(_)));
    }

    #[test]
    fn pointcut_expr_qualifier_variant() {
        let pc = PointcutExpr::Qualifier(QualifierPattern {
            qualifier: "primary".to_string(),
        });
        assert!(matches!(pc, PointcutExpr::Qualifier(_)));
    }
}
