//! BeanFactoryPostProcessor — Spring 风格的 Bean 工厂后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanFactoryPostProcessor`。
//!
//! 在 Bean 实例化之前，允许修改 Bean 定义。

use crate::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

/// Spring 风格的 Bean 工厂后处理器接口。
///
/// 对应 Spring 的 `BeanFactoryPostProcessor`。
///
/// 在 Bean 实例化之前，允许修改 Bean 定义。
/// 这是 Spring 配置系统的核心扩展点之一。
pub trait BeanFactoryPostProcessor: Send + Sync + 'static {
    /// 在 Bean 工厂初始化后、Bean 实例化前调用。
    ///
    /// 可以修改 Bean 定义、注册新的 Bean 定义等。
    fn post_process_bean_factory(
        &self,
        bean_factory: &mut dyn ConfigurableListableBeanFactory,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
