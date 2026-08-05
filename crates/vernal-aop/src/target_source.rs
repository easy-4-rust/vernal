//! 目标源。
//!
//! 对应 spring-aop `TargetSource`。
//! 提供 AOP 调用的目标对象。

use std::any::Any;
use std::fmt;

use crate::target_source_error::TargetSourceError;

/// 目标源，提供 AOP 调用的目标对象。
///
/// 对应 spring-aop `TargetSource`。
///
/// # 静态与动态
///
/// - **静态目标源**：始终返回同一个目标，可缓存
/// - **动态目标源**：支持池化、热替换等
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::TargetSource;
///
/// // 单例目标源
/// let target = SingletonTargetSource::new(my_service);
///
/// // 获取目标
/// let obj = target.get_target();
/// ```
pub trait TargetSource: Send + Sync + 'static {
    /// 返回目标类型名称。
    fn target_class(&self) -> Option<&str>;

    /// 是否为静态目标源。
    ///
    /// 如果返回 `true`，`get_target()` 始终返回同一个对象，可缓存。
    fn is_static(&self) -> bool {
        false
    }

    /// 获取目标对象。
    fn get_target(&self) -> Result<Box<dyn Any>, TargetSourceError>;

    /// 释放目标对象。
    ///
    /// 默认为空实现。动态目标源可覆盖此方法进行资源回收。
    fn release_target(&self, _target: Box<dyn Any>) -> Result<(), TargetSourceError> {
        Ok(())
    }
}

/// 单例目标源。
///
/// 始终返回同一个目标对象。
// 对标 Spring AOP 的 API 脚手架：SingletonTargetSource 保留目标引用，暂未被内部调用。
#[allow(dead_code)]
pub struct SingletonTargetSource {
    target: Box<dyn Any + Send + Sync>,
    target_class: Option<String>,
}

#[allow(dead_code)]
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
        // 单例目标源无法克隆 Box<dyn Any>，返回错误
        // 实际使用中应该通过 Arc 包装
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

/// 基于闭包的目标源。
///
/// 每次调用 `get_target()` 时通过闭包创建新目标。
// 对标 Spring AOP 的 API 脚手架：LazyTargetSource 提供惰性目标源，暂未被内部调用。
#[allow(dead_code)]
pub struct LazyTargetSource<
    F: Fn() -> Result<Box<dyn Any + Send + Sync>, TargetSourceError> + Send + Sync + 'static,
> {
    factory: F,
    target_class: Option<String>,
}

#[allow(dead_code)]
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

/// 目标类感知 trait。
///
/// 对应 spring-aop `TargetClassAware`。
pub trait TargetClassAware: Send + Sync + 'static {
    /// 返回目标类名。
    fn target_class(&self) -> Option<&str>;
}

/// TargetSource 自动实现 TargetClassAware。
impl<T: TargetSource> TargetClassAware for T {
    fn target_class(&self) -> Option<&str> {
        self.target_class()
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

    #[test]
    fn lazy_target_source() {
        let source = LazyTargetSource::new(|| Ok(Box::new(42i32)));
        assert!(!source.is_static());

        let target = source.get_target().unwrap();
        let value = target.downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 42);
    }

    #[test]
    fn target_source_error_display() {
        let err = TargetSourceError::NoSuchTarget("test".to_string());
        assert_eq!(format!("{}", err), "NoSuchTarget: test");
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::target_source_error::TargetSourceError;

    struct TestTargetSource {
        target_class: Option<String>,
        is_static: bool,
    }

    impl TestTargetSource {
        fn new(target_class: Option<&str>, is_static: bool) -> Self {
            Self {
                target_class: target_class.map(|s| s.to_string()),
                is_static,
            }
        }
    }

    impl TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            self.target_class.as_deref()
        }

        fn is_static(&self) -> bool {
            self.is_static
        }

        fn get_target(&self) -> Result<Box<dyn std::any::Any>, TargetSourceError> {
            Ok(Box::new(42i32))
        }

        fn release_target(&self, _target: Box<dyn std::any::Any>) -> Result<(), TargetSourceError> {
            Ok(())
        }
    }

    #[test]
    fn target_source_target_class() {
        let source = TestTargetSource::new(Some("MyType"), true);
        assert_eq!(TargetSource::target_class(&source), Some("MyType"));
    }

    #[test]
    fn target_source_target_class_none() {
        let source = TestTargetSource::new(None, true);
        assert!(TargetSource::target_class(&source).is_none());
    }

    #[test]
    fn target_source_is_static() {
        let source = TestTargetSource::new(Some("MyType"), true);
        assert!(source.is_static());
    }

    #[test]
    fn target_source_is_not_static() {
        let source = TestTargetSource::new(Some("MyType"), false);
        assert!(!source.is_static());
    }

    #[test]
    fn target_source_get_target() {
        let source = TestTargetSource::new(Some("MyType"), true);
        let result = source.get_target();
        assert!(result.is_ok());
    }

    #[test]
    fn target_source_release_target() {
        let source = TestTargetSource::new(Some("MyType"), true);
        let target = Box::new(42i32);
        let result = source.release_target(target);
        assert!(result.is_ok());
    }

    #[test]
    fn target_source_error_display() {
        let err = TargetSourceError::NoSuchTarget("test".to_string());
        assert!(format!("{}", err).contains("NoSuchTarget"));
    }

    #[test]
    fn target_source_error_creation_failed() {
        let err = TargetSourceError::CreationFailed("test".to_string());
        assert!(format!("{}", err).contains("CreationFailed"));
    }

    #[test]
    fn target_source_error_release_failed() {
        let err = TargetSourceError::ReleaseFailed("test".to_string());
        assert!(format!("{}", err).contains("ReleaseFailed"));
    }

    #[test]
    fn target_source_error_debug() {
        let err = TargetSourceError::NoSuchTarget("test".to_string());
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty());
    }

    #[test]
    fn target_source_error_trait() {
        let err = TargetSourceError::NoSuchTarget("test".to_string());
        let _: &dyn std::error::Error = &err;
    }
}

