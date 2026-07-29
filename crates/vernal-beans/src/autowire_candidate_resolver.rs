//! AutowireCandidateResolver — Spring 风格的自动装配候选解析器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AutowireCandidateResolver`。
//!
//! 定义判断一个 Bean 定义是否可作为自动装配候选的策略接口。

use crate::bean_definition::BeanDefinition;
use crate::dependency_descriptor::DependencyDescriptor;

/// Spring 风格的自动装配候选解析器 trait。
///
/// 对应 Spring 的 `AutowireCandidateResolver`。
///
/// 判断指定的 Bean 定义是否可作为某依赖描述符的自动装配候选。
/// 默认实现总是返回 `true`。
///
/// ## 实现
///
/// - `SimpleAutowireCandidateResolver` — 简单实现，始终返回 `true`
/// - `QualifierAnnotationAutowireCandidateResolver` — 支持 `@Qualifier` 注解（尚未实现）
pub trait AutowireCandidateResolver: Send + Sync + std::fmt::Debug {
    /// 判断指定的 Bean 定义是否可作为自动装配候选。
    ///
    /// 对应 Spring 的 `boolean isAutowireCandidate(BeanDefinitionHolder bdHolder, DependencyDescriptor descriptor)`。
    ///
    /// # 参数
    ///
    /// * `definition` — Bean 定义
    /// * `descriptor` — 依赖描述符，提供请求的上下文
    ///
    /// # 返回
    ///
    /// `true` 表示该 Bean 定义可以作为自动装配候选。
    fn is_autowire_candidate(
        &self,
        definition: &dyn BeanDefinition,
        _descriptor: Option<&DependencyDescriptor>,
    ) -> bool {
        definition.is_autowire_candidate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependency_descriptor::DependencyDescriptor;
    use crate::root_bean_definition::RootBeanDefinition;

    #[derive(Debug)]
    struct AlwaysTrueResolver;

    impl AutowireCandidateResolver for AlwaysTrueResolver {
        fn is_autowire_candidate(
            &self,
            definition: &dyn BeanDefinition,
            _descriptor: Option<&DependencyDescriptor>,
        ) -> bool {
            definition.is_autowire_candidate()
        }
    }

    #[derive(Debug)]
    struct AlwaysFalseResolver;

    impl AutowireCandidateResolver for AlwaysFalseResolver {
        fn is_autowire_candidate(
            &self,
            _definition: &dyn BeanDefinition,
            _descriptor: Option<&DependencyDescriptor>,
        ) -> bool {
            false
        }
    }

    #[test]
    fn test_default_trait_impl() {
        #[derive(Debug)]
        struct DefaultResolver;
        impl AutowireCandidateResolver for DefaultResolver {}

        let resolver = DefaultResolver;
        let def = RootBeanDefinition::new();
        assert!(resolver.is_autowire_candidate(&def, None));
    }

    #[test]
    fn test_always_true_resolver() {
        let resolver = AlwaysTrueResolver;
        let def = RootBeanDefinition::new();
        assert!(resolver.is_autowire_candidate(&def, None));
    }

    #[test]
    fn test_always_false_resolver() {
        let resolver = AlwaysFalseResolver;
        let def = RootBeanDefinition::new();
        assert!(!resolver.is_autowire_candidate(&def, None));
    }

    #[test]
    fn test_with_descriptor() {
        let resolver = AlwaysTrueResolver;
        let def = RootBeanDefinition::new();
        let descriptor = DependencyDescriptor::for_field(
            std::any::TypeId::of::<String>(),
            "alloc::string::String",
        );
        assert!(resolver.is_autowire_candidate(&def, Some(&descriptor)));
    }
}
