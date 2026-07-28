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

/// 优先排序标记 trait。
///
/// 对标 Spring `PriorityOrdered extends Ordered`。
///
/// 实现此 trait 的组件**总是**排在普通 `Ordered` 组件之前。
/// Spring 中 `PriorityOrdered` 的典型实现者：
/// - `ConfigurationClassPostProcessor`（配置类处理）
/// - `AutowiredAnnotationBeanPostProcessor`（自动注入处理）
/// - `CommonAnnotationBeanPostProcessor`（JSR-250 注解处理）
///
/// # 与 `INIT_SORT_*` 常量的关系
///
/// `PriorityOrdered` 是**类型级**标记（编译期确定），
/// `INIT_SORT_*` 是**值级**排序（运行时比较）。
/// 两者可以组合使用：`PriorityOrdered` 组件内部仍可用 `INIT_SORT_*` 控制相对顺序。
///
/// # 示例
///
/// ```rust
/// use vernal_core::ordered::PriorityOrdered;
///
/// struct BeanFactoryProcessor;
///
/// impl PriorityOrdered for BeanFactoryProcessor {
///     fn order(&self) -> i32 {
///         0 // 最高优先级组内的顺序
///     }
/// }
///
/// // PriorityOrdered 组件总是排在普通 Ordered 组件之前
/// ```
pub trait PriorityOrdered {
    /// 获取排序值（值越小越先执行）。
    ///
    /// 对标 Spring `Ordered.getOrder()`。
    fn order(&self) -> i32 {
        0
    }

    /// 是否为优先排序组件（始终返回 true）。
    ///
    /// 用于类型级区分 `PriorityOrdered` 和普通 `Ordered`。
    fn is_priority_ordered(&self) -> bool {
        true
    }
}

/// 普通排序 trait。
///
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
    /// 对标 Spring `Ordered.getOrder()`。
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

/// 排序比较器。
///
/// 对标 Spring `OrderComparator`。
///
/// 用于对实现了 `Ordered` 或 `PriorityOrdered` 的组件进行排序。
/// `PriorityOrdered` 组件总是排在普通 `Ordered` 组件之前。
///
/// # 排序规则
///
/// 1. `PriorityOrdered` 组件排在 `Ordered` 组件之前
/// 2. 同组内按 `order()` 返回值升序排列
/// 3. `order()` 值相同则保持原始顺序（稳定排序）
///
/// # 示例
///
/// ```rust
/// use vernal_core::ordered::{Ordered, PriorityOrdered, OrderComparator};
///
/// struct HighPriority;
/// impl PriorityOrdered for HighPriority {
///     fn order(&self) -> i32 { 100 }
/// }
///
/// struct LowPriority;
/// impl Ordered for LowPriority {
///     fn order(&self) -> i32 { 0 }
/// }
///
/// // HighPriority 总是排在 LowPriority 之前，即使 order() 值更大
/// ```
#[derive(Debug, Clone, Copy)]
pub struct OrderComparator;

impl OrderComparator {
    /// 比较两个组件的排序优先级。
    ///
    /// 返回 `std::cmp::Ordering`：
    /// - `Less` 表示 `a` 排在 `b` 前面
    /// - `Greater` 表示 `a` 排在 `b` 后面
    /// - `Equal` 表示两者排序相同
    #[must_use]
    pub fn compare(
        a_priority: bool,
        a_order: i32,
        b_priority: bool,
        b_order: i32,
    ) -> std::cmp::Ordering {
        // PriorityOrdered 总是排在 Ordered 之前
        match (a_priority, b_priority) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a_order.cmp(&b_order),
        }
    }

    /// 对组件列表进行排序。
    ///
    /// 对标 Spring `OrderComparator.sort(List)`。
    ///
    /// # 参数
    ///
    /// - `items`: 要排序的组件列表
    /// - `get_priority`: 获取组件是否为 PriorityOrdered 的闭包
    /// - `get_order`: 获取组件排序值的闭包
    pub fn sort<T, F, G>(items: &mut [T], get_priority: F, get_order: G)
    where
        F: Fn(&T) -> bool,
        G: Fn(&T) -> i32,
    {
        items.sort_by(|a, b| {
            Self::compare(get_priority(a), get_order(a), get_priority(b), get_order(b))
        });
    }
}

impl Ordered for OrderComparator {
    fn order(&self) -> i32 {
        LOWEST_PRECEDENCE
    }
}

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

    // ── PriorityOrdered / Ordered trait 测试 ──

    #[test]
    fn priority_ordered_trait_default_order() {
        struct TestPriority;
        impl PriorityOrdered for TestPriority {}

        let p = TestPriority;
        assert_eq!(p.order(), 0);
        assert!(p.is_priority_ordered());
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
    fn order_comparator_priority_beats_ordered() {
        // PriorityOrdered 总是排在 Ordered 之前
        let ordering = OrderComparator::compare(true, 100, false, 0);
        assert_eq!(ordering, std::cmp::Ordering::Less);

        let ordering = OrderComparator::compare(false, 0, true, 100);
        assert_eq!(ordering, std::cmp::Ordering::Greater);
    }

    #[test]
    fn order_comparator_same_priority_sorts_by_order() {
        // 同组内按 order() 值升序
        let ordering = OrderComparator::compare(false, 1, false, 2);
        assert_eq!(ordering, std::cmp::Ordering::Less);

        let ordering = OrderComparator::compare(false, 2, false, 1);
        assert_eq!(ordering, std::cmp::Ordering::Greater);

        let ordering = OrderComparator::compare(false, 1, false, 1);
        assert_eq!(ordering, std::cmp::Ordering::Equal);
    }

    #[test]
    fn order_comparator_sort_mixed_list() {
        #[derive(Debug, PartialEq)]
        struct Item {
            name: String,
            priority: bool,
            order: i32,
        }

        let mut items = vec![
            Item {
                name: "C".into(),
                priority: false,
                order: 0,
            },
            Item {
                name: "A".into(),
                priority: true,
                order: 100,
            },
            Item {
                name: "B".into(),
                priority: false,
                order: -10,
            },
            Item {
                name: "D".into(),
                priority: true,
                order: 200,
            },
        ];

        OrderComparator::sort(&mut items, |item| item.priority, |item| item.order);

        // PriorityOrdered 组件排在前面，同组内按 order 升序
        assert_eq!(items[0].name, "A"); // priority=true, order=100
        assert_eq!(items[1].name, "D"); // priority=true, order=200
        assert_eq!(items[2].name, "B"); // priority=false, order=-10
        assert_eq!(items[3].name, "C"); // priority=false, order=0
    }

    #[test]
    fn order_comparator_sort_stable() {
        #[derive(Debug, PartialEq)]
        struct Item {
            name: String,
            order: i32,
        }

        let mut items = vec![
            Item {
                name: "A".into(),
                order: 1,
            },
            Item {
                name: "B".into(),
                order: 1,
            },
            Item {
                name: "C".into(),
                order: 1,
            },
        ];

        OrderComparator::sort(&mut items, |_| false, |item| item.order);

        // 稳定排序：相同 order 保持原始顺序
        assert_eq!(items[0].name, "A");
        assert_eq!(items[1].name, "B");
        assert_eq!(items[2].name, "C");
    }
}
