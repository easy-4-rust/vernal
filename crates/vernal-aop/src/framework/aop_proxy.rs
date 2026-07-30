//! AOP 代理。
//!
//! 对应 spring-aop `AopProxy`。
//! 创建实际的代理对象。

use std::any::Any;
use std::fmt;

/// AOP 代理接口。
///
/// 对应 spring-aop `AopProxy`。
///
/// # 实现方式
///
/// - JDK 动态代理（Java 特有）
/// - CGLIB 代理（Java 特有）
/// - 过程宏代理（Rust 原生）
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::AopProxy;
///
/// let proxy = my_aop_proxy.get_proxy();
/// ```
pub trait AopProxy: Send + Sync + 'static {
    /// 创建新的代理对象。
    fn get_proxy(&self) -> Result<Box<dyn Any>, AopProxyError>;

    /// 获取代理对象的类型名。
    fn proxy_type_name(&self) -> &str;
}

/// AOP 代理错误。
#[derive(Debug)]
pub enum AopProxyError {
    /// 代理创建失败。
    CreationFailed(String),
    /// 目标不存在。
    TargetNotFound(String),
    /// 配置错误。
    ConfigurationError(String),
}

impl fmt::Display for AopProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AopProxyError::CreationFailed(msg) => write!(f, "AopProxy creation failed: {}", msg),
            AopProxyError::TargetNotFound(msg) => write!(f, "Target not found: {}", msg),
            AopProxyError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AopProxyError {}

/// AOP 代理工厂。
///
/// 对应 spring-aop `AopProxyFactory`。
pub trait AopProxyFactory: Send + Sync + 'static {
    /// 创建 AOP 代理。
    fn create_aop_proxy(&self, target: Box<dyn Any>) -> Result<Box<dyn AopProxy>, AopProxyError>;
}

/// 基于闭包的 AOP 代理。
pub struct FnAopProxy<F: Fn() -> Result<Box<dyn Any>, AopProxyError> + Send + Sync + 'static> {
    factory: F,
    type_name: String,
}

impl<F: Fn() -> Result<Box<dyn Any>, AopProxyError> + Send + Sync + 'static> FnAopProxy<F> {
    /// 创建基于闭包的 AOP 代理。
    pub fn new(factory: F, type_name: impl Into<String>) -> Self {
        Self {
            factory,
            type_name: type_name.into(),
        }
    }
}

impl<F: Fn() -> Result<Box<dyn Any>, AopProxyError> + Send + Sync + 'static> AopProxy
    for FnAopProxy<F>
{
    fn get_proxy(&self) -> Result<Box<dyn Any>, AopProxyError> {
        (self.factory)()
    }

    fn proxy_type_name(&self) -> &str {
        &self.type_name
    }
}

impl fmt::Debug for FnAopProxy<fn() -> Result<Box<dyn Any>, AopProxyError>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FnAopProxy")
            .field("type_name", &self.type_name)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aop_proxy_error_display() {
        let err = AopProxyError::CreationFailed("test error".to_string());
        assert_eq!(format!("{}", err), "AopProxy creation failed: test error");
    }

    #[test]
    fn fn_aop_proxy() {
        let proxy = FnAopProxy::new(|| Ok(Box::new(42i32)), "TestProxy");
        assert_eq!(proxy.proxy_type_name(), "TestProxy");

        let result = proxy.get_proxy().unwrap();
        let value = result.downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 42);
    }
}
