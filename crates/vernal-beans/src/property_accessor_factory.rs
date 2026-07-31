//! PropertyAccessorFactory — 对应 Spring `org.springframework.beans.PropertyAccessorFactory`。
//!
//! 属性访问器工厂。

use crate::bean_wrapper_impl::BeanWrapperImpl;

/// 属性访问器工厂。
///
/// 对应 Java 类：`org.springframework.beans.PropertyAccessorFactory`。
///
/// 提供了创建 PropertyAccessor 实例的工厂方法。
pub struct PropertyAccessorFactory;

impl PropertyAccessorFactory {
    /// 创建一个 BeanWrapper。
    ///
    /// 对应 Java 方法：`PropertyAccessor forBeanPropertyAccess(Object target)`
    pub fn for_bean_property_access(target: impl std::any::Any + Send + Sync + 'static) -> BeanWrapperImpl {
        BeanWrapperImpl::new(std::sync::Arc::new(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_bean_property_access() {
        let wrapper = PropertyAccessorFactory::for_bean_property_access(String::from("test"));
        assert_eq!(wrapper.property_count(), 0);
    }
}
