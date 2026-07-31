//! AutowireCandidateResolver — Spring 风格的自动装配候选解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AutowireCandidateResolver`。
//!
//! 判断某个 Bean 是否符合自动装配条件。在 Spring 中，此接口是
//! 自动装配决策的核心，支持 `@Qualifier`、`@Primary` 等注解。
//!
//! ## 实现类
//!
//! - `SimpleAutowireCandidateResolver` — 简单实现，所有 Bean 都是候选
//! - `QualifierAnnotationAutowireCandidateResolver` — 支持 `@Qualifier` 注解

use std::any::TypeId;

/// 自动装配候选解析器接口。
///
/// 对应 Spring 的 `AutowireCandidateResolver`。
///
/// 判断某个 Bean 是否符合自动装配条件。
/// Spring 中有多个实现，包括简单的全通过和基于注解的精确匹配。
pub trait AutowireCandidateResolver: Send + Sync {
    /// 判断是否为自动装配候选。
    ///
    /// 对应 Spring 的 `boolean isAutowireCandidate(BeanDefinitionHolder, DependencyDescriptor)`。
    ///
    /// # 参数
    /// - `type_id` — Bean 类型
    /// - `bean_name` — Bean 名称
    ///
    /// # 返回
    /// `true` 表示该 Bean 可以作为自动装配候选。
    fn is_autowire_candidate(&self, type_id: TypeId, bean_name: &str) -> bool;
}

/// 简单的自动装配候选解析器 — 所有 Bean 都是候选。
///
/// 对应 Spring 中最简单的解析器行为：不做任何过滤。
pub struct SimpleAutowireCandidateResolver;

impl SimpleAutowireCandidateResolver {
    /// 创建一个新的实例。
    pub fn new() -> Self { Self }
}

impl Default for SimpleAutowireCandidateResolver {
    fn default() -> Self { Self::new() }
}

impl AutowireCandidateResolver for SimpleAutowireCandidateResolver {
    fn is_autowire_candidate(&self, _type_id: TypeId, _bean_name: &str) -> bool {
        true
    }
}

/// 基于类型的自动装配候选解析器 — 只有注册过的类型才是候选。
///
/// 用于测试场景或需要白名单过滤的场景。
pub struct TypeBasedAutowireCandidateResolver {
    allowed_types: std::sync::Mutex<std::collections::HashSet<TypeId>>,
}

impl TypeBasedAutowireCandidateResolver {
    /// 创建一个新的实例。
    pub fn new() -> Self {
        Self {
            allowed_types: std::sync::Mutex::new(std::collections::HashSet::new()),
        }
    }

    /// 允许指定类型作为自动装配候选。
    pub fn allow_type(&self, type_id: TypeId) {
        self.allowed_types.lock().unwrap().insert(type_id);
    }

    /// 移除指定类型的候选资格。
    pub fn disallow_type(&self, type_id: TypeId) {
        self.allowed_types.lock().unwrap().remove(&type_id);
    }

    /// 获取已允许的类型数量。
    pub fn allowed_count(&self) -> usize {
        self.allowed_types.lock().unwrap().len()
    }
}

impl Default for TypeBasedAutowireCandidateResolver {
    fn default() -> Self { Self::new() }
}

impl AutowireCandidateResolver for TypeBasedAutowireCandidateResolver {
    fn is_autowire_candidate(&self, type_id: TypeId, _bean_name: &str) -> bool {
        self.allowed_types.lock().unwrap().contains(&type_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_resolver_always_returns_true() {
        let resolver = SimpleAutowireCandidateResolver::new();
        assert!(resolver.is_autowire_candidate(TypeId::of::<String>(), "myBean"));
        assert!(resolver.is_autowire_candidate(TypeId::of::<i32>(), "otherBean"));
    }

    #[test]
    fn type_based_resolver_filters_by_type() {
        let resolver = TypeBasedAutowireCandidateResolver::new();
        let string_tid = TypeId::of::<String>();
        let i32_tid = TypeId::of::<i32>();

        resolver.allow_type(string_tid);

        assert!(resolver.is_autowire_candidate(string_tid, "myBean"));
        assert!(!resolver.is_autowire_candidate(i32_tid, "otherBean"));
    }

    #[test]
    fn type_based_resolver_disallow_type() {
        let resolver = TypeBasedAutowireCandidateResolver::new();
        let tid = TypeId::of::<String>();

        resolver.allow_type(tid);
        assert!(resolver.is_autowire_candidate(tid, "bean"));
        assert_eq!(resolver.allowed_count(), 1);

        resolver.disallow_type(tid);
        assert!(!resolver.is_autowire_candidate(tid, "bean"));
        assert_eq!(resolver.allowed_count(), 0);
    }
}
