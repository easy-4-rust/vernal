//! BeanDefinitionDefaults — Spring 风格 Bean 定义默认值。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanDefinitionDefaults`。
//!
//! 存储 Bean 定义的默认配置值。

/// Spring 风格的 Bean 定义默认值。
///
/// 对应 Spring 的 `BeanDefinitionDefaults`。
///
/// 存储 Bean 定义的默认配置值，包括：
/// - 是否懒加载
/// - 是否自动装配
/// - 是否依赖检查
/// - 是否自动装配候选
/// - 是否主要候选
#[derive(Debug, Clone, Default)]
pub struct BeanDefinitionDefaults {
    /// 是否懒加载
    pub lazy_init: bool,
    /// 是否自动装配
    pub autowire: bool,
    /// 是否依赖检查
    pub dependency_check: bool,
    /// 是否自动装配候选
    pub autowire_candidate: bool,
    /// 是否主要候选
    pub primary: bool,
}

impl BeanDefinitionDefaults {
    /// 创建新的 BeanDefinitionDefaults。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置懒加载
    pub fn set_lazy_init(&mut self, v: bool) {
        self.lazy_init = v;
    }

    /// 是否懒加载
    pub fn is_lazy_init(&self) -> bool {
        self.lazy_init
    }

    /// 设置自动装配
    pub fn set_autowire(&mut self, v: bool) {
        self.autowire = v;
    }

    /// 是否自动装配
    pub fn is_autowire(&self) -> bool {
        self.autowire
    }

    /// 设置依赖检查
    pub fn set_dependency_check(&mut self, v: bool) {
        self.dependency_check = v;
    }

    /// 是否依赖检查
    pub fn is_dependency_check(&self) -> bool {
        self.dependency_check
    }

    /// 设置自动装配候选
    pub fn set_autowire_candidate(&mut self, v: bool) {
        self.autowire_candidate = v;
    }

    /// 是否自动装配候选
    pub fn is_autowire_candidate(&self) -> bool {
        self.autowire_candidate
    }

    /// 设置主要候选
    pub fn set_primary(&mut self, v: bool) {
        self.primary = v;
    }

    /// 是否主要候选
    pub fn is_primary(&self) -> bool {
        self.primary
    }

    /// 从另一个 BeanDefinitionDefaults 复制所有值。
    ///
    /// 对应 Spring 的 `BeanDefinitionDefaults.copyFrom`。
    pub fn copy_from(&mut self, other: &BeanDefinitionDefaults) {
        self.lazy_init = other.lazy_init;
        self.autowire = other.autowire;
        self.dependency_check = other.dependency_check;
        self.autowire_candidate = other.autowire_candidate;
        self.primary = other.primary;
    }

    /// 检查是否有任何非默认值（即至少有一个字段为 true）。
    pub fn has_non_default_values(&self) -> bool {
        self.lazy_init
            || self.autowire
            || self.dependency_check
            || self.autowire_candidate
            || self.primary
    }

    /// 重置所有值为默认值（false）。
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 创建一个启用了懒加载和自动装配的默认配置。
    pub fn lazy_autowire() -> Self {
        Self {
            lazy_init: true,
            autowire: true,
            dependency_check: false,
            autowire_candidate: true,
            primary: false,
        }
    }

    /// 创建一个严格模式的默认配置（启用依赖检查）。
    pub fn strict() -> Self {
        Self {
            lazy_init: false,
            autowire: false,
            dependency_check: true,
            autowire_candidate: true,
            primary: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_are_false() {
        let defaults = BeanDefinitionDefaults::new();
        assert!(!defaults.is_lazy_init());
        assert!(!defaults.is_autowire());
        assert!(!defaults.is_dependency_check());
        assert!(!defaults.is_autowire_candidate());
        assert!(!defaults.is_primary());
    }

    #[test]
    fn set_and_get_all_fields() {
        let mut defaults = BeanDefinitionDefaults::new();
        defaults.set_lazy_init(true);
        defaults.set_autowire(true);
        defaults.set_dependency_check(true);
        defaults.set_autowire_candidate(true);
        defaults.set_primary(true);

        assert!(defaults.is_lazy_init());
        assert!(defaults.is_autowire());
        assert!(defaults.is_dependency_check());
        assert!(defaults.is_autowire_candidate());
        assert!(defaults.is_primary());
    }

    #[test]
    fn copy_from_overwrites_values() {
        let mut src = BeanDefinitionDefaults::new();
        src.set_lazy_init(true);
        src.set_primary(true);

        let mut dst = BeanDefinitionDefaults::new();
        dst.copy_from(&src);

        assert!(dst.is_lazy_init());
        assert!(dst.is_primary());
        assert!(!dst.is_autowire());
    }

    #[test]
    fn has_non_default_values_detects_changes() {
        let mut defaults = BeanDefinitionDefaults::new();
        assert!(!defaults.has_non_default_values());

        defaults.set_lazy_init(true);
        assert!(defaults.has_non_default_values());
    }

    #[test]
    fn reset_restores_defaults() {
        let mut defaults = BeanDefinitionDefaults::new();
        defaults.set_lazy_init(true);
        defaults.set_autowire(true);
        defaults.reset();

        assert!(!defaults.is_lazy_init());
        assert!(!defaults.is_autowire());
    }

    #[test]
    fn lazy_autowire_preset() {
        let defaults = BeanDefinitionDefaults::lazy_autowire();
        assert!(defaults.is_lazy_init());
        assert!(defaults.is_autowire());
        assert!(defaults.is_autowire_candidate());
        assert!(!defaults.is_dependency_check());
    }

    #[test]
    fn strict_preset() {
        let defaults = BeanDefinitionDefaults::strict();
        assert!(defaults.is_dependency_check());
        assert!(defaults.is_autowire_candidate());
        assert!(!defaults.is_lazy_init());
        assert!(!defaults.is_autowire());
    }
}
