//! 切面实例工厂。
//!
//! 对应 spring-aop `AspectInstanceFactory`。
//! 提供切面实例的创建和管理。

use std::any::Any;
use std::fmt;

/// 切面实例工厂接口。
///
/// 对应 spring-aop `AspectInstanceFactory`。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::AspectInstanceFactory;
///
/// let factory = SingletonAspectInstanceFactory::new(my_aspect);
/// let aspect = factory.get_aspect_instance();
/// ```
pub trait AspectInstanceFactory: Send + Sync + 'static {
    /// 获取切面实例。
    fn get_aspect_instance(&self) -> Result<Box<dyn Any>, AspectInstanceError>;

    /// 获取切面类型名。
    fn get_aspect_type(&self) -> &str;

    /// 是否为单例。
    fn is_singleton(&self) -> bool {
        true
    }
}

/// 切面实例错误。
#[derive(Debug)]
pub enum AspectInstanceError {
    /// 实例创建失败。
    CreationFailed(String),
    /// 类型不匹配。
    TypeMismatch(String),
}

impl fmt::Display for AspectInstanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AspectInstanceError::CreationFailed(msg) => {
                write!(f, "Aspect instance creation failed: {}", msg)
            }
            AspectInstanceError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
        }
    }
}

impl std::error::Error for AspectInstanceError {}

/// 单例切面实例工厂。
pub struct SingletonAspectInstanceFactory {
    // 对标 Spring AOP 的 API 脚手架：保留切面实例引用，当前 get_aspect_instance 不返回克隆实例。
    #[allow(dead_code)]
    instance: Box<dyn Any + Send + Sync>,
    aspect_type: String,
}

impl SingletonAspectInstanceFactory {
    /// 创建单例切面实例工厂。
    pub fn new<T: Any + Send + Sync + 'static>(instance: T) -> Self {
        Self {
            instance: Box::new(instance),
            aspect_type: std::any::type_name::<T>().to_string(),
        }
    }
}

impl AspectInstanceFactory for SingletonAspectInstanceFactory {
    fn get_aspect_instance(&self) -> Result<Box<dyn Any>, AspectInstanceError> {
        // 单例无法克隆，返回错误
        Err(AspectInstanceError::CreationFailed(
            "Cannot clone singleton instance".to_string(),
        ))
    }

    fn get_aspect_type(&self) -> &str {
        &self.aspect_type
    }

    fn is_singleton(&self) -> bool {
        true
    }
}

/// 延迟切面实例工厂。
pub struct LazyAspectInstanceFactory<
    F: Fn() -> Result<Box<dyn Any + Send + Sync>, AspectInstanceError> + Send + Sync + 'static,
> {
    factory: F,
    aspect_type: String,
}

impl<F: Fn() -> Result<Box<dyn Any + Send + Sync>, AspectInstanceError> + Send + Sync + 'static>
    LazyAspectInstanceFactory<F>
{
    /// 创建延迟切面实例工厂。
    pub fn new(factory: F, aspect_type: impl Into<String>) -> Self {
        Self {
            factory,
            aspect_type: aspect_type.into(),
        }
    }
}

impl<F: Fn() -> Result<Box<dyn Any + Send + Sync>, AspectInstanceError> + Send + Sync + 'static>
    AspectInstanceFactory for LazyAspectInstanceFactory<F>
{
    fn get_aspect_instance(&self) -> Result<Box<dyn Any>, AspectInstanceError> {
        (self.factory)().map(|b| b as Box<dyn Any>)
    }

    fn get_aspect_type(&self) -> &str {
        &self.aspect_type
    }

    fn is_singleton(&self) -> bool {
        false
    }
}

/// 元数据感知切面实例工厂。
///
/// 对应 spring-aop `MetadataAwareAspectInstanceFactory`。
pub trait MetadataAwareAspectInstanceFactory: AspectInstanceFactory {
    /// 获取切面类名。
    fn get_aspect_class_name(&self) -> &str;

    /// 获取切面元数据。
    fn get_aspect_metadata(&self) -> Option<&dyn Any> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspect_instance_error_display() {
        let err = AspectInstanceError::CreationFailed("test".to_string());
        assert_eq!(format!("{}", err), "Aspect instance creation failed: test");
    }

    #[test]
    fn lazy_aspect_instance_factory() {
        let factory = LazyAspectInstanceFactory::new(|| Ok(Box::new(42i32)), "i32");
        assert_eq!(factory.get_aspect_type(), "i32");
        assert!(!factory.is_singleton());

        let instance = factory.get_aspect_instance().unwrap();
        let value = instance.downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 42);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn singleton_aspect_instance_factory_new() {
        let factory = SingletonAspectInstanceFactory::new(42i32);
        assert!(factory.is_singleton());
    }

    #[test]
    fn singleton_aspect_instance_factory_get_aspect_type() {
        let factory = SingletonAspectInstanceFactory::new(42i32);
        let type_name = factory.get_aspect_type();
        assert!(!type_name.is_empty());
    }

    #[test]
    fn singleton_aspect_instance_factory_get_aspect_instance() {
        let factory = SingletonAspectInstanceFactory::new(42i32);
        let result = factory.get_aspect_instance();
        assert!(result.is_err());
    }

    #[test]
    fn aspect_instance_error_creation_failed_display() {
        let err = AspectInstanceError::CreationFailed("test error".to_string());
        assert!(format!("{}", err).contains("test error"));
    }

    #[test]
    fn aspect_instance_error_type_mismatch_display() {
        let err = AspectInstanceError::TypeMismatch("test error".to_string());
        assert!(format!("{}", err).contains("test error"));
    }

    #[test]
    fn aspect_instance_error_debug() {
        let err = AspectInstanceError::CreationFailed("test".to_string());
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty());
    }

    #[test]
    fn aspect_instance_error_trait() {
        let err = AspectInstanceError::CreationFailed("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn lazy_aspect_instance_factory_creation_failed() {
        let factory = LazyAspectInstanceFactory::new(
            || Err(AspectInstanceError::CreationFailed("test".to_string())),
            "TestType",
        );
        let result = factory.get_aspect_instance();
        assert!(result.is_err());
    }

    #[test]
    fn lazy_aspect_instance_factory_debug() {
        let factory = LazyAspectInstanceFactory::new(|| Ok(Box::new(42i32)), "i32");
        // LazyAspectInstanceFactory doesn't implement Debug, but we can test its methods
        assert_eq!(factory.get_aspect_type(), "i32");
    }
}
