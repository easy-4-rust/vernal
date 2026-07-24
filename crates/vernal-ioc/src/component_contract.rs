//! 可派生组件定义契约。

use std::any::Any;

use crate::ComponentDefinition;

/// 由过程宏或调用方实现的静态组件定义契约。
///
/// 该 trait 不承担实例存储、全局自动发现或运行时 Service Locator 职责，只把
/// 一个 Rust 类型映射为不可变 [`ComponentDefinition`]。应用仍须显式调用
/// [`crate::RegistryBuilder::register`]，使注册来源、依赖图和启动顺序保持可见。
pub trait Component: Any + Send + Sync + Sized {
    /// 创建该类型的组件定义。
    ///
    /// `#[derive(vernal_macros::Component)]` 会根据结构体的 `Arc<T>` 字段生成
    /// 构造器和 `depends_on::<T>()` 元数据。
    #[must_use]
    fn definition() -> ComponentDefinition;
}
