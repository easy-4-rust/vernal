//! Aware — Spring 风格的容器感知标记接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.Aware`。
//!
//! `Aware` 是一个标记接口（marker interface），Bean 实现它后，容器在创建
//! Bean 时会回调对应的 `set*` 方法，将容器内部组件注入 Bean。
//!
//! ## 与 vernal 现有体系的关系
//!
//! - `BeanNameAware` → 在 `vernal-beans` 中实现，由 `Component::inner_init` 触发
//! - `BeanFactoryAware` → 对应 `Component::inner_init(&Resolver)`
//! - 其余 Aware 接口（EnvironmentAware、ApplicationContextAware 等）由 `vernal-context` 承接
//!
//! ## 设计
//!
//! 采用 marker trait + 分离的回调 trait，避免在单个 `Aware` 中暴露所有方法。
//! 容器按 `is::<XxxAware>()` 检测后调用对应的 `set_xxx` 方法。

/// Spring 风格的标记 trait。
///
/// 对应 Spring 的 `Aware` marker interface。
/// 实现此 trait 后，容器在创建 Bean 时回调具体的 `set_xxx` 方法。
///
/// # 示例
///
/// ```rust,ignore
/// use vernal_beans::aware::Aware;
/// use vernal_beans::bean_name_aware::BeanNameAware;
///
/// struct MyService;
/// impl Aware for MyService {}
/// impl BeanNameAware for MyService {
///     fn set_bean_name(&mut self, name: &str) {
///         println!("Bean name: {}", name);
///     }
/// }
/// ```
pub trait Aware: Send + Sync + 'static {}