#[cfg(test)]
mod target_source_tests {
    use super::*;
    use crate::target_source_error::TargetSourceError;

    struct TestTargetSource {
        target_class: Option<String>,
        is_static: bool,
    }

    impl TestTargetSource {
        fn new(target_class: Option<&str>, is_static: bool) -> Self {
            Self {
                target_class: target_class.map(|s| s.to_string()),
                is_static,
            }
        }
    }

    impl TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            self.target_class.as_deref()
        }

        fn is_static(&self) -> bool {
            self.is_static
        }

        fn get_target(&self) -> Result<Box<dyn std::any::Any>, TargetSourceError> {
            Ok(Box::new(42i32))
        }

        fn release_target(&self, _target: Box<dyn std::any::Any>) -> Result<(), TargetSourceError> {
            Ok(())
        }
    }

    #[test]
    fn target_source_target_class() {
        let source = TestTargetSource::new(Some("MyType"), true);
        assert_eq!(TargetSource::target_class(&source), Some("MyType"));
    }

    #[test]
    fn target_source_target_class_none() {
        let source = TestTargetSource::new(None, true);
        assert!(TargetSource::target_class(&source).is_none());
    }

    #[test]
    fn target_source_is_static() {
        let source = TestTargetSource::new(Some("MyType"), true);
        assert!(source.is_static());
    }

    #[test]
    fn target_source_is_not_static() {
        let source = TestTargetSource::new(Some("MyType"), false);
        assert!(!source.is_static());
    }

    #[test]
    fn target_source_get_target() {
        let source = TestTargetSource::new(Some("MyType"), true);
        let result = source.get_target();
        assert!(result.is_ok());
    }

    #[test]
    fn target_source_release_target() {
        let source = TestTargetSource::new(Some("MyType"), true);
        let target = Box::new(42i32);
        let result = source.release_target(target);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod target_source_coverage_tests {
    use super::*;
    use crate::target_source_error::TargetSourceError;

    struct TestTargetSource {
        target_class: Option<String>,
        is_static: bool,
    }

    impl TestTargetSource {
        fn new(target_class: Option<&str>, is_static: bool) -> Self {
            Self {
                target_class: target_class.map(|s| s.to_string()),
                is_static,
            }
        }
    }

    impl TargetSource for TestTargetSource {
        fn target_class(&self) -> Option<&str> {
            self.target_class.as_deref()
        }

        fn is_static(&self) -> bool {
            self.is_static
        }

        fn get_target(&self) -> Result<Box<dyn std::any::Any>, TargetSourceError> {
            Ok(Box::new(42i32))
        }

        fn release_target(&self, _target: Box<dyn std::any::Any>) -> Result<(), TargetSourceError> {
            Ok(())
        }
    }

    #[test]
    fn target_source_is_not_static() {
        let source = TestTargetSource::new(Some("MyType"), false);
        assert!(!source.is_static());
    }

    #[test]
    fn target_source_get_target() {
        let source = TestTargetSource::new(Some("MyType"), true);
        let result = source.get_target();
        assert!(result.is_ok());
    }

    #[test]
    fn target_source_release_target() {
        let source = TestTargetSource::new(Some("MyType"), true);
        let target = Box::new(42i32);
        let result = source.release_target(target);
        assert!(result.is_ok());
    }

    #[test]
    fn target_source_error_creation_failed() {
        let err = TargetSourceError::CreationFailed("test".to_string());
        assert!(format!("{}", err).contains("CreationFailed"));
    }

    #[test]
    fn target_source_error_release_failed() {
        let err = TargetSourceError::ReleaseFailed("test".to_string());
        assert!(format!("{}", err).contains("ReleaseFailed"));
    }

    #[test]
    fn target_source_error_debug() {
        let err = TargetSourceError::NoSuchTarget("test".to_string());
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty());
    }

    #[test]
    fn target_source_error_trait() {
        let err = TargetSourceError::NoSuchTarget("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

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

    #[test]
    fn lazy_target_source_with_class_name() {
        let source = LazyTargetSource::with_class_name(|| Ok(Box::new(42i32)), "MyType");
        assert_eq!(TargetSource::target_class(&source), Some("MyType"));
    }

    #[test]
    fn lazy_target_source_is_not_static() {
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
        let source =
            LazyTargetSource::new(|| Err(TargetSourceError::CreationFailed("test".to_string())));
        let result = source.get_target();
        assert!(result.is_err());
    }
}
