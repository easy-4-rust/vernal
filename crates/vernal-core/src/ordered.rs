//! 初始化排序常量与普通排序契约。
//!
//! 对标 Spring `org.springframework.core.Ordered`。
//!
//! `INIT_SORT_*` 是 vernal 对 `tx_di` `init_sort` 模式的标准化：同一拓扑深度的组件按此值
//! 升序排列。`Ordered` trait 对应 Spring 的 `Ordered` 接口；`PriorityOrdered` 与
//! `OrderComparator` 分别见 `priority_ordered.rs` 与 `order_comparator.rs`。
//!
//! # 与 Spring `Ordered` 的对齐
//!
//! | vernal-core 常量 | 数值 | Spring 等价 |
//! |---|---|---|
//! | [`INIT_SORT_INFRASTRUCTURE`] | `i32::MIN + 1` | `Ordered.HIGHEST_PRECEDENCE = Integer.MIN_VALUE` |
//! | [`INIT_SORT_BEAN_FACTORY`] | `i32::MIN + 2` | `BeanPostProcessor` 注册期 |
//! | [`INIT_SORT_EVENT_LISTENER`] | `-2_000_000_000` | `ApplicationListener` 注册期 |
//! | [`INIT_SORT_MESSAGE_SOURCE`] | `-1_000_000_000` | `MessageSource` 注册期 |
//! | [`INIT_SORT_BUSINESS`] | `0` | 默认业务 Bean（普通 `Ordered`） |
//! | [`INIT_SORT_APPLICATION`] | `i32::MAX - 100` | Web 服务器 |
//! | [`INIT_SORT_TASK`] | `i32::MAX - 1` | 异步任务启动 |
//! | [`INIT_SORT_DEFAULT`] | `i32::MAX` | `Ordered.LOWEST_PRECEDENCE = Integer.MAX_VALUE` |
//!
//! # 偏序关系（必须保持不变）
//!
//! ```text
//! INFRASTRUCTURE < BEAN_FACTORY < EVENT_LISTENER < MESSAGE_SOURCE
//!              < BUSINESS < APPLICATION < TASK < DEFAULT
//! ```

/// 基础设施层排序值（日志、配置中心等）。
///
/// 对标 `tx_di` 的 `init_sort = i32::MIN`。
/// 使用此值的组件会在所有业务组件之前初始化。
pub const INIT_SORT_INFRASTRUCTURE: i32 = i32::MIN + 1;

/// `BeanFactory` 初始化排序值（注册 `BeanPostProcessor` 等）。
///
/// 对标 Spring `BeanPostProcessor` 注册期（在 `BeanFactory` 准备就绪后、业务 Bean 创建前）。
pub const INIT_SORT_BEAN_FACTORY: i32 = i32::MIN + 2;

/// 事件监听器注册排序值。
///
/// 对标 Spring `ApplicationListener` 注册期（业务开始前）。
pub const INIT_SORT_EVENT_LISTENER: i32 = -2_000_000_000;

/// 消息源（i18n）初始化排序值。
///
/// 对标 Spring `MessageSource` 注册期（事件监听器之后）。
pub const INIT_SORT_MESSAGE_SOURCE: i32 = -1_000_000_000;

/// 默认业务组件排序值。
///
/// 对标 Spring 普通 `Ordered` 组件（`order()` 返回 0）。
pub const INIT_SORT_BUSINESS: i32 = 0;

/// 应用层（Web 服务器等）排序值。
///
/// 对标 Spring `WebServerInitializedEvent` 之后启动的组件。
pub const INIT_SORT_APPLICATION: i32 = i32::MAX - 100;

/// 异步任务启动排序值。
///
/// 对标 Spring 后台任务在应用层之后、兜底默认值之前启动。
pub const INIT_SORT_TASK: i32 = i32::MAX - 1;

/// 未显式指定时的兜底默认排序值。
///
/// 对标 Spring `Ordered.LOWEST_PRECEDENCE = Integer.MAX_VALUE`。
pub const INIT_SORT_DEFAULT: i32 = i32::MAX;

/// 最高优先级别名（对标 Spring `Ordered.HIGHEST_PRECEDENCE`）。
pub const HIGHEST_PRECEDENCE: i32 = INIT_SORT_INFRASTRUCTURE;

/// 最低优先级别名（对标 Spring `Ordered.LOWEST_PRECEDENCE`）。
pub const LOWEST_PRECEDENCE: i32 = INIT_SORT_DEFAULT;

