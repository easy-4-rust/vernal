//! BeanWrapperInfo — Spring 风格 Bean 包装器信息。
//!
//! 对应 Java 类：`org.springframework.beans.BeanWrapper` 的信息描述部分。
//!
//! 在 Spring 中，`BeanWrapper` 提供对 Bean 属性的统一访问接口。
//! `BeanWrapperInfo` 保存了 Bean 包装器的元数据信息，包括
//! 包装的 Bean 类型、可读/可写属性列表等。

use std::any::TypeId;
use std::collections::HashSet;
use std::sync::Mutex;

/// Bean 包装器信息，描述一个被包装 Bean 的元数据。
///
/// 对应 Spring `BeanWrapper` 的信息查询部分。
///
/// 保存 Bean 的类型标识、属性名称集合以及嵌套路径信息。
/// 用于在属性访问和类型转换过程中提供上下文。
#[derive(Debug)]
pub struct BeanWrapperInfo {
    /// 被包装 Bean 的 TypeId
    bean_type: TypeId,
    /// 被包装 Bean 的类型名称
    type_name: String,
    /// 可读属性名称集合
    readable_properties: Mutex<HashSet<String>>,
    /// 可写属性名称集合
    writable_properties: Mutex<HashSet<String>>,
    /// 嵌套路径前缀（如 "order.customer"）
    nested_path: String,
}

impl BeanWrapperInfo {
    /// 创建新的 BeanWrapperInfo。
    ///
    /// # 参数
    /// - `bean_type` — 被包装 Bean 的 TypeId
    /// - `type_name` — 类型名称
    pub fn new(bean_type: TypeId, type_name: String) -> Self {
        Self {
            bean_type,
            type_name,
            readable_properties: Mutex::new(HashSet::new()),
            writable_properties: Mutex::new(HashSet::new()),
            nested_path: String::new(),
        }
    }

    /// 获取被包装 Bean 的 TypeId。
    pub fn bean_type(&self) -> TypeId {
        self.bean_type
    }

    /// 获取类型名称。
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// 设置嵌套路径前缀。
    pub fn set_nested_path(&mut self, path: String) {
        self.nested_path = path;
    }

    /// 获取嵌套路径前缀。
    pub fn nested_path(&self) -> &str {
        &self.nested_path
    }

    /// 添加可读属性。
    pub fn add_readable_property(&self, name: &str) {
        self.readable_properties
            .lock()
            .unwrap()
            .insert(name.to_string());
    }

    /// 添加可写属性。
    pub fn add_writable_property(&self, name: &str) {
        self.writable_properties
            .lock()
            .unwrap()
            .insert(name.to_string());
    }

    /// 检查属性是否可读。
    pub fn is_readable(&self, name: &str) -> bool {
        self.readable_properties.lock().unwrap().contains(name)
    }

    /// 检查属性是否可写。
    pub fn is_writable(&self, name: &str) -> bool {
        self.writable_properties.lock().unwrap().contains(name)
    }

    /// 获取可读属性数量。
    pub fn readable_property_count(&self) -> usize {
        self.readable_properties.lock().unwrap().len()
    }

    /// 获取可写属性数量。
    pub fn writable_property_count(&self) -> usize {
        self.writable_properties.lock().unwrap().len()
    }

