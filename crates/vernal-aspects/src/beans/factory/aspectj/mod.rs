//! 对标 `org.springframework.beans.factory.aspectj` 包。
//!
//! 提供 AspectJ 可配置对象依赖注入切面的 Rust 等价实现。

mod configurable_object;
mod abstract_dependency_injection_aspect;
mod annotation_bean_configurer_aspect;

pub use configurable_object::ConfigurableObject;
pub use abstract_dependency_injection_aspect::AbstractDependencyInjectionAspect;
pub use annotation_bean_configurer_aspect::AnnotationBeanConfigurerAspect;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configurable_object() {
        fn assert_impl<T: ConfigurableObject>() {}
        struct Test;
        impl ConfigurableObject for Test {}
        assert_impl::<Test>();
    }

    #[test]
    fn test_dependency_injection_aspect() {
        struct Test;
        impl ConfigurableObject for Test {}
        let aspect = AbstractDependencyInjectionAspect::<Test>::new();
        let _ = aspect;
    }

    #[test]
    fn test_annotation_bean_configurer_aspect() {
        struct Test;
        impl ConfigurableObject for Test {}
        let aspect = AnnotationBeanConfigurerAspect::<Test>::new();
        let _ = aspect;
    }
}
