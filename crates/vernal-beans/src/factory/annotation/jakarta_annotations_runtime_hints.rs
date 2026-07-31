//! JakartaAnnotationsRuntimeHints — Jakarta 注解运行时提示。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.JakartaAnnotationsRuntimeHints`。
//!
//! 在 Spring Native / GraalVM AOT 场景下，此类负责注册 Jakarta 注解
//! （`@PostConstruct`、`@PreDestroy`、`@Resource` 等）的运行时反射提示，
//! 确保这些注解在原生镜像中可用。
//! 在 vernal 中，此结构体跟踪提示注册次数以支持调试和可观测性。

use std::sync::atomic::{AtomicU32, Ordering};

/// Jakarta 注解运行时提示。
///
/// 对应 Spring 的 `JakartaAnnotationsRuntimeHints`。
///
/// 提供 Jakarta 注解的运行时提示信息，用于 AOT / GraalVM 原生镜像场景。
pub struct JakartaAnnotationsRuntimeHints {
    registered: AtomicU32,
}

impl JakartaAnnotationsRuntimeHints {
    /// 创建新的运行时提示。
    pub fn new() -> Self { Self { registered: AtomicU32::new(0) } }

    /// 注册一个提示（原子递增计数）。
    pub fn register(&self) { self.registered.fetch_add(1, Ordering::SeqCst); }

    /// 已注册的提示数量。
    pub fn registered_count(&self) -> u32 { self.registered.load(Ordering::SeqCst) }

    /// 重置计数器。
    pub fn reset(&self) { self.registered.store(0, Ordering::SeqCst); }

    /// 批量注册指定数量的提示。
    pub fn register_n(&self, count: u32) {
        self.registered.fetch_add(count, Ordering::SeqCst);
    }
}

impl Default for JakartaAnnotationsRuntimeHints { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_increments_count() {
        let hints = JakartaAnnotationsRuntimeHints::new();
        assert_eq!(hints.registered_count(), 0);
        hints.register();
        hints.register();
        assert_eq!(hints.registered_count(), 2);
    }

    #[test]
    fn reset_sets_count_to_zero() {
        let hints = JakartaAnnotationsRuntimeHints::new();
        hints.register();
        hints.register();
        hints.reset();
        assert_eq!(hints.registered_count(), 0);
    }

    #[test]
    fn register_n_batch() {
        let hints = JakartaAnnotationsRuntimeHints::new();
        hints.register_n(5);
        assert_eq!(hints.registered_count(), 5);
    }
}