    /// 获取所有可读属性名称。
    pub fn readable_property_names(&self) -> Vec<String> {
        self.readable_properties
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    /// 获取所有可写属性名称。
    pub fn writable_property_names(&self) -> Vec<String> {
        self.writable_properties
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    /// 检查是否有嵌套路径。
    pub fn has_nested_path(&self) -> bool {
        !self.nested_path.is_empty()
    }

    /// 构建完整的属性路径（嵌套路径 + 属性名）。
    pub fn build_property_path(&self, property_name: &str) -> String {
        if self.nested_path.is_empty() {
            property_name.to_string()
        } else {
            format!("{}.{}", self.nested_path, property_name)
        }
    }
}

impl Clone for BeanWrapperInfo {
    fn clone(&self) -> Self {
        Self {
            bean_type: self.bean_type,
            type_name: self.type_name.clone(),
            readable_properties: Mutex::new(self.readable_properties.lock().unwrap().clone()),
            writable_properties: Mutex::new(self.writable_properties.lock().unwrap().clone()),
            nested_path: self.nested_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_info_has_correct_type() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        assert_eq!(info.bean_type(), TypeId::of::<String>());
        assert_eq!(info.type_name(), "String");
    }

    #[test]
    fn readable_and_writable_properties() {
        let info = BeanWrapperInfo::new(TypeId::of::<i32>(), "i32".to_string());
        info.add_readable_property("value");
        info.add_writable_property("value");

        assert!(info.is_readable("value"));
        assert!(info.is_writable("value"));
        assert!(!info.is_readable("missing"));
        assert_eq!(info.readable_property_count(), 1);
        assert_eq!(info.writable_property_count(), 1);
    }

    #[test]
    fn nested_path_and_property_path() {
        let mut info = BeanWrapperInfo::new(TypeId::of::<Vec<String>>(), "Vec<String>".to_string());
        assert!(!info.has_nested_path());

        info.set_nested_path("order.customer".to_string());
        assert!(info.has_nested_path());
        assert_eq!(info.build_property_path("name"), "order.customer.name");
    }

    #[test]
    fn clone_preserves_state() {
        let info = BeanWrapperInfo::new(TypeId::of::<bool>(), "bool".to_string());
        info.add_readable_property("flag");
        let cloned = info.clone();
        assert!(cloned.is_readable("flag"));
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn new_info_has_empty_nested_path() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        assert_eq!(info.nested_path(), "");
        assert!(!info.has_nested_path());
    }

    #[test]
    fn build_property_path_without_nested() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        assert_eq!(info.build_property_path("name"), "name");
    }

    #[test]
    fn build_property_path_with_nested() {
        let mut info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        info.set_nested_path("order.customer".to_string());
        assert_eq!(info.build_property_path("name"), "order.customer.name");
    }

    #[test]
    fn add_multiple_readable_properties() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        info.add_readable_property("name");
        info.add_readable_property("age");
        info.add_readable_property("email");
        assert_eq!(info.readable_property_count(), 3);
        assert!(info.is_readable("name"));
        assert!(info.is_readable("age"));
        assert!(info.is_readable("email"));
        assert!(!info.is_readable("missing"));
    }

    #[test]
    fn add_multiple_writable_properties() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        info.add_writable_property("name");
        info.add_writable_property("age");
        assert_eq!(info.writable_property_count(), 2);
        assert!(info.is_writable("name"));
        assert!(info.is_writable("age"));
    }

    #[test]
    fn readable_property_names() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        info.add_readable_property("a");
        info.add_readable_property("b");
        let names = info.readable_property_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a".to_string()));
        assert!(names.contains(&"b".to_string()));
    }

    #[test]
    fn writable_property_names() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        info.add_writable_property("x");
        info.add_writable_property("y");
        let names = info.writable_property_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"x".to_string()));
        assert!(names.contains(&"y".to_string()));
    }

    #[test]
    fn clone_preserves_writable_properties() {
        let info = BeanWrapperInfo::new(TypeId::of::<i32>(), "i32".to_string());
        info.add_writable_property("value");
        info.add_readable_property("value");
        let cloned = info.clone();
        assert!(cloned.is_writable("value"));
        assert!(cloned.is_readable("value"));
    }

    #[test]
    fn clone_preserves_nested_path() {
        let mut info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        info.set_nested_path("a.b.c".to_string());
        let cloned = info.clone();
        assert_eq!(cloned.nested_path(), "a.b.c");
        assert!(cloned.has_nested_path());
    }

    #[test]
    fn set_nested_path_overwrites() {
        let mut info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        info.set_nested_path("first".to_string());
        assert_eq!(info.nested_path(), "first");
        info.set_nested_path("second".to_string());
        assert_eq!(info.nested_path(), "second");
    }

    #[test]
    fn debug_format() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        let debug = format!("{:?}", info);
        assert!(debug.contains("BeanWrapperInfo"));
        assert!(debug.contains("String"));
    }

    #[test]
    fn is_readable_empty() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        assert!(!info.is_readable("anything"));
    }

    #[test]
    fn is_writable_empty() {
        let info = BeanWrapperInfo::new(TypeId::of::<String>(), "String".to_string());
        assert!(!info.is_writable("anything"));
    }
}
