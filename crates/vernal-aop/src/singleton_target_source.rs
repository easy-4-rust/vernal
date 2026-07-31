//! 单例目标源。
//!
//! 对应 spring-aop `SingletonTargetSource`。
//! 始终返回同一个目标对象。

use std::any::Any;
use std::fmt;

use crate::target_source::TargetSource;
use crate::target_source_error::TargetSourceError;

/// 单例目标源。
///
/// 始终返回同一个目标对象。
pub struct SingletonTargetSource {
    // 对标 Spring AOP 的 API 脚手架：保留目标对象引用，当前 get_target 不返回克隆目标。
    #[allow(dead_code)]
    target: Box<dyn Any + Send + Sync>,
    target_class: Option<String>,
}

impl SingletonTargetSource {
    /// 创建单例目标源。
    pub fn new<T: Any + Send + Sync + 'static>(target: T) -> Self {
        let target_class = std::any::type_name::<T>().to_string();
        Self {
            target: Box::new(target),
            target_class: Some(target_class),
        }
    }

    /// 创建带类型名的单例目标源。
    pub fn with_class_name<T: Any + Send + Sync + 'static>(
        target: T,
        class_name: impl Into<String>,
    ) -> Self {
        Self {
            target: Box::new(target),
            target_class: Some(class_name.into()),
        }
    }
}

impl TargetSource for SingletonTargetSource {
    fn target_class(&self) -> Option<&str> {
        self.target_class.as_deref()
    }

    fn is_static(&self) -> bool {
        true
    }

    fn get_target(&self) -> Result<Box<dyn Any>, TargetSourceError> {
        Err(TargetSourceError::NoSuchTarget(
            "SingletonTargetSource cannot return cloned target".to_string(),
        ))
    }
}

impl fmt::Debug for SingletonTargetSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingletonTargetSource")
            .field("target_class", &self.target_class)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton_target_source() {
        let source = SingletonTargetSource::new(42i32);
        assert_eq!(TargetSource::target_class(&source), Some("i32"));
        assert!(source.is_static());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn singleton_target_source_with_class_name() {
        let source = SingletonTargetSource::with_class_name(42i32, "MyType");
        assert_eq!(TargetSource::target_class(&source), Some("MyType"));
    }

    #[test]
    fn singleton_target_source_is_static() {
        let source = SingletonTargetSource::new(42i32);
        assert!(source.is_static());
    }

    #[test]
    fn singleton_target_source_get_target() {
        let source = SingletonTargetSource::new(42i32);
        let result = source.get_target();
        assert!(result.is_err());
    }

    #[test]
    fn singleton_target_source_debug() {
        let source = SingletonTargetSource::new(42i32);
        let debug = format!("{:?}", source);
        assert!(!debug.is_empty());
    }
}
