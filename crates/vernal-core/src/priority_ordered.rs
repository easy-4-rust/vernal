//! 优先排序标记契约。
//!
//! 对标 Spring `org.springframework.core.PriorityOrdered`。

/// 优先排序标记 trait。
///
/// 对应 Java: org.springframework.core.PriorityOrdered
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
/// use vernal_core::priority_ordered::PriorityOrdered;
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
    /// 对应 Java: `Ordered#getOrder()`
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_ordered_trait_default_order() {
        struct TestPriority;
        impl PriorityOrdered for TestPriority {}

        let p = TestPriority;
        assert_eq!(p.order(), 0);
        assert!(p.is_priority_ordered());
    }

    #[test]
    fn priority_ordered_custom_order() {
        // 对标 Spring: PriorityOrdered 组件可以有自定义 order
        struct CustomPriority;
        impl PriorityOrdered for CustomPriority {
            fn order(&self) -> i32 {
                42
            }
        }

        let p = CustomPriority;
        assert_eq!(p.order(), 42);
        assert!(p.is_priority_ordered());
    }
}
