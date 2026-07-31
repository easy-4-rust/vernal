//! 连接点。
//!
//! 对应 aopalliance `Joinpoint`。
//! 表示 AOP 中的运行时连接点。

use std::any::Any;
use std::fmt;

use crate::Operation;

/// 连接点接口。
///
/// 对应 aopalliance `Joinpoint`。
///
/// # 概念
///
/// - **静态连接点**：程序中的位置（如方法、构造器）
/// - **运行时连接点**：静态连接点上的事件（如方法调用）
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::Joinpoint;
///
/// // 获取连接点的静态部分
/// let operation = joinpoint.get_static_part();
///
/// // 继续执行拦截器链
/// let result = joinpoint.proceed();
/// ```
pub trait Joinpoint: Send + Sync + 'static {
    /// 继续执行拦截器链。
    fn proceed(&self) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取当前连接点的静态部分（操作描述）。
    fn get_static_part(&self) -> &Operation;

    /// 获取目标对象（如果有）。
    fn get_this(&self) -> Option<&dyn Any> {
        None
    }
}

/// 方法调用接口。
///
/// 对应 aopalliance `MethodInvocation`。
///
/// # 用法
///
/// ```rust,ignore
/// use vernal_aop::MethodInvocation;
///
/// // 获取方法名
/// let method = invocation.get_method();
///
/// // 获取参数
/// let args = invocation.get_arguments();
///
/// // 继续执行
/// let result = invocation.proceed();
/// ```
pub trait MethodInvocation: Joinpoint {
    /// 获取方法名。
    fn get_method(&self) -> &str;

    /// 获取参数列表。
    fn get_arguments(&self) -> Vec<Box<dyn Any>>;
}

/// 调用链推进器。
///
/// 对应 vernal-aop 的 `Next`，用于推进拦截器链。
pub struct InvocationChain {
    operation: Operation,
    interceptors: Vec<Box<dyn Fn(&dyn Any) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>>,
    current_index: usize,
}

impl InvocationChain {
    /// 创建新的调用链。
    pub fn new(operation: Operation) -> Self {
        Self {
            operation,
            interceptors: Vec::new(),
            current_index: 0,
        }
    }

    /// 添加拦截器。
    pub fn add_interceptor(
        &mut self,
        interceptor: Box<dyn Fn(&dyn Any) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
    ) {
        self.interceptors.push(interceptor);
    }

    /// 获取当前操作。
    pub fn operation(&self) -> &Operation {
        &self.operation
    }
}

impl Joinpoint for InvocationChain {
    fn proceed(&self) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
        // 这里需要实现拦截器链的推进逻辑
        // 简化实现：如果没有拦截器，返回错误
        Err("No target implementation".into())
    }

    fn get_static_part(&self) -> &Operation {
        &self.operation
    }
}

impl fmt::Debug for InvocationChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvocationChain")
            .field("operation", &self.operation)
            .field("interceptor_count", &self.interceptors.len())
            .field("current_index", &self.current_index)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_chain_creation() {
        let op = Operation::new("Service", "method");
        let chain = InvocationChain::new(op.clone());

        assert_eq!(chain.operation().component(), "Service");
        assert_eq!(chain.operation().method(), "method");
    }

    #[test]
    fn invocation_chain_add_interceptor() {
        let op = Operation::new("Service", "method");
        let mut chain = InvocationChain::new(op);

        chain.add_interceptor(Box::new(|_| Ok(Box::new(42i32))));
        assert_eq!(chain.interceptors.len(), 1);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    struct TestJoinpoint {
        operation: Operation,
    }

    impl Joinpoint for TestJoinpoint {
        fn proceed(&self) -> Result<Box<dyn Any>, Box<dyn std::error::Error + Send + Sync>> {
            Ok(Box::new(42i32))
        }

        fn get_static_part(&self) -> &Operation {
            &self.operation
        }
    }

    #[test]
    fn joinpoint_proceed() {
        let op = Operation::new("Service", "method");
        let jp = TestJoinpoint { operation: op };
        let result = jp.proceed();
        assert!(result.is_ok());
    }

    #[test]
    fn joinpoint_get_static_part() {
        let op = Operation::new("Service", "method");
        let jp = TestJoinpoint { operation: op };
        assert_eq!(jp.get_static_part().component(), "Service");
    }

    #[test]
    fn joinpoint_get_this_default() {
        let op = Operation::new("Service", "method");
        let jp = TestJoinpoint { operation: op };
        assert!(jp.get_this().is_none());
    }

    #[test]
    fn invocation_chain_debug() {
        let op = Operation::new("Service", "method");
        let chain = InvocationChain::new(op);
        let debug = format!("{:?}", chain);
        assert!(!debug.is_empty());
    }

    #[test]
    fn invocation_chain_get_static_part() {
        let op = Operation::new("Service", "method");
        let chain = InvocationChain::new(op);
        assert_eq!(chain.get_static_part().component(), "Service");
    }

    #[test]
    fn invocation_chain_proceed_returns_error() {
        let op = Operation::new("Service", "method");
        let chain = InvocationChain::new(op);
        let result = chain.proceed();
        assert!(result.is_err());
    }
}
