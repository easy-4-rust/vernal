//! ExtendedBeanInfo — 对应 Spring `org.springframework.beans.ExtendedBeanInfo`。
//!
//! 扩展的 BeanInfo，支持更多元数据。

use std::any::TypeId;
use std::collections::HashMap;

/// 扩展的 BeanInfo。
///
/// 对应 Java 类：`org.springframework.beans.ExtendedBeanInfo`。
///
/// 提供了比标准 BeanInfo 更丰富的元数据信息。
#[derive(Debug)]
pub struct ExtendedBeanInfo {
    type_name: String,
    properties: HashMap<String, PropertyDescriptor>,
    methods: HashMap<String, MethodDescriptor>,
}

/// 属性描述符。
#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    name: String,
    property_type: TypeId,
    readable: bool,
    writable: bool,
}

impl PropertyDescriptor {
    pub fn new(name: impl Into<String>, property_type: TypeId, readable: bool, writable: bool) -> Self {
        Self { name: name.into(), property_type, readable, writable }
    }
    pub fn name(&self) -> &str { &self.name }
    pub fn property_type(&self) -> TypeId { self.property_type }
    pub fn is_readable(&self) -> bool { self.readable }
    pub fn is_writable(&self) -> bool { self.writable }
}

/// 方法描述符。
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    name: String,
    parameter_count: usize,
}

impl MethodDescriptor {
    pub fn new(name: impl Into<String>, parameter_count: usize) -> Self {
        Self { name: name.into(), parameter_count }
    }
    pub fn name(&self) -> &str { &self.name }
    pub fn parameter_count(&self) -> usize { self.parameter_count }
}

impl ExtendedBeanInfo {
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_name: type_name.into(),
            properties: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, descriptor: PropertyDescriptor) {
        self.properties.insert(descriptor.name.clone(), descriptor);
    }

    pub fn add_method(&mut self, descriptor: MethodDescriptor) {
        self.methods.insert(descriptor.name.clone(), descriptor);
    }

    pub fn get_property(&self, name: &str) -> Option<&PropertyDescriptor> {
        self.properties.get(name)
    }

    pub fn get_method(&self, name: &str) -> Option<&MethodDescriptor> {
        self.methods.get(name)
    }

    pub fn property_names(&self) -> Vec<String> {
        self.properties.keys().cloned().collect()
    }

    pub fn method_names(&self) -> Vec<String> {
        self.methods.keys().cloned().collect()
    }

    pub fn property_count(&self) -> usize { self.properties.len() }
    pub fn method_count(&self) -> usize { self.methods.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_bean_info() {
        let mut info = ExtendedBeanInfo::new("MyBean");
        info.add_property(PropertyDescriptor::new("name", TypeId::of::<String>(), true, true));
        info.add_method(MethodDescriptor::new("getName", 0));

        assert_eq!(info.property_count(), 1);
        assert_eq!(info.method_count(), 1);
        assert!(info.get_property("name").is_some());
    }

    #[test]
    fn test_empty_bean_info() {
        let info = ExtendedBeanInfo::new("Empty");
        assert_eq!(info.property_count(), 0);
        assert_eq!(info.method_count(), 0);
        assert!(info.property_names().is_empty());
        assert!(info.method_names().is_empty());
    }

    #[test]
    fn test_get_property_not_found() {
        let info = ExtendedBeanInfo::new("Test");
        assert!(info.get_property("missing").is_none());
    }

    #[test]
    fn test_get_method_not_found() {
        let info = ExtendedBeanInfo::new("Test");
        assert!(info.get_method("missing").is_none());
    }

    #[test]
    fn test_property_descriptor_fields() {
        let pd = PropertyDescriptor::new("age", TypeId::of::<i32>(), true, false);
        assert_eq!(pd.name(), "age");
        assert_eq!(pd.property_type(), TypeId::of::<i32>());
        assert!(pd.is_readable());
        assert!(!pd.is_writable());
    }

    #[test]
    fn test_property_descriptor_readonly() {
        let pd = PropertyDescriptor::new("id", TypeId::of::<u64>(), true, false);
        assert!(pd.is_readable());
        assert!(!pd.is_writable());
    }

    #[test]
    fn test_property_descriptor_writeonly() {
        let pd = PropertyDescriptor::new("secret", TypeId::of::<String>(), false, true);
        assert!(!pd.is_readable());
        assert!(pd.is_writable());
    }

    #[test]
    fn test_method_descriptor_fields() {
        let md = MethodDescriptor::new("calculate", 3);
        assert_eq!(md.name(), "calculate");
        assert_eq!(md.parameter_count(), 3);
    }

    #[test]
    fn test_method_descriptor_zero_params() {
        let md = MethodDescriptor::new("getName", 0);
        assert_eq!(md.parameter_count(), 0);
    }

    #[test]
    fn test_multiple_properties_and_methods() {
        let mut info = ExtendedBeanInfo::new("Service");
        info.add_property(PropertyDescriptor::new("name", TypeId::of::<String>(), true, true));
        info.add_property(PropertyDescriptor::new("age", TypeId::of::<i32>(), true, true));
        info.add_property(PropertyDescriptor::new("email", TypeId::of::<String>(), true, true));
        info.add_method(MethodDescriptor::new("getName", 0));
        info.add_method(MethodDescriptor::new("setName", 1));
        info.add_method(MethodDescriptor::new("getEmail", 0));

        assert_eq!(info.property_count(), 3);
        assert_eq!(info.method_count(), 3);

        let mut prop_names = info.property_names();
        prop_names.sort();
        assert_eq!(prop_names, vec!["age", "email", "name"]);

        let mut method_names = info.method_names();
        method_names.sort();
        // sorted lexicographically
        assert_eq!(method_names, vec!["getEmail", "getName", "setName"]);
    }

    #[test]
    fn test_add_property_overwrites() {
        let mut info = ExtendedBeanInfo::new("Test");
        info.add_property(PropertyDescriptor::new("name", TypeId::of::<String>(), true, true));
        info.add_property(PropertyDescriptor::new("name", TypeId::of::<i32>(), true, false));
        assert_eq!(info.property_count(), 1);
        let pd = info.get_property("name").unwrap();
        assert_eq!(pd.property_type(), TypeId::of::<i32>());
    }

    #[test]
    fn test_add_method_overwrites() {
        let mut info = ExtendedBeanInfo::new("Test");
        info.add_method(MethodDescriptor::new("doStuff", 1));
        info.add_method(MethodDescriptor::new("doStuff", 3));
        assert_eq!(info.method_count(), 1);
        let md = info.get_method("doStuff").unwrap();
        assert_eq!(md.parameter_count(), 3);
    }

    #[test]
    fn test_debug_format() {
        let info = ExtendedBeanInfo::new("TestBean");
        let debug = format!("{:?}", info);
        assert!(debug.contains("ExtendedBeanInfo"));
        assert!(debug.contains("TestBean"));
    }

    #[test]
    fn test_clone_property_descriptor() {
        let pd = PropertyDescriptor::new("x", TypeId::of::<f64>(), true, true);
        let cloned = pd.clone();
        assert_eq!(cloned.name(), "x");
        assert_eq!(cloned.property_type(), TypeId::of::<f64>());
    }

    #[test]
    fn test_clone_method_descriptor() {
        let md = MethodDescriptor::new("run", 2);
        let cloned = md.clone();
        assert_eq!(cloned.name(), "run");
        assert_eq!(cloned.parameter_count(), 2);
    }
}
