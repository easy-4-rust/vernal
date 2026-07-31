//! ConfigurableListableBeanFactory — Spring 风格的可配置可列举 BeanFactory 接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.ConfigurableListableBeanFactory`。
//!
//! 扩展 `ListableBeanFactory` + `ConfigurableBeanFactory`，
//! 是 Spring 容器的最高级接口。

use std::sync::Arc;

use crate::factory::config::configurable_bean_factory::ConfigurableBeanFactory;
use crate::factory::listable_bean_factory::ListableBeanFactory;

/// Spring 风格的可配置可列举 BeanFactory 接口。
///
/// 对应 Spring 的 `ConfigurableListableBeanFactory`。
///
/// 继承 `ListableBeanFactory` + `ConfigurableBeanFactory`，
/// 是 Spring 容器功能最完整的接口。`Container` 应该实现此接口。
///
/// ## 额外能力
///
/// - 忽略特定类型的自动装配
/// - 注册可解析依赖
/// - 冻结配置
/// - 预实例化所有 singleton
pub trait ConfigurableListableBeanFactory: ListableBeanFactory + ConfigurableBeanFactory {
    /// 忽略特定类型的自动装配。
    fn ignore_dependency_type(&mut self, type_id: std::any::TypeId);

    /// 忽略特定接口的自动装配。
    fn ignore_dependency_interface(&mut self, interface_id: std::any::TypeId);

    /// 注册可解析依赖（`@Autowired` 的隐式依赖）。
    fn register_resolvable_dependency(
        &mut self,
        dependency_type: std::any::TypeId,
        autowired_value: Arc<dyn std::any::Any + Send + Sync>,
    );

    /// 检查是否是 autowire candidate。
    fn is_autowire_candidate(&self, bean_name: &str) -> bool;

    /// 冻结配置。
    fn freeze_configuration(&mut self);

    /// 检查配置是否已冻结。
    fn is_configuration_frozen(&self) -> bool;

    /// 预实例化所有 singleton Bean。
    fn pre_instantiate_singletons(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
