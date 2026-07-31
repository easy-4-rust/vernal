//! AOP 上下文。
//!
//! 对应 spring-aop `org.springframework.aop.framework.AopContext`。
//! 用于在方法调用中暴露当前代理。

use std::cell::RefCell;

thread_local! {
    static CURRENT_PROXY: RefCell<Option<*const dyn std::any::Any>> = RefCell::new(None);
}

/// AOP 上下文。
///
/// 对应 spring-aop `AopContext`。
///
/// 用于在方法调用中暴露当前代理。当 `ProxyConfig.setExposeProxy(true)` 时，
/// 代理会暴露给当前线程，以便在方法内部访问。
///
/// # 安全性
///
/// 使用 ThreadLocal 存储当前代理。只在当前线程有效。
///
/// # 警告
///
/// 此类使用 `unsafe` 来存储原始指针。使用时必须确保：
/// 1. 代理在使用期间保持有效
/// 2. 不要在代理被释放后访问
pub struct AopContext;

impl AopContext {
    /// 设置当前代理。
    ///
    /// # 安全性
    ///
    /// 此方法使用 `unsafe` 来存储原始指针。调用者必须确保
    /// 代理在调用 `remove_current_proxy()` 之前保持有效。
    pub fn set_current_proxy(proxy: *const dyn std::any::Any) {
        CURRENT_PROXY.with(|current| {
            *current.borrow_mut() = Some(proxy);
        });
    }

    /// 获取当前代理。
    ///
    /// # 返回
    ///
    /// 如果当前线程有代理，返回 `Some(proxy)`，否则返回 `None`。
    pub fn current_proxy() -> Option<*const dyn std::any::Any> {
        CURRENT_PROXY.with(|current| *current.borrow())
    }

    /// 移除当前代理。
    pub fn remove_current_proxy() {
        CURRENT_PROXY.with(|current| {
            *current.borrow_mut() = None;
        });
    }

    /// 是否有当前代理。
    pub fn has_current_proxy() -> bool {
        CURRENT_PROXY.with(|current| current.borrow().is_some())
    }
}

impl Drop for AopContext {
    fn drop(&mut self) {
        // 不需要清理 ThreadLocal
    }
}

impl std::fmt::Debug for AopContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AopContext").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aop_context_initial_state() {
        assert!(!AopContext::has_current_proxy());
        assert!(AopContext::current_proxy().is_none());
    }

    #[test]
    fn aop_context_set_and_get() {
        let value = 42i32;
        let proxy = &value as *const i32 as *const dyn std::any::Any;

        AopContext::set_current_proxy(proxy);
        assert!(AopContext::has_current_proxy());
        assert!(AopContext::current_proxy().is_some());

        AopContext::remove_current_proxy();
        assert!(!AopContext::has_current_proxy());
        assert!(AopContext::current_proxy().is_none());
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn aop_context_initial_state() {
        assert!(!AopContext::has_current_proxy());
        assert!(AopContext::current_proxy().is_none());
    }

    #[test]
    fn aop_context_set_and_get() {
        let value = 42i32;
        let proxy = &value as *const i32 as *const dyn std::any::Any;

        AopContext::set_current_proxy(proxy);
        assert!(AopContext::has_current_proxy());
        assert!(AopContext::current_proxy().is_some());

        AopContext::remove_current_proxy();
        assert!(!AopContext::has_current_proxy());
        assert!(AopContext::current_proxy().is_none());
    }

    #[test]
    fn aop_context_debug() {
        let debug = format!("{:?}", AopContext);
        assert!(!debug.is_empty());
    }
}
