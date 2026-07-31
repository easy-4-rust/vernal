//! DocumentDefaultsDefinition — Spring 风格的文档默认值定义。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DocumentDefaultsDefinition`。
//!
//! 在 Spring 中，`DocumentDefaultsDefinition` 封装了从 `<beans>` 根元素
//! 解析出的默认属性（如 default-lazy-init, default-autowire 等）。
//! 这些默认值会应用到文档中所有没有显式指定的 Bean 定义。

/// 文档默认值定义。
///
/// 对应 Spring 的 `DocumentDefaultsDefinition`。
///
/// 封装 `<beans>` 根元素的默认属性。
#[derive(Debug, Clone)]
pub struct DocumentDefaultsDefinition {
    /// 默认作用域（如 singleton, prototype）。
    pub default_scope: Option<String>,
    /// 默认是否延迟初始化。
    pub default_lazy_init: Option<bool>,
    /// 默认自动装配模式（no, byName, byType, constructor）。
    pub default_autowire: Option<String>,
    /// 默认依赖检查模式（none, simple, object, all）。
    pub default_dependency_check: Option<String>,
    /// 默认初始化方法名。
    pub default_init_method: Option<String>,
    /// 默认销毁方法名。
    pub default_destroy_method: Option<String>,
    /// 默认 merge 标志。
    pub default_merge: Option<bool>,
    /// 默认 profile。
    pub default_profile: Option<String>,
}

impl DocumentDefaultsDefinition {
    /// 创建文档默认值定义。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置默认作用域。
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.default_scope = Some(scope.into());
        self
    }

    /// 设置默认延迟初始化。
    pub fn with_lazy_init(mut self, lazy: bool) -> Self {
        self.default_lazy_init = Some(lazy);
        self
    }

    /// 设置默认自动装配模式。
    pub fn with_autowire(mut self, autowire: impl Into<String>) -> Self {
        self.default_autowire = Some(autowire.into());
        self
    }

    /// 检查是否为默认延迟初始化。
    pub fn is_default_lazy_init(&self) -> bool {
        self.default_lazy_init.unwrap_or(false)
    }

    /// 获取默认自动装配模式。
    pub fn autowire_mode(&self) -> &str {
        self.default_autowire.as_deref().unwrap_or("no")
    }
}

impl Default for DocumentDefaultsDefinition {
    fn default() -> Self {
        Self {
            default_scope: None,
            default_lazy_init: None,
            default_autowire: None,
            default_dependency_check: None,
            default_init_method: None,
            default_destroy_method: None,
            default_merge: None,
            default_profile: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_definition_has_no_overrides() {
        let def = DocumentDefaultsDefinition::new();
        assert!(def.default_scope.is_none());
        assert!(def.default_lazy_init.is_none());
        assert_eq!(def.is_default_lazy_init(), false);
        assert_eq!(def.autowire_mode(), "no");
    }

    #[test]
    fn builder_pattern_sets_fields() {
        let def = DocumentDefaultsDefinition::new()
            .with_scope("prototype")
            .with_lazy_init(true)
            .with_autowire("byName");
        assert_eq!(def.default_scope, Some("prototype".to_string()));
        assert_eq!(def.default_lazy_init, Some(true));
        assert_eq!(def.autowire_mode(), "byName");
    }

    #[test]
    fn clone_preserves_values() {
        let def = DocumentDefaultsDefinition::new()
            .with_scope("singleton")
            .with_lazy_init(false);
        let cloned = def.clone();
        assert_eq!(cloned.default_scope, Some("singleton".to_string()));
        assert_eq!(cloned.default_lazy_init, Some(false));
    }
}
