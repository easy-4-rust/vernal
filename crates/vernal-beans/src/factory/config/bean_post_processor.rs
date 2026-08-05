//! BeanPostProcessor — Spring 风格的 Bean 后处理器接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanPostProcessor`。
//!
//! 容器在 Bean 实例化后、属性注入完成后，对每个 Bean 调用 `BeanPostProcessor` 链。
//! 用户通过注册 `BeanPostProcessor` 实现来拦截 Bean 的初始化过程。

use std::any::Any;
use std::sync::Arc;

/// Spring 风格的 Bean 后处理器接口。
///
/// 对应 Spring 的 `BeanPostProcessor`。
///
/// 容器在每个 Bean 的 `afterPropertiesSet` 前后各调用一次：
/// - `post_process_before_initialization` — 在 `afterPropertiesSet` 之前
/// - `post_process_after_initialization` — 在 `afterPropertiesSet` 之后
///
/// ## 默认行为
///
/// 两个方法默认返回传入的 `bean`（不做任何修改）。
/// 这与 Spring 的 `BeanPostProcessor` 默认行为一致。
///
/// ## 使用场景
///
/// - AOP 代理包装
/// - 自定义注解处理（`@PostConstruct`）
/// - Bean 属性验证
/// - Bean 包装（如返回代理对象）
///
/// ## 与 vernal-aop 的关系
///
/// `vernal-aop` 的 `InterceptorChain` 可以通过实现 `BeanPostProcessor` 注入容器。
/// 这样每个 Bean 在初始化前后都会经过 AOP 拦截链。
pub trait BeanPostProcessor: Send + Sync + 'static {
    /// 在 Bean 的 `afterPropertiesSet` 之前调用。
    ///
    /// 对应 Spring 的 `BeanPostProcessor.postProcessBeforeInitialization(Object bean, String beanName)`。
    ///
    /// # 参数
    ///
    /// - `bean` — 已实例化但未完成初始化的 Bean（类型擦除）
    /// - `bean_name` — Bean 在容器中的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Some(new_bean))` — 返回替代 Bean（例如 AOP 代理）
    /// - `Ok(None)` — 返回原始 Bean
    /// - `Err` — 阻止 Bean 创建，抛出异常
    fn post_process_before_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }

    /// 在 Bean 的 `afterPropertiesSet` 之后调用。
    ///
    /// 对应 Spring 的 `BeanPostProcessor.postProcessAfterInitialization(Object bean, String beanName)`。
    ///
    /// # 参数
    ///
    /// - `bean` — 已完成初始化的 Bean（类型擦除）
    /// - `_bean_name` — Bean 在容器中的名称
    ///
    /// # 返回
    ///
    /// - `Ok(Some(new_bean))` — 返回替代 Bean（例如 AOP 代理）
    /// - `Ok(None)` — 返回原始 Bean
    /// - `Err` — 阻止 Bean 创建，抛出异常
    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some(bean))
    }

    /// 在 Bean 实例化之前调用。
    ///
    /// 对应 Spring 的 `InstantiationAwareBeanPostProcessor.postProcessBeforeInstantiation`。
    ///
    /// 返回 `Some(proxy)` 时跳过正常实例化流程。
    /// 默认返回 `None`（不跳过实例化）。
    fn post_process_before_instantiation(
        &self,
        _bean_class: &str,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    /// 在 Bean 实例化之后、属性注入之前调用。
    ///
    /// 对应 Spring 的 `InstantiationAwareBeanPostProcessor.postProcessAfterInstantiation`。
    ///
    /// 返回 `false` 时跳过属性注入。
    /// 默认返回 `true`（继续属性注入）。
    fn post_process_after_instantiation(
        &self,
        _bean: &dyn Any,
        _bean_name: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }

    /// 在 Bean 销毁之前调用。
    ///
    /// 对应 Spring 的 `DestructionAwareBeanPostProcessor.postProcessBeforeDestruction`。
    ///
    /// 默认不做任何操作。
    fn post_process_before_destruction(
        &self,
        _bean: &dyn Any,
        _bean_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// 检查是否需要销毁此 Bean。
    ///
    /// 对应 Spring 的 `DestructionAwareBeanPostProcessor.requiresDestruction`。
    ///
    /// 默认返回 `true`。
    fn requires_destruction(&self, _bean: &dyn Any) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test BeanPostProcessor that wraps beans in a marker string.
    struct WrappingProcessor;

    impl BeanPostProcessor for WrappingProcessor {
        fn post_process_before_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }

        fn post_process_after_initialization(
            &self,
            bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(bean))
        }
    }

    /// A processor that returns None (no replacement).
    struct NoOpProcessor;

    impl BeanPostProcessor for NoOpProcessor {}

    /// A processor that errors on before_initialization.
    struct ErrorProcessor;

    impl BeanPostProcessor for ErrorProcessor {
        fn post_process_before_initialization(
            &self,
            _bean: Arc<dyn Any + Send + Sync>,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Err("intentional error".into())
        }
    }

    /// A processor that returns a proxy on before_instantiation.
    struct ProxyProcessor;

    impl BeanPostProcessor for ProxyProcessor {
        fn post_process_before_instantiation(
            &self,
            _bean_class: &str,
            _bean_name: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(Some(Arc::new("proxy_bean".to_string())))
        }
    }

    #[test]
    fn default_before_initialization_returns_bean() {
        let processor = NoOpProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor
            .post_process_before_initialization(bean.clone(), "myBean")
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn default_after_initialization_returns_bean() {
        let processor = NoOpProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor
            .post_process_after_initialization(bean.clone(), "myBean")
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn default_before_instantiation_returns_none() {
        let processor = NoOpProcessor;
        let result = processor
            .post_process_before_instantiation("MyClass", "myBean")
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn default_after_instantiation_returns_true() {
        let processor = NoOpProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor
            .post_process_after_instantiation(bean.as_ref(), "myBean")
            .unwrap();
        assert!(result);
    }

    #[test]
    fn default_before_destruction_returns_ok() {
        let processor = NoOpProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor.post_process_before_destruction(bean.as_ref(), "myBean");
        assert!(result.is_ok());
    }

    #[test]
    fn default_requires_destruction_returns_true() {
        let processor = NoOpProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        assert!(processor.requires_destruction(bean.as_ref()));
    }

    #[test]
    fn wrapping_processor_before() {
        let processor = WrappingProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor
            .post_process_before_initialization(bean, "myBean")
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn wrapping_processor_after() {
        let processor = WrappingProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor
            .post_process_after_initialization(bean, "myBean")
            .unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn error_processor_before_initialization() {
        let processor = ErrorProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor.post_process_before_initialization(bean, "myBean");
        assert!(result.is_err());
    }

    #[test]
    fn proxy_processor_returns_proxy() {
        let processor = ProxyProcessor;
        let result = processor
            .post_process_before_instantiation("MyClass", "myBean")
            .unwrap();
        assert!(result.is_some());
        let proxy = result.unwrap();
        let name = proxy.downcast_ref::<String>().unwrap();
        assert_eq!(name, "proxy_bean");
    }

    #[test]
    fn proxy_processor_default_after_instantiation() {
        let processor = ProxyProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        let result = processor
            .post_process_after_instantiation(bean.as_ref(), "myBean")
            .unwrap();
        assert!(result);
    }

    #[test]
    fn proxy_processor_default_before_destruction() {
        let processor = ProxyProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        assert!(
            processor
                .post_process_before_destruction(bean.as_ref(), "myBean")
                .is_ok()
        );
    }

    #[test]
    fn proxy_processor_requires_destruction() {
        let processor = ProxyProcessor;
        let bean: Arc<dyn Any + Send + Sync> = Arc::new("test".to_string());
        assert!(processor.requires_destruction(bean.as_ref()));
    }
}
