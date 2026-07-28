//! 对应 aspect-rs：aspect-core/src/pointcut/pattern.rs
//! 语义参照 spring-aop：ClassFilter / MethodMatcher / NameMatchMethodPointcut
//!
//! 切点匹配模式类型（4 类，直接移植自 aspect-rs，名称完全一致）。

/// 函数可见性模式。
///
/// 对应 aspect-rs `Visibility` enum，Rust 原生可见性概念。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    /// 公开：`pub`
    Public,
    /// crate 内公开：`pub(crate)`
    Crate,
    /// 父模块公开：`pub(super)`
    Super,
    /// 私有（无可见性修饰符）
    Private,
}

impl Visibility {
    /// 检查可见性字符串是否匹配此模式。
    pub fn matches(&self, vis: &str) -> bool {
        matches!(
            (self, vis),
            (Visibility::Public, "pub")
                | (Visibility::Crate, "pub(crate)")
                | (Visibility::Super, "pub(super)")
                | (Visibility::Private, "")
        )
    }
}

/// 函数名模式。
///
/// 对应 spring-aop `NameMatchMethodPointcut` 的匹配语义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamePattern {
    /// 匹配任意名称：`*`
    Wildcard,
    /// 精确匹配：`save_user`
    Exact(String),
    /// 前缀匹配：`save*`
    Prefix(String),
    /// 后缀匹配：`*_user`
    Suffix(String),
    /// 包含匹配：`*save*`
    Contains(String),
}

impl NamePattern {
    /// 检查函数名是否匹配此模式。
    pub fn matches(&self, name: &str) -> bool {
        match self {
            NamePattern::Wildcard => true,
            NamePattern::Exact(expected) => name == expected,
            NamePattern::Prefix(prefix) => name.starts_with(prefix.as_str()),
            NamePattern::Suffix(suffix) => name.ends_with(suffix.as_str()),
            NamePattern::Contains(substring) => name.contains(substring.as_str()),
        }
    }
}

/// 执行模式：匹配函数签名。
///
/// 对应 spring-aop AspectJ `execution()` 设计器。
///
/// # 示例
///
/// - `execution(pub fn *(..))` — 所有公开函数
/// - `execution(fn save(..))` — 名为 save 的函数
/// - `execution(pub fn save*(..) -> Result<*, *>)` — 名为 save* 且返回 Result 的公开函数
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionPattern {
    /// 可见性模式（pub、pub(crate) 等）
    pub visibility: Option<Visibility>,

    /// 函数名模式
    pub name: NamePattern,

    /// 返回类型模式（简化字符串匹配）
    pub return_type: Option<String>,
}

impl ExecutionPattern {
    /// 创建匹配所有函数的模式。
    pub fn any() -> Self {
        Self {
            visibility: None,
            name: NamePattern::Wildcard,
            return_type: None,
        }
    }

    /// 创建匹配所有公开函数的模式。
    pub fn public() -> Self {
        Self {
            visibility: Some(Visibility::Public),
            name: NamePattern::Wildcard,
            return_type: None,
        }
    }

    /// 创建匹配指定函数名的模式。
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            visibility: None,
            name: NamePattern::Exact(name.into()),
            return_type: None,
        }
    }
}

/// 模块模式：按模块路径匹配函数。
///
/// 对应 spring-aop AspectJ `within()` 设计器。
///
/// # 示例
///
/// - `within(crate::api)` — api 模块内的函数
/// - `within(crate::api::users)` — users 子模块内的函数
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePattern {
    /// 要匹配的模块路径（如 "crate::api"）
    pub path: String,
}

impl ModulePattern {
    /// 创建新的模块模式。
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// 检查模块路径是否匹配此模式。
    ///
    /// 支持精确匹配和前缀匹配（子模块）。
    pub fn matches_path(&self, module_path: &str) -> bool {
        module_path == self.path || module_path.starts_with(&format!("{}::", self.path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_matches_correctly() {
        assert!(Visibility::Public.matches("pub"));
        assert!(Visibility::Crate.matches("pub(crate)"));
        assert!(Visibility::Super.matches("pub(super)"));
        assert!(Visibility::Private.matches(""));
        assert!(!Visibility::Public.matches("pub(crate)"));
        assert!(!Visibility::Crate.matches("pub"));
        assert!(!Visibility::Private.matches("pub"));
    }

    #[test]
    fn name_pattern_wildcard_matches_all() {
        assert!(NamePattern::Wildcard.matches("anything"));
        assert!(NamePattern::Wildcard.matches("save_user"));
        assert!(NamePattern::Wildcard.matches(""));
    }

    #[test]
    fn name_pattern_exact_matches_only_exact() {
        let pattern = NamePattern::Exact("save".to_string());
        assert!(pattern.matches("save"));
        assert!(!pattern.matches("save_user"));
        assert!(!pattern.matches("update"));
    }

    #[test]
    fn name_pattern_prefix_matches_from_start() {
        let pattern = NamePattern::Prefix("save".to_string());
        assert!(pattern.matches("save"));
        assert!(pattern.matches("save_user"));
        assert!(!pattern.matches("update_user"));
        assert!(!pattern.matches("my_save"));
    }

    #[test]
    fn name_pattern_suffix_matches_from_end() {
        let pattern = NamePattern::Suffix("_user".to_string());
        assert!(pattern.matches("save_user"));
        assert!(pattern.matches("update_user"));
        assert!(!pattern.matches("save"));
        assert!(!pattern.matches("user_save"));
    }

    #[test]
    fn name_pattern_contains_matches_substring() {
        let pattern = NamePattern::Contains("save".to_string());
        assert!(pattern.matches("save"));
        assert!(pattern.matches("save_user"));
        assert!(pattern.matches("my_save_data"));
        assert!(!pattern.matches("update"));
    }

    #[test]
    fn module_pattern_exact_match() {
        let pattern = ModulePattern::new("crate::api");
        assert!(pattern.matches_path("crate::api"));
    }

    #[test]
    fn module_pattern_prefix_match() {
        let pattern = ModulePattern::new("crate::api");
        assert!(pattern.matches_path("crate::api::users"));
        assert!(pattern.matches_path("crate::api::users::models"));
    }

    #[test]
    fn module_pattern_no_match() {
        let pattern = ModulePattern::new("crate::api");
        assert!(!pattern.matches_path("crate::internal"));
        assert!(!pattern.matches_path("crate"));
        assert!(!pattern.matches_path("other::api"));
    }

    #[test]
    fn execution_pattern_any_matches_all() {
        let pattern = ExecutionPattern::any();
        assert_eq!(pattern.name, NamePattern::Wildcard);
        assert!(pattern.visibility.is_none());
        assert!(pattern.return_type.is_none());
    }

    #[test]
    fn execution_pattern_public_has_correct_visibility() {
        let pattern = ExecutionPattern::public();
        assert_eq!(pattern.visibility, Some(Visibility::Public));
    }

    #[test]
    fn execution_pattern_named_has_exact_name() {
        let pattern = ExecutionPattern::named("save_user");
        assert_eq!(pattern.name, NamePattern::Exact("save_user".to_string()));
    }
}
