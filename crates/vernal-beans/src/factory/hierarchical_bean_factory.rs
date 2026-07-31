//! HierarchicalBeanFactory — Spring 风格的层级 BeanFactory 接口。
//!
//! 对应 Java 类：`org.springframework.beans.factory.HierarchicalBeanFactory`。
//!
//! 扩展 `BeanFactory`，支持父子容器层级结构。

use std::sync::Arc;

use crate::factory::bean_factory::BeanFactory;

/// Spring 风格的层级 BeanFactory 接口。
///
/// 对应 Spring 的 `HierarchicalBeanFactory`。
///
/// 提供父子容器层级结构：
/// - 子容器可以委托父容器解析 Bean
/// - 父容器不能访问子容器的 Bean
/// - 子容器的同名 Bean 覆盖父容器的同名 Bean
pub trait HierarchicalBeanFactory: BeanFactory {
    /// 获取父 BeanFactory（类型擦除）。
    ///
    /// 对应 Spring 的 `BeanFactory getParentBeanFactory()`。
    ///
    /// # 返回
    ///
    /// - `Some(parent)` — 父容器的 Arc 引用
    /// - `None` — 此容器是根容器
    fn parent_bean_factory(&self) -> Option<Arc<dyn BeanFactory>>;

    /// 检查本地（非父容器）是否包含指定 Bean。
    ///
    /// 对应 Spring 的 `boolean containsLocalBean(String name)`。
    fn contains_local_bean(&self, name: &str) -> bool;
}
