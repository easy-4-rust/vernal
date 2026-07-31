//! StandardBeanInfoFactory — 对应 Spring `org.springframework.beans.StandardBeanInfoFactory`。
//!
//! 标准的 BeanInfo 工厂。

use std::any::TypeId;

use crate::bean_info_factory::{BeanInfoFactory, BeanInfoEntries};

/// 标准的 BeanInfo 工厂。
///
/// 对应 Java 类：`org.springframework.beans.StandardBeanInfoFactory`。
pub struct StandardBeanInfoFactory;

impl BeanInfoFactory for StandardBeanInfoFactory {
    fn get_bean_info_entries(&self, _type_id: TypeId) -> Option<BeanInfoEntries> {
        Some(BeanInfoEntries::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory() {
        let factory = StandardBeanInfoFactory;
        let entries = factory.get_bean_info_entries(TypeId::of::<String>());
        assert!(entries.is_some());
    }
}
