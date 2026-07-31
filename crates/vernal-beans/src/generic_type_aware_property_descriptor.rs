//! GenericTypeAwarePropertyDescriptor — 对应 Spring `org.springframework.beans.GenericTypeAwarePropertyDescriptor`。
//!
//! 感知泛型类型的属性描述符。
//!
//! 在 Java Spring 中，`GenericTypeAwarePropertyDescriptor` 扩展了标准的 `PropertyDescriptor`，
//! 能够解析泛型类型信息。在 Rust 中，由于编译时类型擦除和生命周期限制，
//! 我们通过 `TypeId` 和可选的泛型类型名来实现类似功能。
//!
//! # Spring 对标
//!
//! 对应 Java 类：`org.springframework.beans.GenericTypeAwarePropertyDescriptor`。
//! 此类用于在 Bean 属性绑定时获取精确的泛型类型信息。

use std::any::TypeId;

/// 感知泛型类型的属性描述符。
///
/// 对应 Java 类：`org.springframework.beans.GenericTypeAwarePropertyDescriptor`。
///
/// 在标准属性描述符的基础上，增加了泛型类型感知能力。
/// 用于 Bean 属性绑定、类型转换等场景。
#[derive(Debug, Clone)]
pub struct GenericTypeAwarePropertyDescriptor {
    /// 属性名称
    name: String,
    /// 属性的原始类型 ID
    property_type: TypeId,
    /// 属性的原始类型名
    property_type_name: String,
    /// 泛型类型名（如 `Vec<String>` 中的 `String`）
    generic_type_name: Option<String>,
    /// 泛型参数的类型 ID
    generic_type_id: Option<TypeId>,
    /// 是否可读
    readable: bool,
    /// 是否可写
    writable: bool,
    /// 属性是否可选（对应 Rust 的 `Option<T>`）
    optional: bool,
}

impl GenericTypeAwarePropertyDescriptor {
    /// 创建一个新的 GenericTypeAwarePropertyDescriptor。
    ///
    /// # Arguments
    ///
    /// * `name` - 属性名称
    /// * `property_type` - 属性的类型 ID
    /// * `property_type_name` - 属性的类型名
    pub fn new(
        name: impl Into<String>,
        property_type: TypeId,
        property_type_name: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            property_type,
            property_type_name: property_type_name.into(),
            generic_type_name: None,
            generic_type_id: None,
            readable: true,
            writable: true,
            optional: false,
        }
    }

    /// 设置泛型类型信息。
    ///
    /// # Arguments
    ///
    /// * `generic_type_name` - 泛型类型名
    /// * `generic_type_id` - 泛型类型 ID
    pub fn with_generic_type(
        mut self,
        generic_type_name: impl Into<String>,
        generic_type_id: TypeId,
    ) -> Self {
        self.generic_type_name = Some(generic_type_name.into());
        self.generic_type_id = Some(generic_type_id);
        self
    }

    /// 设置是否可读。
    pub fn readable(mut self, readable: bool) -> Self {
        self.readable = readable;
        self
    }

    /// 设置是否可写。
    pub fn writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    /// 设置是否可选。
    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    /// 获取属性名称。
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取属性的类型 ID。
    pub fn property_type(&self) -> TypeId {
        self.property_type
    }

    /// 获取属性的类型名。
    pub fn property_type_name(&self) -> &str {
        &self.property_type_name
    }

    /// 获取泛型类型名。
    pub fn generic_type_name(&self) -> Option<&str> {
        self.generic_type_name.as_deref()
    }

    /// 获取泛型类型 ID。
    pub fn generic_type_id(&self) -> Option<TypeId> {
        self.generic_type_id
    }

    /// 判断属性是否可读。
    pub fn is_readable(&self) -> bool {
        self.readable
    }

    /// 判断属性是否可写。
    pub fn is_writable(&self) -> bool {
        self.writable
    }

    /// 判断属性是否可选。
    pub fn is_optional(&self) -> bool {
        self.optional
    }

    /// 判断属性是否有泛型类型信息。
    pub fn has_generic_type(&self) -> bool {
        self.generic_type_name.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let desc = GenericTypeAwarePropertyDescriptor::new("name", TypeId::of::<String>(), "String");
        assert_eq!(desc.name(), "name");
        assert_eq!(desc.property_type_name(), "String");
        assert_eq!(desc.property_type(), TypeId::of::<String>());
        assert!(desc.is_readable());
        assert!(desc.is_writable());
        assert!(!desc.is_optional());
        assert!(!desc.has_generic_type());
    }

    #[test]
    fn test_with_generic_type() {
        let desc = GenericTypeAwarePropertyDescriptor::new(
            "items",
            TypeId::of::<Vec<String>>(),
            "Vec",
        )
        .with_generic_type("String", TypeId::of::<String>());

        assert_eq!(desc.name(), "items");
        assert!(desc.has_generic_type());
        assert_eq!(desc.generic_type_name(), Some("String"));
        assert_eq!(desc.generic_type_id(), Some(TypeId::of::<String>()));
    }

    #[test]
    fn test_builder_pattern() {
        let desc = GenericTypeAwarePropertyDescriptor::new(
            "count",
            TypeId::of::<Option<i32>>(),
            "Option",
        )
        .readable(true)
        .writable(false)
        .optional(true)
        .with_generic_type("i32", TypeId::of::<i32>());

        assert!(desc.is_readable());
        assert!(!desc.is_writable());
        assert!(desc.is_optional());
        assert_eq!(desc.generic_type_name(), Some("i32"));
    }

    #[test]
    fn test_readonly_property() {
        let desc = GenericTypeAwarePropertyDescriptor::new(
            "id",
            TypeId::of::<u64>(),
            "u64",
        )
        .writable(false);

        assert!(desc.is_readable());
        assert!(!desc.is_writable());
    }

    #[test]
    fn test_clone() {
        let desc = GenericTypeAwarePropertyDescriptor::new("test", TypeId::of::<bool>(), "bool");
        let cloned = desc.clone();
        assert_eq!(cloned.name(), "test");
        assert_eq!(cloned.property_type_name(), "bool");
    }
}
