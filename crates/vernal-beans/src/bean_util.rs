//! Bean 工具类。
//!
//! 对标 hutool-core 的 `BeanUtil`。
//! 提供属性描述符查询等实用功能。

use std::any::TypeId;

use super::bean_descriptor::PropertyDescriptor;

/// Bean 工具类。
///
/// 对标 hutool-core 的 `BeanUtil`。
/// 提供属性描述符相关的工具方法。
pub struct BeanUtil;

impl BeanUtil {
    /// 检查两个类型是否相同。
    #[must_use]
    pub fn type_eq<A: 'static, B: 'static>() -> bool {
        TypeId::of::<A>() == TypeId::of::<B>()
    }

    /// 获取类型的 TypeId。
    #[must_use]
    pub fn type_id_of<T: 'static>() -> TypeId {
        TypeId::of::<T>()
    }

    /// 检查属性是否为可选类型。
    #[must_use]
    pub fn is_optional(property: &PropertyDescriptor) -> bool {
        property.optional
    }

    /// 检查属性是否有默认值。
    #[must_use]
    pub fn has_default(property: &PropertyDescriptor) -> bool {
        property.has_default
    }
}
