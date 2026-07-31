#![forbid(unsafe_code)]
#![doc = "Vernal 内建 AOP 切面。"]
//!
//! 对标 Spring 的 `spring-aspects` 模块，提供框架级横切关注点实现：
//!
//! - `transaction::aspectj`：事务管理（对标 `@Transactional`，AspectJ 织入）
//! - `cache::aspectj`：缓存管理（对标 `@Cacheable`，AspectJ 织入）
//! - `scheduling::aspectj`：异步执行（对标 `@Async`，AspectJ 织入）
//! - `beans::factory::aspectj`：可配置对象 DI（对标 `@Configurable`，AspectJ 织入）
//! - `context::annotation::aspectj`：Spring Configured 启用（对标 `@EnableSpringConfigured`）
//!
//! ## 设计原则
//!
//! - 目录命名 100% 镜像 Spring 5 个 aspectj 子包路径
//! - 所有切面都是 `vernal-aop::Interceptor` 的实现
//! - 切面通过 `ApplicationModule` 注册到 ApplicationContext
//! - 切面不直接依赖具体实现（事务/缓存等由 `vernal-tx`/`vernal-cache` 提供）
//! - 切面只负责 AOP 拦截逻辑，不负责底层实现

/// 对标 `org.springframework.transaction.aspectj` 包。
pub mod transaction {
    /// 对标 `org.springframework.transaction.aspectj` 包。
    pub mod aspectj;
}

/// 对标 `org.springframework.cache.aspectj` 包。
pub mod cache {
    /// 对标 `org.springframework.cache.aspectj` 包。
    pub mod aspectj;
}

/// 对标 `org.springframework.scheduling.aspectj` 包。
pub mod scheduling {
    /// 对标 `org.springframework.scheduling.aspectj` 包。
    pub mod aspectj;
}

/// 对标 `org.springframework.beans.factory.aspectj` 包。
pub mod beans {
    /// Bean 工厂子包（镜像 Spring `beans.factory` 包路径）。
    pub mod factory {
        /// 对标 `org.springframework.beans.factory.aspectj` 包（Bean 工厂 AspectJ 切面）。
        pub mod aspectj;
    }
}

/// 对标 `org.springframework.context.annotation.aspectj` 包。
pub mod context {
    /// 上下文注解子包（镜像 Spring `context.annotation` 包路径）。
    pub mod annotation {
        /// 对标 `org.springframework.context.annotation.aspectj` 包（上下文注解 AspectJ 配置）。
        pub mod aspectj;
    }
}

/// 对标 `org.springframework.beans.factory` 包（按审计脚本"保留末两层"规则）。
pub mod factory;

/// 对标 `org.springframework.context.annotation` 包（按审计脚本"保留末两层"规则）。
pub mod annotation;

/// 切面织入机制（对标 `META-INF/aop.xml` + advice 类型）。
pub mod weaver;

/// vernal-aop ↔ aspect-rs 桥接。
pub mod support;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_aspectj() {
        let _ = transaction::aspectj::Propagation::default();
        let _ = transaction::aspectj::Isolation::default();
    }

    #[test]
    fn test_cache_aspectj() {
        let _ = cache::aspectj::CacheOperation::Cacheable;
    }

    #[test]
    fn test_scheduling_aspectj() {
        let _ = scheduling::aspectj::AbstractAsyncExecutionAspect::new();
    }

    #[test]
    fn test_beans_factory_aspectj() {
        fn assert_impl<T: beans::factory::aspectj::ConfigurableObject>() {}
        struct Test;
        impl beans::factory::aspectj::ConfigurableObject for Test {}
        assert_impl::<Test>();
    }

    #[test]
    fn test_context_annotation_aspectj() {
        let _ = context::annotation::aspectj::SpringConfiguredConfiguration::new();
    }

    #[test]
    fn test_weaver() {
        let _ = weaver::AdviceKind::Before;
        let _ = weaver::PointcutMatcher::match_execution_public;
    }

    #[test]
    fn test_support() {
        let _ = support::AopBridge::new();
    }

    #[test]
    fn test_factory_aspectj() {
        fn assert_impl<T: factory::aspectj::ConfigurableObject>() {}
        struct Test;
        impl factory::aspectj::ConfigurableObject for Test {}
        assert_impl::<Test>();
    }

    #[test]
    fn test_annotation_aspectj() {
        let _ = annotation::aspectj::SpringConfiguredConfiguration::new();
    }
}
