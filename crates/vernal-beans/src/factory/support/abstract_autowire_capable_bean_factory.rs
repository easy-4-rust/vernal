//! AbstractAutowireCapableBeanFactory — Spring 风格自动装配 Bean 工厂抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractAutowireCapableBeanFactory`。
//!
//! 在 Spring 中，此抽象类扩展了 `AbstractBeanFactory`，提供了自动装配能力，
//! 包括按名称/类型自动装配、构造器注入、属性注入等。
//! 它是 `DefaultListableBeanFactory` 的父类。
//!
//! ## 设计说明
//!
//! Spring 的 `AbstractAutowireCapableBeanFactory` 包含大量反射逻辑，
//! 在 vernal 中通过 `TypeId` 和闭包替代反射，保留了语义对齐。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

/// 自动装配模式枚举。
///
/// 对应 Spring 的 `AutowireCapableBeanFactory.AUTOWIRE_NO` / `AUTOWIRE_BY_NAME` / `AUTOWIRE_BY_TYPE` / `AUTOWIRE_CONSTRUCTOR`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutowireMode {
    /// 不自动装配
    No = 0,
    /// 按名称装配
    ByName = 1,
    /// 按类型装配
    ByType = 2,
    /// 构造器装配
    Constructor = 3,
}

impl AutowireMode {
    /// 从整数值创建 AutowireMode。
    pub fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::No),
            1 => Some(Self::ByName),
            2 => Some(Self::ByType),
            3 => Some(Self::Constructor),
            _ => None,
        }
    }
}

/// 抽象自动装配 Bean 工厂基类。
///
/// 对应 Spring 的 `AbstractAutowireCapableBeanFactory`。
///
/// 提供自动装配的核心能力：
/// - 忽略特定类型的依赖注入
/// - 自动装配模式管理
/// - 构造器注入支持
pub struct AbstractAutowireCapableBeanFactory {
    /// 忽略的依赖类型集合
    ignored_dependency_types: Mutex<HashSet<TypeId>>,
    /// 默认自动装配模式
    default_autowire_mode: Mutex<AutowireMode>,
    /// 是否允许循环引用
    allow_circular_references: Mutex<bool>,
}

impl AbstractAutowireCapableBeanFactory {
    /// 创建新的自动装配 Bean 工厂。
    pub fn new() -> Self {
        Self {
            ignored_dependency_types: Mutex::new(HashSet::new()),
            default_autowire_mode: Mutex::new(AutowireMode::No),
            allow_circular_references: Mutex::new(true),
        }
    }

    /// 将指定类型添加到忽略列表。
    ///
    /// 对应 Spring 的 `ignoreDependencyType(Class<?> type)`。
    ///
    /// 被忽略的类型在自动装配时不会被注入。
    /// 典型的忽略类型包括 `BeanFactory`、`ApplicationContext` 等，
    /// 因为这些通过 Aware 接口注入。
    pub fn ignore_dependency_type(&self, type_id: TypeId) {
        self.ignored_dependency_types
            .lock()
            .unwrap()
            .insert(type_id);
    }

    /// 从忽略列表中移除指定类型。
    pub fn unignore_dependency_type(&self, type_id: TypeId) {
        self.ignored_dependency_types
            .lock()
            .unwrap()
            .remove(&type_id);
    }

    /// 检查指定类型是否被忽略。
    pub fn is_dependency_ignored(&self, type_id: TypeId) -> bool {
        self.ignored_dependency_types
            .lock()
            .unwrap()
            .contains(&type_id)
    }

    /// 获取已忽略的依赖类型数量。
    pub fn ignored_count(&self) -> usize {
        self.ignored_dependency_types.lock().unwrap().len()
    }

    /// 设置默认自动装配模式。
    ///
    /// 对应 Spring 的 `setAutowireMode(int autowireMode)`。
    pub fn set_default_autowire_mode(&self, mode: AutowireMode) {
        *self.default_autowire_mode.lock().unwrap() = mode;
    }

    /// 获取默认自动装配模式。
    pub fn default_autowire_mode(&self) -> AutowireMode {
        *self.default_autowire_mode.lock().unwrap()
    }

    /// 设置是否允许循环引用。
    ///
    /// 对应 Spring 的 `setAllowCircularReferences(boolean allowCircularReferences)`。
    pub fn set_allow_circular_references(&self, allow: bool) {
        *self.allow_circular_references.lock().unwrap() = allow;
    }

    /// 检查是否允许循环引用。
    pub fn is_allow_circular_references(&self) -> bool {
        *self.allow_circular_references.lock().unwrap()
    }

    /// 清空所有忽略的依赖类型。
    pub fn clear_ignored_dependency_types(&self) {
        self.ignored_dependency_types.lock().unwrap().clear();
    }
}

impl Default for AbstractAutowireCapableBeanFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_factory_has_no_ignored_types() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        assert_eq!(factory.ignored_count(), 0);
    }

    #[test]
    fn ignore_and_check_dependency_type() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        let tid = TypeId::of::<String>();

        factory.ignore_dependency_type(tid);
        assert!(factory.is_dependency_ignored(tid));
        assert_eq!(factory.ignored_count(), 1);
    }

    #[test]
    fn unignore_dependency_type() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        let tid = TypeId::of::<i32>();

        factory.ignore_dependency_type(tid);
        assert!(factory.is_dependency_ignored(tid));

        factory.unignore_dependency_type(tid);
        assert!(!factory.is_dependency_ignored(tid));
        assert_eq!(factory.ignored_count(), 0);
    }

    #[test]
    fn default_autowire_mode_is_no() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        assert_eq!(factory.default_autowire_mode(), AutowireMode::No);
    }

    #[test]
    fn set_autowire_mode() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        factory.set_default_autowire_mode(AutowireMode::ByType);
        assert_eq!(factory.default_autowire_mode(), AutowireMode::ByType);
    }

    #[test]
    fn circular_references_allowed_by_default() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        assert!(factory.is_allow_circular_references());
    }

    #[test]
    fn disable_circular_references() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        factory.set_allow_circular_references(false);
        assert!(!factory.is_allow_circular_references());
    }

    #[test]
    fn clear_ignored_dependency_types() {
        let factory = AbstractAutowireCapableBeanFactory::new();
        factory.ignore_dependency_type(TypeId::of::<String>());
        factory.ignore_dependency_type(TypeId::of::<i32>());
        assert_eq!(factory.ignored_count(), 2);

        factory.clear_ignored_dependency_types();
        assert_eq!(factory.ignored_count(), 0);
    }

    #[test]
    fn autowire_mode_from_value() {
        assert_eq!(AutowireMode::from_value(0), Some(AutowireMode::No));
        assert_eq!(AutowireMode::from_value(1), Some(AutowireMode::ByName));
        assert_eq!(AutowireMode::from_value(2), Some(AutowireMode::ByType));
        assert_eq!(AutowireMode::from_value(3), Some(AutowireMode::Constructor));
        assert_eq!(AutowireMode::from_value(99), None);
    }
}
