//! Bean 属性描述符。
//!
//! 对标 hutool-core 的 `BeanDesc` 和 Spring 的 `BeanInfo`。
//! 用于 `ConfigurationProperties::bind_with_prefix` 的属性绑定。
//!
//! # 设计原则
//!
//! - Rust 没有运行时反射，所以 BeanDescriptor 通过 trait 实现
//! - 每个需要属性描述的结构体实现 `BeanDescriptor` trait
//! - 提供属性名称列表和类型信息，用于配置绑定

use std::any::TypeId;

/// Bean 属性描述符 trait。
///
/// 对标 hutool-core 的 `BeanDesc`。
/// 为结构体提供属性元数据，用于配置绑定和诊断。
///
/// # 实现方式
///
/// 通常由 `#[derive(ConfigurationProperties)]` 宏自动生成。
pub trait BeanDescriptor: Send + Sync + 'static {
    /// 结构体名称。
    fn name(&self) -> &'static str;

    /// 属性列表。
    fn properties(&self) -> &[PropertyDescriptor];

    /// 查找指定名称的属性。
    fn find_property(&self, name: &str) -> Option<&PropertyDescriptor> {
        self.properties().iter().find(|p| p.name == name)
    }
}

/// 属性描述符。
///
/// 描述结构体的一个属性（字段）。
#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    /// 属性名称
    pub name: &'static str,
    /// 属性类型 ID
    pub type_id: TypeId,
    /// 属性类型名
    pub type_name: &'static str,
    /// 是否可选（Option<T>）
    pub optional: bool,
    /// 是否有默认值
    pub has_default: bool,
}

impl PropertyDescriptor {
    /// 创建属性描述符。
    #[must_use]
    pub const fn new(
        name: &'static str,
        type_id: TypeId,
        type_name: &'static str,
        optional: bool,
        has_default: bool,
    ) -> Self {
        Self {
            name,
            type_id,
            type_name,
            optional,
            has_default,
        }
    }
}