/// 普通排序 trait。
///
/// 对应 Java: org.springframework.core.Ordered
/// 对标 Spring `Ordered`。
///
/// 实现此 trait 的组件按 `order()` 返回值排序。
/// `PriorityOrdered` 组件总是排在此类组件之前。
///
/// # 示例
///
/// ```rust
/// use vernal_core::ordered::Ordered;
///
/// struct MyService;
///
/// impl Ordered for MyService {
///     fn order(&self) -> i32 {
///         100
///     }
/// }
/// ```
pub trait Ordered {
    /// 获取排序值（值越小越先执行）。
    ///
    /// 对应 Java: `Ordered#getOrder()`
    fn order(&self) -> i32 {
        0
    }

    /// 是否为优先排序组件（默认 false）。
    ///
    /// `PriorityOrdered` 组件覆盖此方法返回 true。
    fn is_priority_ordered(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[allow(clippy::assertions_on_constants, clippy::absurd_extreme_comparisons)]
// 偏序与极值断言是文档化不变量,故意用常量断言表达
mod tests {
    use super::*;

    #[test]
    fn partial_order_is_strictly_increasing() {
        // 验证文档承诺的偏序关系
        assert!(INIT_SORT_INFRASTRUCTURE < INIT_SORT_BEAN_FACTORY);
        assert!(INIT_SORT_BEAN_FACTORY < INIT_SORT_EVENT_LISTENER);
        assert!(INIT_SORT_EVENT_LISTENER < INIT_SORT_MESSAGE_SOURCE);
        assert!(INIT_SORT_MESSAGE_SOURCE < INIT_SORT_BUSINESS);
        assert!(INIT_SORT_BUSINESS < INIT_SORT_APPLICATION);
        assert!(INIT_SORT_APPLICATION < INIT_SORT_TASK);
        assert!(INIT_SORT_TASK < INIT_SORT_DEFAULT);
    }

    #[test]
    fn infrastructure_is_lowest_meaningful_value() {
        // 避开 i32::MIN 真正的极值，让用户有 1 个 slot 写"比基础设施还早"
        assert!(INIT_SORT_INFRASTRUCTURE > i32::MIN);
    }

    #[test]
    fn highest_precedence_alias_matches_infrastructure() {
        assert_eq!(HIGHEST_PRECEDENCE, INIT_SORT_INFRASTRUCTURE);
    }

    #[test]
    fn lowest_precedence_alias_matches_default() {
        assert_eq!(LOWEST_PRECEDENCE, INIT_SORT_DEFAULT);
    }

    #[test]
    fn task_runs_after_application_layer() {
        // 后台任务应该在 Web 服务器之后启动
        assert!(INIT_SORT_TASK > INIT_SORT_APPLICATION);
        // 但仍早于"完全不指定 init_sort"的兜底默认值
        assert!(INIT_SORT_TASK < INIT_SORT_DEFAULT);
    }

    #[test]
    fn all_constants_are_in_i32_range() {
        // 编译期断言（通过 const assert）
        const _: () = {
            assert!(INIT_SORT_INFRASTRUCTURE >= i32::MIN);
            assert!(INIT_SORT_DEFAULT <= i32::MAX);
        };
    }

    #[test]
    fn high_low_precedence_aliases() {
        // Spring `Ordered.HIGHEST_PRECEDENCE` = Integer.MIN_VALUE = -2147483648
        // vernal-core 选择 i32::MIN + 1 = -2147483647 以保留 1 slot
        assert_eq!(HIGHEST_PRECEDENCE, -2_147_483_647);
        // Spring `Ordered.LOWEST_PRECEDENCE` = Integer.MAX_VALUE = 2147483647
        assert_eq!(LOWEST_PRECEDENCE, 2_147_483_647);
    }

    #[test]
    fn ordered_trait_default_order() {
        struct TestOrdered;
        impl Ordered for TestOrdered {}

        let o = TestOrdered;
        assert_eq!(o.order(), 0);
        assert!(!o.is_priority_ordered());
    }

    #[test]
    fn ordered_trait_custom_order() {
        // 对标 Spring: 实现者覆盖 getOrder() 返回自定义值
        struct CustomOrdered;
        impl Ordered for CustomOrdered {
            fn order(&self) -> i32 {
                42
            }
        }

        let o = CustomOrdered;
        assert_eq!(o.order(), 42);
        assert!(!o.is_priority_ordered());
    }
}
