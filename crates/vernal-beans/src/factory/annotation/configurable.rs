//! Configurable — Spring 风格 @Configurable 注解标记。
//!
//! 对应 Java 注解：`org.springframework.beans.factory.annotation.Configurable`。
//!
//! 标记一个类为可配置的，允许 Spring 容器在 AspectJ 织入场景中
//! 管理其依赖注入。典型用法：`@Configurable(autowire = Autowire.BY_TYPE)`。

/// Spring 风格的 @Configurable 注解标记。
///
/// 对应 Spring 的 `@Configurable`。
///
/// 标记一个类为可配置的，允许 Spring 容器管理其依赖注入。
/// 通常与 AspectJ 编译时/加载时织入配合使用。
#[derive(Debug, Clone)]
pub struct Configurable {
    /// 是否启用此注解
    enabled: bool,
    /// 是否自动装配
    autowire: bool,
    /// 预构建标志
    pre_construction: bool,
}

impl Configurable {
    /// 创建默认的 Configurable 注解（enabled=true, autowire=true, preConstruction=false）。
    pub fn new() -> Self { Self { enabled: true, autowire: true, pre_construction: false } }

    /// 是否启用。
    pub fn enabled(&self) -> bool { self.enabled }

    /// 设置启用状态。
    pub fn set_enabled(&mut self, v: bool) { self.enabled = v; }

    /// 是否自动装配。
    pub fn is_autowire(&self) -> bool { self.autowire }

    /// 设置自动装配。
    pub fn set_autowire(&mut self, v: bool) { self.autowire = v; }

    /// 是否预构建（在构造前完成依赖注入）。
    pub fn is_pre_construction(&self) -> bool { self.pre_construction }

    /// 设置预构建标志。
    pub fn set_pre_construction(&mut self, v: bool) { self.pre_construction = v; }

    /// 创建一个启用自动装配的配置。
    pub fn autowire_enabled() -> Self {
        Self { enabled: true, autowire: true, pre_construction: false }
    }

    /// 创建一个禁用自动装配的配置。
    pub fn autowire_disabled() -> Self {
        Self { enabled: true, autowire: false, pre_construction: false }
    }

    /// 创建一个启用预构建的配置。
    pub fn with_pre_construction() -> Self {
        Self { enabled: true, autowire: true, pre_construction: true }
    }

    /// 创建一个禁用的配置。
    pub fn disabled() -> Self {
        Self { enabled: false, autowire: false, pre_construction: false }
    }

    /// 检查是否所有选项都为默认值。
    pub fn is_all_default(&self) -> bool {
        self.enabled && self.autowire && !self.pre_construction
    }

    /// 获取配置的摘要字符串。
    pub fn summary(&self) -> String {
        format!(
            "Configurable(enabled={}, autowire={}, preConstruction={})",
            self.enabled, self.autowire, self.pre_construction
        )
    }
}

impl Default for Configurable { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let c = Configurable::new();
        assert!(c.enabled());
        assert!(c.is_autowire());
        assert!(!c.is_pre_construction());
    }

    #[test]
    fn set_and_get() {
        let mut c = Configurable::new();
        c.set_enabled(false);
        c.set_autowire(false);
        c.set_pre_construction(true);

        assert!(!c.enabled());
        assert!(!c.is_autowire());
        assert!(c.is_pre_construction());
    }

    #[test]
    fn clone_preserves_values() {
        let mut c = Configurable::new();
        c.set_pre_construction(true);
        let cloned = c.clone();
        assert!(cloned.is_pre_construction());
    }

    #[test]
    fn autowire_enabled_preset() {
        let c = Configurable::autowire_enabled();
        assert!(c.enabled());
        assert!(c.is_autowire());
        assert!(!c.is_pre_construction());
    }

    #[test]
    fn autowire_disabled_preset() {
        let c = Configurable::autowire_disabled();
        assert!(c.enabled());
        assert!(!c.is_autowire());
    }

    #[test]
    fn with_pre_construction_preset() {
        let c = Configurable::with_pre_construction();
        assert!(c.enabled());
        assert!(c.is_autowire());
        assert!(c.is_pre_construction());
    }

    #[test]
    fn disabled_preset() {
        let c = Configurable::disabled();
        assert!(!c.enabled());
        assert!(!c.is_autowire());
        assert!(!c.is_pre_construction());
    }

    #[test]
    fn is_all_default() {
        assert!(Configurable::new().is_all_default());
        assert!(!Configurable::disabled().is_all_default());
    }

    #[test]
    fn summary_format() {
        let c = Configurable::new();
        assert_eq!(
            c.summary(),
            "Configurable(enabled=true, autowire=true, preConstruction=false)"
        );
    }
}
