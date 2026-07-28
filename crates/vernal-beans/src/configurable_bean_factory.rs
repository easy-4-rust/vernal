//! ConfigurableBeanFactory — Spring 风格的可配置 BeanFactory 接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.ConfigurableBeanFactory`。
//!
//! 扩展 `HierarchicalBeanFactory`，提供配置容器的能力。

use std::sync::Arc;

use crate::bean_post_processor::BeanPostProcessor;
use crate::bean_scope::BeanScope;
use crate::hierarchical_bean_factory::HierarchicalBeanFactory;

/// 标准 singleton 作用域名称：`"singleton"`。
pub const SCOPE_SINGLETON: &str = "singleton";

/// 标准 prototype 作用域名称：`"prototype"`。
pub const SCOPE_PROTOTYPE: &str = "prototype";

/// Spring 风格的可配置 BeanFactory 接口。
///
/// 对应 Spring 的 `ConfigurableBeanFactory`。
///
/// 提供配置容器的能力，包括：
/// - 设置父容器
/// - 注册/获取 Scope
/// - 管理 BeanPostProcessor
/// - 注册/解析别名
/// - 销毁 Bean
pub trait ConfigurableBeanFactory: HierarchicalBeanFactory {
    /// 设置父 BeanFactory。
    ///
    /// 对应 Spring 的 `void setParentBeanFactory(BeanFactory parentBeanFactory)`。
    fn set_parent_bean_factory(
        &mut self,
        parent: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 注册自定义 Scope。
    ///
    /// 对应 Spring 的 `void registerScope(String scopeName, Scope scope)`。
    fn register_scope(&mut self, scope_name: &str, scope: Box<dyn BeanScope>);

    /// 获取已注册的 Scope 名称。
    ///
    /// 对应 Spring 的 `String[] getRegisteredScopeNames()`。
    fn registered_scope_names(&self) -> Vec<String>;

    /// 按名称获取已注册的 Scope。
    ///
    /// 对应 Spring 的 `Scope getRegisteredScope(String scopeName)`。
    fn get_registered_scope(&self, scope_name: &str) -> Option<&dyn BeanScope>;

    /// 添加 BeanPostProcessor。
    ///
    /// 对应 Spring 的 `void addBeanPostProcessor(BeanPostProcessor beanPostProcessor)`。
    fn add_bean_post_processor(&mut self, processor: Arc<dyn BeanPostProcessor>);

    /// 获取 BeanPostProcessor 数量。
    ///
    /// 对应 Spring 的 `int getBeanPostProcessorCount()`。
    fn bean_post_processor_count(&self) -> usize;

    /// 注册别名。
    ///
    /// 对应 Spring 的 `void registerAlias(String beanName, String alias)`。
    fn register_alias(
        &mut self,
        bean_name: &str,
        alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 检查是否是 FactoryBean。
    ///
    /// 对应 Spring 的 `boolean isFactoryBean(String name)`。
    fn is_factory_bean(&self, name: &str) -> bool;

    /// 设置 Bean 当前是否在创建中。
    ///
    /// 对应 Spring 的 `void setCurrentlyInCreation(String beanName, boolean inCreation)`。
    fn set_currently_in_creation(&mut self, bean_name: &str, in_creation: bool);

    /// 检查 Bean 是否当前在创建中。
    ///
    /// 对应 Spring 的 `boolean isCurrentlyInCreation(String beanName)`。
    fn is_currently_in_creation(&self, bean_name: &str) -> bool;

    /// 注册依赖关系。
    ///
    /// 对应 Spring 的 `void registerDependentBean(String beanName, String dependentBeanName)`。
    fn register_dependent_bean(&mut self, bean_name: &str, dependent_bean_name: &str);

    /// 获取依赖指定 Bean 的所有 Bean。
    ///
    /// 对应 Spring 的 `String[] getDependentBeans(String beanName)`。
    fn get_dependent_beans(&self, bean_name: &str) -> Vec<String>;

    /// 获取指定 Bean 的所有依赖。
    ///
    /// 对应 Spring 的 `String[] getDependenciesForBean(String beanName)`。
    fn get_dependencies_for_bean(&self, bean_name: &str) -> Vec<String>;

    /// 销毁指定 Bean。
    ///
    /// 对应 Spring 的 `void destroyBean(String beanName, Object beanInstance)`。
    fn destroy_bean(
        &self,
        bean_name: &str,
        bean_instance: &dyn std::any::Any,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 销毁所有 singleton Bean。
    ///
    /// 对应 Spring 的 `void destroySingletons()`。
    fn destroy_singletons(&self);

    /// 添加嵌入式值解析器（用于 `@Value` 注解）。
    ///
    /// 对应 Spring 的 `void addEmbeddedValueResolver(StringValueResolver valueResolver)`。
    fn add_embedded_value_resolver(&mut self, resolver: Arc<dyn Fn(&str) -> String + Send + Sync>);

    /// 解析嵌入式值（`${...}` 占位符）。
    ///
    /// 对应 Spring 的 `String resolveEmbeddedValue(String value)`。
    fn resolve_embedded_value(&self, value: &str) -> String;
}
