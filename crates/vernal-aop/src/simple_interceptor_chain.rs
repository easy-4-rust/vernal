//! 简化拦截器链。
//!
//! 对标 tx_di 的 `InterceptorChain`：管理一组有序的 `SimpleInterceptor`，
//! 提供 `before_all` / `after_all` / `around_all` 执行方法。

use std::sync::Arc;

use vernal_core::BoxError;

use super::{SimpleCallResult, SimpleInterceptor, SimpleInvocationContext};

/// 简化拦截器链。
///
/// 管理一组按注册顺序排列的 `SimpleInterceptor`，提供批量执行方法。
/// 拦截器按注册顺序执行 `before`，按逆序执行 `after`（洋葱模型）。
///
/// # 设计来源
///
/// 对标 tx_di 的 `InterceptorChain`，使用 `OnceLock<Arc<SimpleInterceptorChain>>`
/// 存储（与 tx_di 的 per-component 静态模式一致）。
pub struct SimpleInterceptorChain {
    /// 按注册顺序排列的拦截器列表
    interceptors: Vec<Arc<dyn SimpleInterceptor>>,
}

impl SimpleInterceptorChain {
    /// 创建空的拦截器链。
    #[must_use]
    pub fn new() -> Self {
        Self {
            interceptors: Vec::new(),
        }
    }

    /// 创建包含指定拦截器的链。
    #[must_use]
    pub fn with_interceptors(interceptors: Vec<Arc<dyn SimpleInterceptor>>) -> Self {
        Self { interceptors }
    }

    /// 添加一个拦截器到链尾。
    pub fn push(&mut self, interceptor: Arc<dyn SimpleInterceptor>) {
        self.interceptors.push(interceptor);
    }

    /// 返回链中拦截器数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.interceptors.len()
    }

    /// 链是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.interceptors.is_empty()
    }

    /// 按注册顺序执行所有拦截器的 `before` 钩子。
    ///
    /// 遇到第一个 `Err` 立即返回，不执行后续拦截器。
    pub fn before_all(&self, context: &SimpleInvocationContext) -> Result<(), BoxError> {
        for interceptor in &self.interceptors {
            interceptor.before(context)?;
        }
        Ok(())
    }

    /// 按逆序执行所有拦截器的 `after` 钩子。
    ///
    /// `after` 钩子不返回错误，保证所有拦截器都能执行清理逻辑。
    pub fn after_all(&self, context: &SimpleInvocationContext, result: &SimpleCallResult) {
        for interceptor in self.interceptors.iter().rev() {
            interceptor.after(context, result);
        }
    }

    /// 构建洋葱模型的环绕执行链。
    ///
    /// 最外层拦截器最先进入 `around`，在其中调用内层，形成嵌套结构。
    /// 与 tx_di 的 `around_all` 语义一致。
    pub fn around_all<F>(
        &self,
        context: &SimpleInvocationContext,
        body: F,
    ) -> SimpleCallResult
    where
        F: FnOnce() -> SimpleCallResult + Send + 'static,
    {
        if self.interceptors.is_empty() {
            return body();
        }
        // 从最内层开始，逐层向外包裹
        let mut chain: Box<dyn FnOnce() -> SimpleCallResult + Send> = Box::new(body);
        for interceptor in self.interceptors.iter().rev() {
            let interceptor = Arc::clone(interceptor);
            let ctx = context.clone();
            let inner = chain;
            chain = Box::new(move || interceptor.around(&ctx, inner));
        }
        chain()
    }
}

impl Default for SimpleInterceptorChain {
    fn default() -> Self {
        Self::new()
    }
}
