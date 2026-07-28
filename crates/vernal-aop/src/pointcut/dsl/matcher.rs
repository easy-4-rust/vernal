//! 对应 aspect-rs：aspect-core/src/pointcut/matcher.rs（Matcher + FunctionInfo）
//! 语义参照 spring-aop：MethodMatcher / ClassFilter
//!
//! 切点匹配逻辑（改名：Matcher → PointcutMatcher，FunctionInfo → FunctionDescriptor）。
//! 移植自 aspect-rs 的 matcher，扩展 Tag/Qualifier 匹配，接入 vernal Operation。

use crate::Operation;

use super::ast::PointcutExpr;
use super::pattern::{ExecutionPattern, ModulePattern};

/// 函数描述符（用于切点匹配的中间结构）。
///
/// 对应 aspect-rs `FunctionInfo`（改名 `FunctionDescriptor`），
/// 从 vernal [`Operation`] 转换而来。
#[derive(Debug, Clone)]
pub struct FunctionDescriptor {
    /// 函数名
    pub name: String,

    /// 模块路径（如 "crate::api::users"）
    pub module_path: String,

    /// 可见性字符串（"pub"、"pub(crate)" 等）
    pub visibility: String,

    /// 返回类型字符串（简化）
    pub return_type: Option<String>,

    /// 标签列表（vernal 扩展，aspect-rs 无对偶）
    pub tags: Vec<String>,

    /// 限定符（vernal 扩展，aspect-rs 无对偶）
    pub qualifier: Option<String>,
}

impl FunctionDescriptor {
    /// 创建函数描述符（用于测试）。
    pub fn new(
        name: impl Into<String>,
        module_path: impl Into<String>,
        visibility: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            module_path: module_path.into(),
            visibility: visibility.into(),
            return_type: None,
            tags: Vec::new(),
            qualifier: None,
        }
    }

    /// 设置返回类型。
    pub fn with_return_type(mut self, return_type: impl Into<String>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    /// 从 vernal [`Operation`] 转换为函数描述符。
    ///
    /// component → module_path, method → name, 无可见性/返回类型。
    /// 标签和限定符从元数据提取。
    pub fn from_operation(operation: &Operation) -> Self {
        let metadata = operation.metadata();
        Self {
            name: operation.method().to_string(),
            module_path: operation.component().to_string(),
            visibility: String::new(),
            return_type: None,
            tags: metadata.tags().iter().map(|tag| tag.to_string()).collect(),
            qualifier: metadata.qualifier().map(String::from),
        }
    }
}

/// 切点匹配器 trait。
///
/// 对应 aspect-rs `Matcher` trait（改名 `PointcutMatcher`），
/// 方法名改为 `matches_operation` 以接收 vernal [`Operation`]。
pub trait PointcutMatcher {
    /// 检查此切点是否匹配给定操作。
    fn matches_operation(&self, operation: &Operation) -> bool;
}

impl PointcutMatcher for PointcutExpr {
    fn matches_operation(&self, operation: &Operation) -> bool {
        let func = FunctionDescriptor::from_operation(operation);
        match self {
            PointcutExpr::Execution(pattern) => pattern.matches_descriptor(&func),
            PointcutExpr::Within(pattern) => pattern.matches_path(&func.module_path),
            PointcutExpr::Tag(pattern) => pattern
                .tags
                .iter()
                .any(|tag| operation.metadata().has_tag(tag)),
            PointcutExpr::Qualifier(pattern) => {
                operation.metadata().qualifier() == Some(pattern.qualifier.as_str())
            }
            PointcutExpr::And(left, right) => {
                left.matches_operation(operation) && right.matches_operation(operation)
            }
            PointcutExpr::Or(left, right) => {
                left.matches_operation(operation) || right.matches_operation(operation)
            }
            PointcutExpr::Not(inner) => !inner.matches_operation(operation),
        }
    }
}

/// 标签匹配模式（vernal 扩展）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagPattern {
    /// 要匹配的标签列表（任一匹配即可）
    pub tags: Vec<String>,
}

/// 限定符匹配模式（vernal 扩展）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifierPattern {
    /// 要匹配的限定符
    pub qualifier: String,
}

