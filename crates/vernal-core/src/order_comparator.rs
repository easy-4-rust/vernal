//! 排序比较器。
//!
//! 对标 Spring `org.springframework.core.OrderComparator`。

use crate::ordered::{LOWEST_PRECEDENCE, Ordered};

/// 排序比较器。
///
/// 对应 Java: org.springframework.core.OrderComparator
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
/// use vernal_core::order_comparator::OrderComparator;
/// use vernal_core::ordered::Ordered;
/// use vernal_core::priority_ordered::PriorityOrdered;
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
/// let ordering = OrderComparator::compare(true, 100, false, 0);
/// assert_eq!(ordering, std::cmp::Ordering::Less);
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
    /// 对应 Java: `OrderComparator#sort(List)`
    ///
    /// # 参数
    ///
    /// - `items`: 要排序的组件列表
    /// - `get_priority`: 获取组件是否为 `PriorityOrdered` 的闭包
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
    use crate::priority_ordered::PriorityOrdered;

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
            Item { name: "C".into(), priority: false, order: 0 },
            Item { name: "A".into(), priority: true, order: 100 },
            Item { name: "B".into(), priority: false, order: -10 },
            Item { name: "D".into(), priority: true, order: 200 },
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
            Item { name: "A".into(), order: 1 },
            Item { name: "B".into(), order: 1 },
            Item { name: "C".into(), order: 1 },
        ];

        OrderComparator::sort(&mut items, |_| false, |item| item.order);

        // 稳定排序：相同 order 保持原始顺序
        assert_eq!(items[0].name, "A");
        assert_eq!(items[1].name, "B");
        assert_eq!(items[2].name, "C");
    }

    #[test]
    fn order_comparator_same_priority_both_true() {
        // 两个 PriorityOrdered 组件按 order 值排序
        let ordering = OrderComparator::compare(true, 1, true, 2);
        assert_eq!(ordering, std::cmp::Ordering::Less);
    }

    #[test]
    fn order_comparator_default_trait() {
        // 对标 Spring: OrderComparator 实现了 Ordered，order = LOWEST_PRECEDENCE
        let comp = OrderComparator;
        assert_eq!(comp.order(), LOWEST_PRECEDENCE);
        assert!(!comp.is_priority_ordered());
    }

    #[test]
    fn priority_ordered_dispatch_through_comparator() {
        // PriorityOrdered 类型级标记在比较器中生效（B 类：跨对象集成）
        struct A;
        impl PriorityOrdered for A {}
        struct B;
        impl Ordered for B {}

        let ordering = OrderComparator::compare(
            <A as PriorityOrdered>::is_priority_ordered(&A),
            <A as PriorityOrdered>::order(&A),
            B.is_priority_ordered(),
            B.order(),
        );
        assert_eq!(ordering, std::cmp::Ordering::Less);
    }
}
