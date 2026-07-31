//! SimpleAutowireCandidateResolver — Spring 风格的简单自动装配候选解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleAutowireCandidateResolver`。
//!
//! 在 Spring 中，`SimpleAutowireCandidateResolver` 是 `AutowireCandidateResolver`
//! 的基本实现，根据 Bean 定义的 `autowire-candidate` 属性来判断是否为候选。
//! 不支持 `@Qualifier` 等注解（那是 `QualifierAnnotationAutowireCandidateResolver` 的职责）。
//!
//! ## 设计说明
//!
//! 在 vernal 中，此解析器维护一个排除列表，
//! 被排除的 Bean 不会作为自动装配候选。

use std::collections::HashSet;
use std::sync::Mutex;

/// 简单自动装配候选解析器。
///
/// 对应 Spring 的 `SimpleAutowireCandidateResolver`。
///
/// 通过维护排除列表来控制 Bean 的自动装配候选资格。
/// 默认所有 Bean 都是候选，除非被显式排除。
#[derive(Debug, Default)]
pub struct SimpleAutowireCandidateResolver {
    /// 被排除的 Bean 名称集合。
    excluded_beans: Mutex<HashSet<String>>,
    /// 全局候选标志（false 时所有 Bean 都不是候选）。
    default_candidate: bool,
}

impl SimpleAutowireCandidateResolver {
    /// 创建简单自动装配候选解析器（默认所有 Bean 都是候选）。
    pub fn new() -> Self {
        Self {
            excluded_beans: Mutex::new(HashSet::new()),
            default_candidate: true,
        }
    }

    /// 创建指定默认候选状态的解析器。
    pub fn with_default_candidate(default_candidate: bool) -> Self {
        Self {
            excluded_beans: Mutex::new(HashSet::new()),
            default_candidate,
        }
    }

    /// 排除指定 Bean 作为自动装配候选。
    pub fn exclude_bean(&self, bean_name: &str) {
        self.excluded_beans.lock().unwrap().insert(bean_name.to_string());
    }

    /// 恢复指定 Bean 的候选资格。
    pub fn include_bean(&self, bean_name: &str) {
        self.excluded_beans.lock().unwrap().remove(bean_name);
    }

    /// 判断指定 Bean 是否为自动装配候选。
    ///
    /// 对应 Spring 的 `boolean isAutowireCandidate(BeanDefinitionHolder, DependencyDescriptor)`。
    pub fn is_autowire_candidate(&self, bean_name: &str) -> bool {
        if !self.default_candidate {
            return false;
        }
        !self.excluded_beans.lock().unwrap().contains(bean_name)
    }

    /// 获取已排除的 Bean 数量。
    pub fn excluded_count(&self) -> usize {
        self.excluded_beans.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_beans_are_candidates_by_default() {
        let resolver = SimpleAutowireCandidateResolver::new();
        assert!(resolver.is_autowire_candidate("anyBean"));
        assert!(resolver.is_autowire_candidate("otherBean"));
    }

    #[test]
    fn exclude_bean_removes_candidate() {
        let resolver = SimpleAutowireCandidateResolver::new();
        resolver.exclude_bean("internalService");
        assert!(!resolver.is_autowire_candidate("internalService"));
        assert!(resolver.is_autowire_candidate("publicService"));
    }

    #[test]
    fn include_bean_restores_candidate() {
        let resolver = SimpleAutowireCandidateResolver::new();
        resolver.exclude_bean("tempBean");
        assert!(!resolver.is_autowire_candidate("tempBean"));

        resolver.include_bean("tempBean");
        assert!(resolver.is_autowire_candidate("tempBean"));
    }

    #[test]
    fn default_candidate_false_excludes_all() {
        let resolver = SimpleAutowireCandidateResolver::with_default_candidate(false);
        assert!(!resolver.is_autowire_candidate("anyBean"));
        assert_eq!(resolver.excluded_count(), 0);
    }
}
