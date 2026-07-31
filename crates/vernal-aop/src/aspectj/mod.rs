//! AspectJ 模块。
//!
//! 对应 spring-aop `aspectj` 包。

pub mod aspect_instance_factory;

// Re-export
pub use aspect_instance_factory::{
    AspectInstanceError, AspectInstanceFactory, LazyAspectInstanceFactory,
    MetadataAwareAspectInstanceFactory, SingletonAspectInstanceFactory,
};
