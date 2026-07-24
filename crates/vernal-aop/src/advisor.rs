//! 切面顾问对象。

use std::sync::Arc;

use crate::{Interceptor, Operation, Pointcut};

/// 将一个切点、一个环绕拦截器和执行顺序组合成切面声明。
///
/// `order` 越小越先进入调用链、越后退出；相同顺序由注册顺序稳定决定。
#[derive(Clone)]
pub struct Advisor {
    pointcut: Arc<dyn Pointcut>,
    interceptor: Arc<dyn Interceptor>,
    order: i32,
}

impl Advisor {
    /// 使用具体切点与拦截器创建顾问。
    #[must_use]
    pub fn new<P, I>(pointcut: P, interceptor: I, order: i32) -> Self
    where
        P: Pointcut,
        I: Interceptor,
    {
        Self {
            pointcut: Arc::new(pointcut),
            interceptor: Arc::new(interceptor),
            order,
        }
    }

    /// 使用已经共享的 trait object 创建顾问。
    #[must_use]
    pub fn shared(
        pointcut: Arc<dyn Pointcut>,
        interceptor: Arc<dyn Interceptor>,
        order: i32,
    ) -> Self {
        Self {
            pointcut,
            interceptor,
            order,
        }
    }

    /// 返回切点是否匹配操作。
    #[must_use]
    pub fn matches(&self, operation: &Operation) -> bool {
        self.pointcut.matches(operation)
    }

    /// 返回执行顺序。
    #[must_use]
    pub const fn order(&self) -> i32 {
        self.order
    }

    /// 克隆底层拦截器共享指针。
    #[must_use]
    pub fn interceptor(&self) -> Arc<dyn Interceptor> {
        Arc::clone(&self.interceptor)
    }
}
