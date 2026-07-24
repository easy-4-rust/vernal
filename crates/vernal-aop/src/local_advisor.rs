//! 本地切面顾问对象。

use std::sync::Arc;

use crate::{LocalInterceptor, Operation, Pointcut};

/// 将切点、`!Send` 环绕拦截器和顺序组合成本地切面声明。
///
/// 切点和排序规则与 [`crate::Advisor`] 完全一致；只在拦截器调用合同上选择
/// [`LocalInterceptor`]，避免为框架路由重新发明 Operation 匹配语义。
#[derive(Clone)]
pub struct LocalAdvisor {
    pointcut: Arc<dyn Pointcut>,
    interceptor: Arc<dyn LocalInterceptor>,
    order: i32,
}

impl LocalAdvisor {
    /// 使用具体切点与本地拦截器创建顾问。
    #[must_use]
    pub fn new<P, I>(pointcut: P, interceptor: I, order: i32) -> Self
    where
        P: Pointcut,
        I: LocalInterceptor,
    {
        Self {
            pointcut: Arc::new(pointcut),
            interceptor: Arc::new(interceptor),
            order,
        }
    }

    /// 使用已共享的切点与本地拦截器创建顾问。
    #[must_use]
    pub fn shared(
        pointcut: Arc<dyn Pointcut>,
        interceptor: Arc<dyn LocalInterceptor>,
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

    /// 返回执行顺序；数值越小越先进入、越后退出。
    #[must_use]
    pub const fn order(&self) -> i32 {
        self.order
    }

    /// 克隆底层本地拦截器共享指针。
    #[must_use]
    pub fn interceptor(&self) -> Arc<dyn LocalInterceptor> {
        Arc::clone(&self.interceptor)
    }
}
