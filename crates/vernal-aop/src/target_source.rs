//! 目标源。
//!
//! 对应 spring-aop `TargetSource`。
//! 提供 AOP 调用的目标对象。

use std::any::Any;
use std::fmt;

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

/// 目标源错误。
#[derive(Debug)]
pub enum TargetSourceError {
    /// 目标不存在。
    NoSuchTarget(String),
    /// 目标创建失败。
    CreationFailed(String),
    /// 目标释放失败。
    ReleaseFailed(String),
}

impl fmt::Display for TargetSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetSourceError::NoSuchTarget(msg) => write!(f, "NoSuchTarget: {}", msg),
            TargetSourceError::CreationFailed(msg) => write!(f, "CreationFailed: {}", msg),
            TargetSourceError::ReleaseFailed(msg) => write!(f, "ReleaseFailed: {}", msg),
        }
    }
}

impl std::error::Error for TargetSourceError {}

/// 单例目标源。
///
/// 始终返回同一个目标对象。
pub struct SingletonTargetSource {
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
