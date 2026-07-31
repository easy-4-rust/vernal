//! 原始目标访问。
//!
//! 对应 spring-aop `RawTargetAccess`。
//! 标记接口，指示代理应该直接访问目标。

/// 原始目标访问标记。
///
/// 对应 spring-aop `RawTargetAccess`。
///
/// 实现此接口的代理应该绕过 AOP 拦截，直接访问目标。
pub trait RawTargetAccess: Send + Sync + 'static {
    /// 是否允许原始访问。
    fn allows_raw_access(&self) -> bool {
        true
    }
}

/// 可刷新接口。
///
/// 对应 spring-aop `Refreshable`。
pub trait Refreshable: Send + Sync + 'static {
    /// 刷新资源。
    fn refresh(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 池化配置。
///
/// 对应 spring-aop `PoolingConfig`。
pub trait PoolingConfig: Send + Sync + 'static {
    /// 获取池大小。
    fn get_pool_size(&self) -> usize;

    /// 获取活跃对象数。
    fn get_active_count(&self) -> usize;

    /// 获取空闲对象数。
    fn get_idle_count(&self) -> usize;

    /// 是否为可池化。
    fn is_poolable(&self) -> bool {
        true
    }
}

/// 作用域对象。
///
/// 对应 spring-aop `ScopedObject`。
pub trait ScopedObject: Send + Sync + 'static {
    /// 获取作用域内的目标对象。
    fn get_target_object(&self) -> Result<Box<dyn std::any::Any>, Box<dyn std::error::Error + Send + Sync>>;

    /// 释放目标对象。
    fn release_target_object(&self, target: Box<dyn std::any::Any>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Spring 代理标记。
///
/// 对应 spring-aop `SpringProxy`。
pub trait SpringProxy: Send + Sync + 'static {
    /// 获取代理类型名。
    fn get_proxy_type(&self) -> &str;
}

/// 线程本地目标源统计。
///
/// 对应 spring-aop `ThreadLocalTargetSourceStats`。
pub trait ThreadLocalTargetSourceStats: Send + Sync + 'static {
    /// 获取命中数。
    fn get_hit_count(&self) -> u64;

    /// 获取未命中数。
    fn get_miss_count(&self) -> u64;

    /// 重置统计。
    fn reset_stats(&self);
}

/// 异步未捕获异常处理器。
///
/// 对应 spring-aop `AsyncUncaughtExceptionHandler`。
pub trait AsyncUncaughtExceptionHandler: Send + Sync + 'static {
    /// 处理未捕获异常。
    fn handle_uncaught_exception(&self, ex: Box<dyn std::error::Error + Send + Sync>, method: &str, params: &[&dyn std::any::Any]);
}

/// AOP 基础设施 Bean 标记。
///
/// 对应 spring-aop `AopInfrastructureBean`。
pub trait AopInfrastructureBean: Send + Sync + 'static {
    /// 是否为 AOP 基础设施。
    fn is_aop_infrastructure(&self) -> bool {
        true
    }
}

/// 顾问支持监听器。
///
/// 对应 spring-aop `AdvisedSupportListener`。
pub trait AdvisedSupportListener: Send + Sync + 'static {
    /// 顾问激活时调用。
    fn advice_activated(&self, advice: &dyn std::any::Any);

    /// 顾问停用时调用。
    fn advice_deactivated(&self, advice: &dyn std::any::Any);
}

/// 实例化模型感知切点顾问。
///
/// 对应 spring-aop `InstantiationModelAwarePointcutAdvisor`。
pub trait InstantiationModelAwarePointcutAdvisor: Send + Sync + 'static {
    /// 是否为懒加载。
    fn is_lazy(&self) -> bool {
        false
    }

    /// 是否为每次实例化。
    fn is_per_instance(&self) -> bool {
        false
    }
}

/// 元数据感知切点顾问。
///
/// 对应 spring-aop `InstantiationModelAwarePointcutAdvisor`。
pub trait MetadataAwarePointcutAdvisor: Send + Sync + 'static {
    /// 获取切面类名。
    fn get_aspect_name(&self) -> &str;

    /// 获取声明顺序。
    fn get_declaration_order(&self) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPoolingConfig {
        pool_size: usize,
        active: usize,
    }

    impl PoolingConfig for TestPoolingConfig {
        fn get_pool_size(&self) -> usize {
            self.pool_size
        }

        fn get_active_count(&self) -> usize {
            self.active
        }

        fn get_idle_count(&self) -> usize {
            self.pool_size - self.active
        }
    }

    #[test]
    fn pooling_config() {
        let config = TestPoolingConfig {
            pool_size: 10,
            active: 3,
        };
        assert_eq!(config.get_pool_size(), 10);
        assert_eq!(config.get_active_count(), 3);
        assert_eq!(config.get_idle_count(), 7);
        assert!(config.is_poolable());
    }

    struct TestSpringProxy;

    impl SpringProxy for TestSpringProxy {
        fn get_proxy_type(&self) -> &str {
            "TestProxy"
        }
    }

    #[test]
    fn spring_proxy() {
        let proxy = TestSpringProxy;
        assert_eq!(proxy.get_proxy_type(), "TestProxy");
    }
}
