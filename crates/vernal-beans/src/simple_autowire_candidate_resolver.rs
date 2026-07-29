//! SimpleAutowireCandidateResolver — Spring 风格的简单自动装配候选解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.SimpleAutowireCandidateResolver`。
//!
//! 提供最基本的自动装配候选判定：始终返回 `true`。

use crate::bean_definition::BeanDefinition;

/// Spring 风格的简单自动装配候选解析器。
///
/// 对应 Spring 的 `SimpleAutowireCandidateResolver`。
///
/// 简单的默认实现，始终认为所有 Bean 定义都是合法的自动装配候选。
/// 更复杂的实现（如 `QualifierAnnotationAutowireCandidateResolver`）
/// 可根据注解或限定符进一步过滤。
#[derive(Debug, Default)]
pub struct SimpleAutowireCandidateResolver;

impl SimpleAutowireCandidateResolver {
    /// 创建新的 SimpleAutowireCandidateResolver。
    pub fn new() -> Self {
        Self
    }

    /// 判断指定的 Bean 定义是否可作为自动装配候选。
    ///
    /// 对应 Spring 的 `isAutowireCandidate(BeanDefinitionHolder, DependencyDescriptor)`。
    ///
    /// 简单实现直接委托给 `BeanDefinition::is_autowire_candidate()`。
    /// 返回 `true` 表示该 Bean 可以参与按类型自动装配。
    pub fn is_autowire_candidate(&self, definition: &dyn BeanDefinition) -> bool {
        definition.is_autowire_candidate()
    }
}
