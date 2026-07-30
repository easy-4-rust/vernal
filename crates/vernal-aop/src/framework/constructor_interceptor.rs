//! 构造器拦截器。
//!
//! 对应 spring-aop `ConstructorInterceptor`。
//! 拦截构造器调用。

use std::any::Any;
use std::fmt;

/// 构造器调用接口。
///
/// 对应 spring-aop `ConstructorInvocation`。
pub trait ConstructorInvocation: Send + Sync + 'static {
    /// 获取构造器参数。
    fn get_arguments(&self) -> Vec<Box<dyn Any>>;

    /// 获取构造器所属类型名。
    fn get_constructor_type(&self) -> &str;

    /// 继续执行构造器链。
    fn proceed(&self) -> Result<Box<dyn Any>, ConstructorInvocationError>;
}

/// 构造器调用错误。
#[derive(Debug)]
pub enum ConstructorInvocationError {
    /// 构造失败。
    ConstructionFailed(String),
    /// 参数错误。
    ArgumentMismatch(String),
}

impl fmt::Display for ConstructorInvocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConstructorInvocationError::ConstructionFailed(msg) => {
                write!(f, "Construction failed: {}", msg)
            }
            ConstructorInvocationError::ArgumentMismatch(msg) => {
                write!(f, "Argument mismatch: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConstructorInvocationError {}

/// 构造器拦截器接口。
///
/// 对应 spring-aop `ConstructorInterceptor`。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::ConstructorInterceptor;
///
/// struct MyConstructorInterceptor;
///
/// impl ConstructorInterceptor for MyConstructorInterceptor {
///     fn construct(&self, invocation: &dyn ConstructorInvocation) -> Result<Box<dyn std::any::Any>, ConstructorInvocationError> {
///         println!("Before construction");
///         let result = invocation.proceed()?;
///         println!("After construction");
///         Ok(result)
///     }
/// }
/// ```
pub trait ConstructorInterceptor: Send + Sync + 'static {
    /// 拦截构造器调用。
    fn construct(
        &self,
        invocation: &dyn ConstructorInvocation,
    ) -> Result<Box<dyn Any>, ConstructorInvocationError>;
}

/// 基于闭包的构造器拦截器。
pub struct FnConstructorInterceptor<
    F: Fn(&dyn ConstructorInvocation) -> Result<Box<dyn Any>, ConstructorInvocationError>
        + Send
        + Sync
        + 'static,
> {
    interceptor: F,
}

impl<
        F: Fn(&dyn ConstructorInvocation) -> Result<Box<dyn Any>, ConstructorInvocationError>
            + Send
            + Sync
            + 'static,
    > FnConstructorInterceptor<F>
{
    /// 创建基于闭包的构造器拦截器。
    pub fn new(interceptor: F) -> Self {
        Self { interceptor }
    }
}

impl<
        F: Fn(&dyn ConstructorInvocation) -> Result<Box<dyn Any>, ConstructorInvocationError>
            + Send
            + Sync
            + 'static,
    > ConstructorInterceptor for FnConstructorInterceptor<F>
{
    fn construct(
        &self,
        invocation: &dyn ConstructorInvocation,
    ) -> Result<Box<dyn Any>, ConstructorInvocationError> {
        (self.interceptor)(invocation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestConstructorInvocation;

    impl ConstructorInvocation for TestConstructorInvocation {
        fn get_arguments(&self) -> Vec<Box<dyn Any>> {
            vec![]
        }

        fn get_constructor_type(&self) -> &str {
            "TestType"
        }

        fn proceed(&self) -> Result<Box<dyn Any>, ConstructorInvocationError> {
            Ok(Box::new(42i32))
        }
    }

    #[test]
    fn constructor_invocation_error_display() {
        let err = ConstructorInvocationError::ConstructionFailed("test".to_string());
        assert_eq!(format!("{}", err), "Construction failed: test");
    }

    #[test]
    fn fn_constructor_interceptor() {
        let interceptor = FnConstructorInterceptor::new(|inv| {
            println!("Constructing {}", inv.get_constructor_type());
            inv.proceed()
        });

        let invocation = TestConstructorInvocation;
        let result = interceptor.construct(&invocation).unwrap();
        let value = result.downcast_ref::<i32>().unwrap();
        assert_eq!(*value, 42);
    }
}
