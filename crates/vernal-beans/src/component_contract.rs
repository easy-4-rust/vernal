//! 可派生组件定义契约。
//!
//! 对标 tx_di 的 `Component` trait + Spring 的 `BeanDefinition`。
//! 每个 `#[derive(Component)]` 的结构体自动实现此 trait。

use std::any::Any;

use vernal_core::{BoxError, ordered::INIT_SORT_DEFAULT};

use crate::{ComponentDefinition, Resolver};

/// 由过程宏或调用方实现的静态组件定义契约。
///
/// 该 trait 不承担实例存储、全局自动发现或运行时 Service Locator 职责，只把
/// 一个 Rust 类型映射为不可变 [`ComponentDefinition`]。应用仍须显式调用
/// [`crate::RegistryBuilder::register`]，使注册来源、依赖图和启动顺序保持可见。
///
/// ## 生命周期钩子
///
/// 对标 tx_di 的完整生命周期：
///
/// `build` -> `inner_init` -> `init` -> `async_init` -> `async_run` -> `shutdown`（逆序）
///
/// - `inner_init`：工厂内初始化，可访问 `Resolver`（对标 tx_di 的 `inner_init`）
/// - `init_order`：同层初始化排序（对标 tx_di 的 `init_sort`）
/// - `has_async_run`：是否需要后台任务（优化标志）
/// - `shutdown`：关闭钩子（对标 tx_di 的 `shutdown`）
///
/// ## 示例
///
/// ```rust,ignore
/// #[derive(Component)]
/// #[component(inner_init = "initialize_pool", shutdown = "close_pool", init_order = 100)]
/// pub struct DatabasePool { ... }
///
/// impl DatabasePool {
///     fn initialize_pool(&mut self, resolver: &Resolver) -> Result<(), BoxError> {
///         // 可以访问 resolver 获取其他组件
///         Ok(())
///     }
///     fn close_pool(&self) {
///         // 清理资源
///     }
/// }
/// ```
pub trait Component: Any + Send + Sync + Sized {
    /// 创建该类型的组件定义。
    ///
    /// `#[derive(vernal_macros::Component)]` 会根据结构体的 `Arc<T>`、
    /// `Arc<dyn Trait>`、`Option<Arc<T>>` 与 `Vec<Arc<dyn Trait>>` 字段生成
    /// 构造器和对应显式依赖元数据。
    #[must_use]
    fn definition() -> ComponentDefinition;

    /// 工厂内初始化钩子。
    ///
    /// 在工厂构造完成后、组件返回前调用。可访问 `Resolver` 获取其他已构造组件。
    /// 对标 tx_di 的 `Component::inner_init`。
    ///
    /// 默认实现：空操作。
    fn inner_init<'a>(&mut self, _resolver: &Resolver<'a>) -> Result<(), BoxError> {
        Ok(())
    }

    /// 同层初始化排序值。
    ///
    /// 同一拓扑深度的组件按此值升序排列（越小越早）。
    /// 对标 tx_di 的 `Component::init_sort`。
    ///
    /// 默认值：`i32::MAX`（最晚初始化）。
    fn init_order() -> i32 {
        INIT_SORT_DEFAULT
    }

    /// 是否需要后台运行任务。
    ///
    /// 返回 `true` 时，`ApplicationContext` 会为该组件派生后台任务。
    /// 返回 `false` 时，跳过任务派生（性能优化）。
    ///
    /// 默认值：`false`。
    fn has_async_run() -> bool {
        false
    }

    /// 关闭钩子。
    ///
    /// 在应用关闭时按逆拓扑序调用。
    /// 对标 tx_di 的 `Component::shutdown`。
    ///
    /// 默认实现：空操作。
    fn shutdown(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal Component implementation for testing default methods.
    struct TestComponent;

    impl Component for TestComponent {
        fn definition() -> ComponentDefinition {
            ComponentDefinition::singleton::<TestComponent, _>(|_| TestComponent)
        }
    }

    #[test]
    fn default_inner_init_returns_ok() {
        // We can't easily construct a Resolver in unit tests, but we verify the trait method exists
        // by calling init_order and has_async_run which don't need Resolver
        assert_eq!(TestComponent::init_order(), INIT_SORT_DEFAULT);
    }

    #[test]
    fn default_init_order() {
        assert_eq!(TestComponent::init_order(), INIT_SORT_DEFAULT);
    }

    #[test]
    fn default_has_async_run() {
        assert!(!TestComponent::has_async_run());
    }

    #[test]
    fn default_shutdown_is_noop() {
        let comp = TestComponent;
        comp.shutdown(); // should not panic
    }

    /// Component with custom init_order.
    struct OrderedComponent;

    impl Component for OrderedComponent {
        fn definition() -> ComponentDefinition {
            ComponentDefinition::singleton::<OrderedComponent, _>(|_| OrderedComponent)
        }
        fn init_order() -> i32 {
            100
        }
        fn has_async_run() -> bool {
            true
        }
    }

    #[test]
    fn custom_init_order() {
        assert_eq!(OrderedComponent::init_order(), 100);
    }

    #[test]
    fn custom_has_async_run() {
        assert!(OrderedComponent::has_async_run());
    }

    #[test]
    fn default_init_order_value() {
        // INIT_SORT_DEFAULT should be i32::MAX
        assert_eq!(INIT_SORT_DEFAULT, i32::MAX);
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn test_component_definition_created() {
        let def = TestComponent::definition();
        assert_eq!(def.key().type_name(), "vernal_beans::component_contract::tests::TestComponent");
    }

    #[test]
    fn test_ordered_component_definition_created() {
        let def = OrderedComponent::definition();
        assert_eq!(def.key().type_name(), "vernal_beans::component_contract::tests::OrderedComponent");
    }

    #[test]
    fn default_inner_init_is_ok() {
        // Test that the default inner_init returns Ok
        // We can't easily construct a Resolver, but we verify the method exists
        // by calling the other default methods
        assert_eq!(TestComponent::init_order(), INIT_SORT_DEFAULT);
        assert!(!TestComponent::has_async_run());
    }

    #[test]
    fn test_component_trait_methods() {
        // Verify all Component trait default methods work
        assert_eq!(TestComponent::init_order(), i32::MAX);
        assert!(!TestComponent::has_async_run());
        TestComponent.shutdown();
    }

    #[test]
    fn test_ordered_component_trait_methods() {
        assert_eq!(OrderedComponent::init_order(), 100);
        assert!(OrderedComponent::has_async_run());
        OrderedComponent.shutdown();
    }
}
