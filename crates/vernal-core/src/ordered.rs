//! 初始化排序常量。
//!
//! 对标 `tx_di` 的 `init_sort` 模式：同一拓扑深度的组件按此值升序排列。
//! 提供三个标准层级常量，供 `#[component(init_sort = N)]` 使用。
//!
//! # 设计来源
//!
//! `tx_di` 使用 `init_sort = i32::MIN` 让日志组件最先初始化，`init_sort = i32::MAX` 让
//! Web 服务器最后启动。vernal 将这个模式标准化为常量。
//!
//! # 与 Spring `Ordered` 的对齐
//!
//! | vernal-core 常量 | 数值 | Spring 等价 |
//! |---|---|---|
//! | [`INIT_SORT_INFRASTRUCTURE`](Self::INIT_SORT_INFRASTRUCTURE) | `i32::MIN + 1` | `Ordered.HIGHEST_PRECEDENCE = Integer.MIN_VALUE` |
//! | [`INIT_SORT_BEAN_FACTORY`](Self::INIT_SORT_BEAN_FACTORY) | `i32::MIN + 2` | `BeanPostProcessor` 注册期 |
//! | [`INIT_SORT_EVENT_LISTENER`](Self::INIT_SORT_EVENT_LISTENER) | `-2_000_000_000` | `ApplicationListener` 注册期 |
//! | [`INIT_SORT_MESSAGE_SOURCE`](Self::INIT_SORT_MESSAGE_SOURCE) | `-1_000_000_000` | `MessageSource` 注册期 |
//! | [`INIT_SORT_BUSINESS`](Self::INIT_SORT_BUSINESS) | `0` | 默认业务 Bean（普通 `Ordered`） |
//! | [`INIT_SORT_APPLICATION`](Self::INIT_SORT_APPLICATION) | `i32::MAX - 1` | Web 服务器 |
//! | [`INIT_SORT_TASK`](Self::INIT_SORT_TASK) | `i32::MAX - 100` | 异步任务启动 |
//! | [`INIT_SORT_DEFAULT`](Self::INIT_SORT_DEFAULT) | `i32::MAX` | `Ordered.LOWEST_PRECEDENCE = Integer.MAX_VALUE` |
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

/// 业务组件排序值（默认）。
///
/// 大多数业务组件使用此值。
pub const INIT_SORT_BUSINESS: i32 = 0;

/// 应用层排序值（Web 服务器等）。
///
/// 使用此值的组件会在所有业务组件之后初始化。
pub const INIT_SORT_APPLICATION: i32 = i32::MAX - 100;

/// 异步任务启动排序值（在 Web 服务器启动后才启动后台 worker）。
///
/// 用于 tokio task / 后台 worker 等长生命周期任务；比应用层再晚一档。
pub const INIT_SORT_TASK: i32 = i32::MAX - 1;

/// 默认排序值（最晚初始化）。
///
/// 当未指定 `init_order` 时使用的默认值。
pub const INIT_SORT_DEFAULT: i32 = i32::MAX;

/// Spring `Ordered.HIGHEST_PRECEDENCE` 等价值。
///
/// 与 [`INIT_SORT_INFRASTRUCTURE`] 同值，便于 Spring 用户迁移。
pub const HIGHEST_PRECEDENCE: i32 = INIT_SORT_INFRASTRUCTURE;

/// Spring `Ordered.LOWEST_PRECEDENCE` 等价值。
///
/// 与 [`INIT_SORT_DEFAULT`] 同值，便于 Spring 用户迁移。
pub const LOWEST_PRECEDENCE: i32 = INIT_SORT_DEFAULT;

#[cfg(test)]
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
    fn spring_alias_semantics() {
        // Spring `Ordered.HIGHEST_PRECEDENCE` = Integer.MIN_VALUE = -2147483648
        // vernal-core 选择 i32::MIN + 1 = -2147483647 以保留 1 slot
        assert_eq!(HIGHEST_PRECEDENCE, -2_147_483_647);
        // Spring `Ordered.LOWEST_PRECEDENCE` = Integer.MAX_VALUE = 2147483647
        assert_eq!(LOWEST_PRECEDENCE, 2_147_483_647);
    }
}
