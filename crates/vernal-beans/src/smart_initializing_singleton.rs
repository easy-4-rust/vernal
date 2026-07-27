//! SmartInitializingSingleton — Spring 风格的智能初始化回调接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.SmartInitializingSingleton`。
//!
//! 在所有 singleton Bean 实例化完成后调用。

/// Spring 风格的智能初始化回调接口。
///
/// 对应 Spring 的 `SmartInitializingSingleton`。
///
/// 在所有 singleton Bean 实例化完成后调用 `after_singletons_instantiated`。
/// 这是容器 refresh 阶段的最后一步。
///
/// ## 使用场景
///
/// - 启动后注册事件监听器
/// - 启动后执行健康检查
/// - 启动后发布 ready 事件
///
/// ## 与 InitializingBean 的区别
///
/// - `InitializingBean.afterPropertiesSet` — 单个 Bean 初始化时调用
/// - `SmartInitializingSingleton.after_singletons_instantiated` — 所有 singleton 初始化完成后调用
pub trait SmartInitializingSingleton: Send + Sync + 'static {
    /// 所有 singleton 实例化完成后调用。
    ///
    /// 对应 Spring 的 `SmartInitializingSingleton.afterSingletonsInstantiated()`。
    fn after_singletons_instantiated(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
