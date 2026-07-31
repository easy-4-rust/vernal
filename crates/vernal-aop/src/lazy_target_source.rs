//! 延迟目标源。
//!
//! 对应 spring-aop `LazyTargetSource`。
//! 每次调用 `get_target()` 时通过闭包创建新目标。

use std::any::Any;

use crate::target_source::TargetSource;
use crate::target_source_error::TargetSourceError;

/// 基于闭包的目标源。
///
/// 每次调用 `get_target()` 时通过闭包创建新目标。
pub struct LazyTargetSource<F: Fn() -> Result<Box<dyn Any + Send + Sync>, TargetSourceError> + Send + Sync + 'static> {
    factory: F,
    target_class: Option<String>,
}

impl<F: Fn() -> Result<Box<dyn Any + Send + Sync>, TargetSourceError> + Send + Sync + 'static>
    LazyTargetSource<F>
{
    /// 创建延迟目标源。
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            target_class: None,
        }
    }

    /// 创建带类型名的延迟目标源。
    pub fn with_class_name(factory: F, class_name: impl Into<String>) -> Self {
        Self {
            factory,
            target_class: Some(class_name.into()),
        }
    }
}

impl<F: Fn() -> Result<Box<dyn Any + Send + Sync>, TargetSourceError> + Send + Sync + 'static>
    TargetSource for LazyTargetSource<F>
{
    fn target_class(&self) -> Option<&str> {
        self.target_class.as_deref()
    }

    fn is_static(&self) -> bool {
        false
    }

    fn get_target(&self) -> Result<Box<dyn Any>, TargetSourceError> {
        (self.factory)().map(|b| b as Box<dyn Any>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lazy_target_source() {
        let source = LazyTargetSource::new(|| Ok(Box::new(42i32)));
        assert!(!source.is_static());

        let target = source.get_target().unwrap();
        let value = target.downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 42);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::target_source_error::TargetSourceError;

    #[test]
    fn lazy_target_source_target_class() {
        let source = LazyTargetSource::new(|| Ok(Box::new(42i32)));
        assert!(source.target_class().is_none());
    }

    #[test]
    fn lazy_target_source_with_class_name() {
        let source = LazyTargetSource::with_class_name(|| Ok(Box::new(42i32)), "MyType");
        assert_eq!(source.target_class(), Some("MyType"));
    }

    #[test]
    fn lazy_target_source_is_static() {
        let source = LazyTargetSource::new(|| Ok(Box::new(42i32)));
        assert!(!source.is_static());
    }

    #[test]
    fn lazy_target_source_get_target() {
        let source = LazyTargetSource::new(|| Ok(Box::new(42i32)));
        let result = source.get_target();
        assert!(result.is_ok());
    }

    #[test]
    fn lazy_target_source_get_target_error() {
        let source = LazyTargetSource::new(|| {
            Err(TargetSourceError::CreationFailed("test".to_string()))
        });
        let result = source.get_target();
        assert!(result.is_err());
    }
}
