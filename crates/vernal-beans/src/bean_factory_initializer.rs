//! BeanFactoryInitializer — Spring 风格的 BeanFactory 初始化器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.BeanFactoryInitializer`。
//!
//! 用于在 BeanFactory 配置完成后、Bean 实例化前对其进行自定义初始化。

use crate::configurable_listable_bean_factory::ConfigurableListableBeanFactory;

/// Spring 风格的 BeanFactory 初始化器 trait。
///
/// 对应 Spring 的 `BeanFactoryInitializer`（Spring 7.x 引入）。
///
/// 允许在容器加载所有 Bean 定义之后、实例化 singleton Bean 之前，
/// 对 `ConfigurableListableBeanFactory` 进行额外的配置和定制。
///
/// 与 `BeanFactoryPostProcessor` 的区别：
/// - `BeanFactoryPostProcessor` — 在 Bean 定义加载后、Bean 实例化前执行，
///   通常用于修改 Bean 定义
/// - `BeanFactoryInitializer` — 在 BeanFactory 配置完成后执行，
///   用于对工厂本身的额外设置
///
/// ## 示例
///
/// ```rust,ignore
/// use vernal_beans::bean_factory_initializer::BeanFactoryInitializer;
///
/// struct MyInitializer;
/// impl BeanFactoryInitializer for MyInitializer {
///     fn initialize(&self, factory: &mut dyn ConfigurableListableBeanFactory) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///         // 注册自定义依赖
///         factory.register_resolvable_dependency(
///             std::any::TypeId::of::<MyDependency>(),
///             Arc::new(MyDependency::default()),
///         );
///         Ok(())
///     }
/// }
/// ```
pub trait BeanFactoryInitializer: Send + Sync + 'static {
    /// 初始化 BeanFactory。
    ///
    /// 对应 Spring 的 `void initialize(ConfigurableListableBeanFactory factory)`。
    ///
    /// # 参数
    ///
    /// * `factory` — 可配置的可列举 BeanFactory 的可变引用
    ///
    /// # 返回
    ///
    /// - `Ok(())` — 初始化成功
    /// - `Err` — 初始化失败
    fn initialize(
        &self,
        factory: &mut dyn ConfigurableListableBeanFactory,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
