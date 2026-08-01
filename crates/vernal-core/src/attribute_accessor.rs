//! 属性访问器契约。
//!
//! 对标 Spring `org.springframework.core.AttributeAccessor`。

use std::any::Any;

/// 属性访问器契约。
///
/// 对应 Java: org.springframework.core.AttributeAccessor
///
/// Spring 语义：按名称附加/查询/移除任意属性（对标 `BeanWrapper` 的
/// attribute 能力）。
pub trait AttributeAccessor: Send + Sync {
    /// 设置属性值。
    fn set_attribute(&mut self, name: &str, value: Box<dyn Any + Send + Sync>);

    /// 按名称取属性值。
    fn get_attribute(&self, name: &str) -> Option<&dyn Any>;

    /// 移除属性并返回旧值。
    fn remove_attribute(&mut self, name: &str) -> Option<Box<dyn Any + Send + Sync>>;

    /// 判断属性是否存在。
    fn has_attribute(&self, name: &str) -> bool;

    /// 返回全部属性名。
    fn attribute_names(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AttributeAccessorSupport;

    #[test]
    fn support_satisfies_contract() {
        // D 类（重构安全）：`AttributeAccessorSupport` 实现该契约
        fn assert_accessor<T: AttributeAccessor>() {}
        assert_accessor::<AttributeAccessorSupport>();
    }
}
