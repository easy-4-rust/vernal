//! ExtendedBeanInfoFactory — 对应 Spring `org.springframework.beans.ExtendedBeanInfoFactory`。
//!
//! ExtendedBeanInfo 工厂。

use std::any::TypeId;

use crate::bean_info_factory::BeanInfoFactory;

/// ExtendedBeanInfo 工厂。
///
/// 对应 Java 类：`org.springframework.beans.ExtendedBeanInfoFactory`。
pub struct ExtendedBeanInfoFactory;

impl BeanInfoFactory for ExtendedBeanInfoFactory {
    fn get_bean_info_entries(
        &self,
        _type_id: TypeId,
    ) -> Option<crate::bean_info_factory::BeanInfoEntries> {
        Some(crate::bean_info_factory::BeanInfoEntries::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory() {
        let factory = ExtendedBeanInfoFactory;
        let entries = factory.get_bean_info_entries(TypeId::of::<String>());
        assert!(entries.is_some());
    }
}
