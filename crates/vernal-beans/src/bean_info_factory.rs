//! BeanInfoFactory — 对应 Spring `org.springframework.beans.BeanInfoFactory`。
//!
//! BeanInfo 工厂接口。

use std::any::TypeId;

/// BeanInfo 工厂接口。
///
/// 对应 Java 接口：`org.springframework.beans.BeanInfoFactory`。
///
/// 用于创建 BeanInfo 实例。
pub trait BeanInfoFactory: Send + Sync {
    /// 获取指定类型的 BeanInfo 条目。
    ///
    /// 对应 Java 方法：`BeanInfo getBeanInfo(Class<?> beanClass)`
    fn get_bean_info_entries(&self, type_id: TypeId) -> Option<BeanInfoEntries>;
}

/// BeanInfo 条目集合。
#[derive(Debug, Clone)]
pub struct BeanInfoEntries {
    /// 属性描述符。
    properties: Vec<PropertyDescriptorEntry>,
    /// 方法描述符。
    methods: Vec<MethodDescriptorEntry>,
}

/// 属性描述符条目。
#[derive(Debug, Clone)]
pub struct PropertyDescriptorEntry {
    name: String,
    property_type: TypeId,
    readable: bool,
    writable: bool,
}

impl PropertyDescriptorEntry {
    pub fn new(name: impl Into<String>, property_type: TypeId, readable: bool, writable: bool) -> Self {
        Self {
            name: name.into(),
            property_type,
            readable,
            writable,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn property_type(&self) -> TypeId {
        self.property_type
    }

    pub fn is_readable(&self) -> bool {
        self.readable
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }
}

/// 方法描述符条目。
#[derive(Debug, Clone)]
pub struct MethodDescriptorEntry {
    name: String,
    parameter_count: usize,
}

impl MethodDescriptorEntry {
    pub fn new(name: impl Into<String>, parameter_count: usize) -> Self {
        Self {
            name: name.into(),
            parameter_count,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parameter_count(&self) -> usize {
        self.parameter_count
    }
}

impl BeanInfoEntries {
    /// 创建一个新的 BeanInfoEntries。
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
            methods: Vec::new(),
        }
    }

    /// 添加属性描述符。
    pub fn add_property(&mut self, descriptor: PropertyDescriptorEntry) {
        self.properties.push(descriptor);
    }

    /// 添加方法描述符。
    pub fn add_method(&mut self, descriptor: MethodDescriptorEntry) {
        self.methods.push(descriptor);
    }

    /// 获取所有属性描述符。
    pub fn properties(&self) -> &[PropertyDescriptorEntry] {
        &self.properties
    }

    /// 获取所有方法描述符。
    pub fn methods(&self) -> &[MethodDescriptorEntry] {
        &self.methods
    }

    /// 获取属性数量。
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// 获取方法数量。
    pub fn method_count(&self) -> usize {
        self.methods.len()
    }
}

impl Default for BeanInfoEntries {
    fn default() -> Self {
        Self::new()
    }
}

/// BeanInfoFactory 的简单实现。
#[derive(Debug)]
pub struct SimpleBeanInfoFactory;

impl BeanInfoFactory for SimpleBeanInfoFactory {
    fn get_bean_info_entries(&self, _type_id: TypeId) -> Option<BeanInfoEntries> {
        // 默认实现返回空的 BeanInfoEntries
        Some(BeanInfoEntries::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bean_info_entries() {
        let mut entries = BeanInfoEntries::new();
        entries.add_property(PropertyDescriptorEntry::new("name", TypeId::of::<String>(), true, true));
        entries.add_method(MethodDescriptorEntry::new("getName", 0));

        assert_eq!(entries.property_count(), 1);
        assert_eq!(entries.method_count(), 1);
    }

    #[test]
    fn test_simple_factory() {
        let factory = SimpleBeanInfoFactory;
        let entries = factory.get_bean_info_entries(TypeId::of::<String>());
        assert!(entries.is_some());
    }
}
