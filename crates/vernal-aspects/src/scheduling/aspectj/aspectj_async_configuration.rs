//! 对标 `org.springframework.scheduling.aspectj.AspectJAsyncConfiguration`。
//!
//! `@Configuration` 类：注册 AspectJ 异步执行切面 Bean。

use std::sync::{Arc, RwLock};

use super::annotation_async_execution_aspect::AnnotationAsyncExecutionAspect;

/// AspectJ 异步执行配置。
///
/// 对标 Spring 的 `AspectJAsyncConfiguration`。
/// 注册 `AnnotationAsyncExecutionAspect` 单例 Bean。
pub struct AspectJAsyncConfiguration {
    /// 已注册的异步切面（懒初始化）。
    aspect: RwLock<Option<Arc<AnnotationAsyncExecutionAspect>>>,
}

impl AspectJAsyncConfiguration {
    /// 创建配置实例。
    pub fn new() -> Self {
        Self {
            aspect: RwLock::new(None),
        }
    }

    /// 注册异步切面 Bean。
    ///
    /// 对应 Spring 的 `@Bean(name = TaskManagementConfigUtils.ASYNC_EXECUTION_ASPECT_BEAN_NAME)`。
    pub fn register(&self) -> Arc<AnnotationAsyncExecutionAspect> {
        let mut guard = self.aspect.write().unwrap();
        if guard.is_none() {
            *guard = Some(Arc::new(AnnotationAsyncExecutionAspect::new()));
        }
        guard.as_ref().unwrap().clone()
    }

    /// 获取已注册的切面。
    pub fn get_aspect(&self) -> Option<Arc<AnnotationAsyncExecutionAspect>> {
        self.aspect.read().unwrap().clone()
    }
}

impl Default for AspectJAsyncConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_creation() {
        let config = AspectJAsyncConfiguration::new();
        assert!(config.get_aspect().is_none());
    }

    #[test]
    fn test_configuration_register() {
        let config = AspectJAsyncConfiguration::new();
        let aspect = config.register();
        assert!(config.get_aspect().is_some());
        // 两次注册返回同一个实例
        let aspect2 = config.register();
        assert!(Arc::ptr_eq(&aspect, &aspect2));
    }

    #[test]
    fn test_configuration_default() {
        let config = AspectJAsyncConfiguration::default();
        assert!(config.get_aspect().is_none());
    }
}
