//! 默认 AOP 代理工厂。
//!
//! 对应 spring-aop `org.springframework.aop.framework.DefaultAopProxyFactory`。

use super::advised_support::AdvisedSupport;
use super::aop_proxy::{AopProxy, AopProxyError, AopProxyFactory};

/// 默认 AOP 代理工厂。
///
/// 对应 spring-aop `DefaultAopProxyFactory`。
///
/// 根据配置创建 JDK 动态代理或 CGLIB 代理。
///
/// # 选择逻辑
///
/// - 如果 `proxyTargetClass` 为 `true` 或没有接口，使用 CGLIB 代理
/// - 否则使用 JDK 动态代理
///
/// # 注意
///
/// 在 Rust 中，由于没有运行时代理，此工厂主要用于配置验证。
/// 实际代理创建由过程宏 `#[aspect]` 完成。
pub struct DefaultAopProxyFactory;

impl DefaultAopProxyFactory {
    /// 创建新的默认 AOP 代理工厂。
    pub fn new() -> Self {
        Self
    }

    /// 检查是否应该使用 CGLIB 代理。
    pub fn should_use_cglib_proxy(advised: &AdvisedSupport) -> bool {
        advised.config().is_proxy_target_class() || advised.get_interfaces().is_empty()
    }

    /// 验证代理配置。
    pub fn validate_config(advised: &AdvisedSupport) -> Result<(), AopProxyError> {
        if advised.get_advisors().is_empty() {
            return Err(AopProxyError::ConfigurationError(
                "At least one advisor is required".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for DefaultAopProxyFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl AopProxyFactory for DefaultAopProxyFactory {
    fn create_aop_proxy(
        &self,
        _target: Box<dyn std::any::Any>,
    ) -> Result<Box<dyn AopProxy>, AopProxyError> {
        // 在 Rust 中，代理创建由过程宏完成
        // 这里返回错误，表示需要使用 #[aspect] 宏
        Err(AopProxyError::CreationFailed(
            "Use #[aspect] macro instead of ProxyFactory in Rust".to_string(),
        ))
    }
}

impl std::fmt::Debug for DefaultAopProxyFactory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefaultAopProxyFactory").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_aop_proxy_factory_creation() {
        let factory = DefaultAopProxyFactory::new();
        let _ = factory;
    }

    #[test]
    fn should_use_cglib_proxy_empty_interfaces() {
        let advised = AdvisedSupport::new();
        assert!(DefaultAopProxyFactory::should_use_cglib_proxy(&advised));
    }

    #[test]
    fn validate_config_empty_advisors() {
        let advised = AdvisedSupport::new();
        let result = DefaultAopProxyFactory::validate_config(&advised);
        assert!(result.is_err());
    }
}
