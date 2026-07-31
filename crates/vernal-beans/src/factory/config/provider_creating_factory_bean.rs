//! ProviderCreatingFactoryBean — 对应 Spring `org.springframework.beans.factory.config.ProviderCreatingFactoryBean`。
//!
//! Provider 创建工厂 Bean。

use std::sync::Arc;

/// Provider 创建工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.ProviderCreatingFactoryBean`。
///
/// 用于创建 `Provider<T>` 实例，支持延迟获取 Bean。
/// Provider 是一个可以延迟获取 Bean 的工厂。
///
/// ## 使用场景
///
/// - 延迟注入
/// - 循环依赖解决
/// - 按需获取 Bean
#[derive(Debug)]
pub struct ProviderCreatingFactoryBean {
    /// 目标 Bean 名称。
    target_bean_name: String,
}

impl ProviderCreatingFactoryBean {
    /// 创建新的 ProviderCreatingFactoryBean。
    pub fn new(target_bean_name: impl Into<String>) -> Self {
        Self {
            target_bean_name: target_bean_name.into(),
        }
    }

    /// 获取目标 Bean 名称。
    pub fn target_bean_name(&self) -> &str {
        &self.target_bean_name
    }
}

/// Provider 实现，用于延迟获取 Bean。
#[derive(Debug, Clone)]
pub struct BeanProvider<T: Clone + Send + Sync + 'static> {
    /// Bean 名称。
    bean_name: String,
    /// Bean 实例（延迟初始化）。
    instance: Option<Arc<T>>,
}

impl<T: Clone + Send + Sync + 'static> BeanProvider<T> {
    /// 创建新的 BeanProvider。
    pub fn new(bean_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            instance: None,
        }
    }

    /// 获取 Bean 名称。
    pub fn bean_name(&self) -> &str {
        &self.bean_name
    }

    /// 设置 Bean 实例。
    pub fn set_instance(&mut self, instance: Arc<T>) {
        self.instance = Some(instance);
    }

    /// 获取 Bean 实例。
    pub fn get(&self) -> Option<&Arc<T>> {
        self.instance.as_ref()
    }

    /// 是否已初始化。
    pub fn is_initialized(&self) -> bool {
        self.instance.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creating_factory_bean_new() {
        let factory = ProviderCreatingFactoryBean::new("myBean");
        assert_eq!(factory.target_bean_name(), "myBean");
    }

    #[test]
    fn test_bean_provider_new() {
        let provider = BeanProvider::<String>::new("myBean");
        assert_eq!(provider.bean_name(), "myBean");
        assert!(!provider.is_initialized());
        assert!(provider.get().is_none());
    }

    #[test]
    fn test_bean_provider_with_instance() {
        let mut provider = BeanProvider::<String>::new("myBean");
        provider.set_instance(Arc::new(String::from("hello")));

        assert!(provider.is_initialized());
        assert_eq!(provider.get().unwrap().as_ref(), "hello");
    }
}
