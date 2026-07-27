//! 对标 `org.springframework.beans.factory.aspectj` 包。
//!
//! 提供 AspectJ 可配置对象依赖注入切面的 Rust 等价实现。

mod configurable_object;
mod abstract_dependency_injection_aspect;
mod annotation_bean_configurer_aspect;

pub use configurable_object::ConfigurableObject;
pub use abstract_dependency_injection_aspect::AbstractDependencyInjectionAspect;
pub use annotation_bean_configurer_aspect::AnnotationBeanConfigurerAspect;