impl ExecutionPattern {
    /// 检查函数描述符是否匹配此执行模式。
    fn matches_descriptor(&self, func: &FunctionDescriptor) -> bool {
        if let Some(ref vis) = self.visibility {
            if !vis.matches(&func.visibility) {
                return false;
            }
        }

        if !self.name.matches(&func.name) {
            return false;
        }

        if let Some(ref expected_return) = self.return_type {
            match &func.return_type {
                Some(actual_return) if actual_return.contains(expected_return.as_str()) => {}
                _ => return false,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OperationMetadata;
    use crate::pointcut::dsl::pattern::{NamePattern, Visibility};

    #[test]
    fn execution_pattern_matches_public_exact_name() {
        let pattern = ExecutionPattern {
            visibility: Some(Visibility::Public),
            name: NamePattern::Exact("save_user".to_string()),
            return_type: None,
        };

        let func = FunctionDescriptor::new("save_user", "crate::api", "pub");
        assert!(pattern.matches_descriptor(&func));

        let func2 = FunctionDescriptor::new("update_user", "crate::api", "pub");
        assert!(!pattern.matches_descriptor(&func2));

        let func3 = FunctionDescriptor::new("save_user", "crate::api", "");
        assert!(!pattern.matches_descriptor(&func3));
    }

    #[test]
    fn execution_pattern_matches_with_return_type() {
        let pattern = ExecutionPattern {
            visibility: None,
            name: NamePattern::Wildcard,
            return_type: Some("Result".to_string()),
        };

        let func = FunctionDescriptor::new("save", "crate::api", "pub")
            .with_return_type("Result<User, Error>");
        assert!(pattern.matches_descriptor(&func));

        let func2 = FunctionDescriptor::new("save", "crate::api", "pub");
        assert!(!pattern.matches_descriptor(&func2));
    }

    #[test]
    fn module_pattern_matches_descriptor() {
        let pattern = ModulePattern::new("crate::api");

        let func = FunctionDescriptor::new("save", "crate::api", "pub");
        assert!(pattern.matches_path(&func.module_path));

        let func2 = FunctionDescriptor::new("save", "crate::api::users", "pub");
        assert!(pattern.matches_path(&func2.module_path));

        let func3 = FunctionDescriptor::new("save", "crate::internal", "pub");
        assert!(!pattern.matches_path(&func3.module_path));
    }

    #[test]
    fn pointcut_expr_and_matches_only_when_both_match() {
        // 使用无可见性约束的执行模式（因为 FunctionDescriptor::from_operation 没有可见性信息）
        let exec = ExecutionPattern {
            visibility: None,
            name: NamePattern::Wildcard,
            return_type: None,
        };
        let within = ModulePattern::new("crate::api");
        let pointcut = PointcutExpr::Execution(exec).and(PointcutExpr::Within(within));

        let op1 = Operation::new("crate::api", "save");
        assert!(pointcut.matches_operation(&op1));

        let op2 = Operation::new("crate::internal", "save");
        assert!(!pointcut.matches_operation(&op2));
    }

    #[test]
    fn pointcut_expr_or_matches_either() {
        let pattern1 = ExecutionPattern::named("save");
        let pattern2 = ExecutionPattern::named("update");
        let pointcut = PointcutExpr::Execution(pattern1).or(PointcutExpr::Execution(pattern2));

        let op1 = Operation::new("crate::api", "save");
        assert!(pointcut.matches_operation(&op1));

        let op2 = Operation::new("crate::api", "update");
        assert!(pointcut.matches_operation(&op2));

        let op3 = Operation::new("crate::api", "delete");
        assert!(!pointcut.matches_operation(&op3));
    }

    #[test]
    fn pointcut_expr_not_inverts_match() {
        let pattern = ExecutionPattern {
            visibility: Some(Visibility::Public),
            name: NamePattern::Wildcard,
            return_type: None,
        };
        let pointcut = PointcutExpr::Execution(pattern).not();

        // Operation doesn't have visibility, so the execution pattern
        // with visibility=Public won't match via from_operation (empty visibility)
        let op1 = Operation::new("crate::api", "save");
        // The FunctionDescriptor from Operation has empty visibility,
        // so Public visibility pattern won't match → NOT inverts to true
        assert!(pointcut.matches_operation(&op1));
    }

    #[test]
    fn function_descriptor_from_operation_extracts_metadata() {
        let op = Operation::new("UserService", "create_user").with_metadata(
            OperationMetadata::empty()
                .with_tag("transactional")
                .unwrap()
                .with_tag("secured")
                .unwrap()
                .with_qualifier("primary")
                .unwrap(),
        );
        let fd = FunctionDescriptor::from_operation(&op);

        assert_eq!(fd.name, "create_user");
        assert_eq!(fd.module_path, "UserService");
        assert!(fd.tags.contains(&"transactional".to_string()));
        assert!(fd.tags.contains(&"secured".to_string()));
        assert_eq!(fd.qualifier, Some("primary".to_string()));
    }

    #[test]
    fn tag_pattern_matches_operation() {
        let pattern = PointcutExpr::Tag(TagPattern {
            tags: vec!["secured".to_string()],
        });

        let op = Operation::new("Svc", "method")
            .with_metadata(OperationMetadata::empty().with_tag("secured").unwrap());
        assert!(pattern.matches_operation(&op));

        let op2 = Operation::new("Svc", "method");
        assert!(!pattern.matches_operation(&op2));
    }

    #[test]
    fn qualifier_pattern_matches_operation() {
        let pattern = PointcutExpr::Qualifier(QualifierPattern {
            qualifier: "primary".to_string(),
        });

        let op = Operation::new("Svc", "method").with_metadata(
            OperationMetadata::empty()
                .with_qualifier("primary")
                .unwrap(),
        );
        assert!(pattern.matches_operation(&op));

        let op2 = Operation::new("Svc", "method").with_metadata(
            OperationMetadata::empty()
                .with_qualifier("secondary")
                .unwrap(),
        );
        assert!(!pattern.matches_operation(&op2));

        let op3 = Operation::new("Svc", "method");
        assert!(!pattern.matches_operation(&op3));
    }
}
