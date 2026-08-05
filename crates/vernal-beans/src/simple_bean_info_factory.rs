//! SimpleBeanInfoFactory — 对应 Spring `org.springframework.beans.SimpleBeanInfoFactory`。
//!
//! 简单的 BeanInfo 工厂。

use std::any::TypeId;

use crate::bean_info_factory::{BeanInfoEntries, BeanInfoFactory};

/// 简单的 BeanInfo 工厂。
///
/// 对应 Java 类：`org.springframework.beans.SimpleBeanInfoFactory`。
pub struct SimpleBeanInfoFactory;

impl BeanInfoFactory for SimpleBeanInfoFactory {
    fn get_bean_info_entries(&self, _type_id: TypeId) -> Option<BeanInfoEntries> {
        Some(BeanInfoEntries::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory() {
        let factory = SimpleBeanInfoFactory;
        let entries = factory.get_bean_info_entries(TypeId::of::<String>());
        assert!(entries.is_some());
    }
}
