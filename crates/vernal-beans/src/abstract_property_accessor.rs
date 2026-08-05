//! AbstractPropertyAccessor — 属性访问器的抽象基类。
//!
//! 对应 Java 类：`org.springframework.beans.AbstractPropertyAccessor`。
//!
//! 提供 `PropertyAccessor` trait 的基础实现，使用 `HashMap` 存储属性。
//! 这是一个可复用的基类，其他属性访问器可以组合使用它。
//!
//! # 设计原则（Rust 原生）
//!
//! Java 的 `AbstractPropertyAccessor` 是抽象类，子类继承其方法。
//! Rust 中没有类继承，所以使用组合模式：
//! - `AbstractPropertyAccessor` 是一个独立的结构体
//! - 通过 `Deref` 或直接方法调用来复用其功能
//!
//! # 线程安全
//!
//! 内部使用 `RwLock` 保证并发安全。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::property_accessor::PropertyAccessor;

/// 属性访问器的抽象基类。
///
/// 对应 Spring 的 `AbstractPropertyAccessor`。
///
/// 提供基于 `HashMap` 的属性存储，支持：
/// - 属性的读写
/// - 属性类型注册
/// - 属性名称列举
///
/// # 设计
///
/// 使用 `RwLock` 实现内部可变性，保证线程安全。
/// 读操作获取共享读锁，写操作获取独占写锁。
///
/// # 用法
///
/// ```rust,no_run
/// use std::any::TypeId;
/// use std::sync::Arc;
/// use vernal_beans::abstract_property_accessor::AbstractPropertyAccessor;
/// use vernal_beans::property_accessor::PropertyAccessor;
///
/// let accessor = AbstractPropertyAccessor::new();
/// accessor.register_property("name", TypeId::of::<String>());
/// accessor.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
///
/// let value = accessor.get_property_value("name").unwrap();
/// let name = value.downcast_ref::<String>().unwrap();
/// assert_eq!(name, "Alice");
/// ```
pub struct AbstractPropertyAccessor {
    /// 属性值存储。
    properties: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 属性类型映射。
    property_types: RwLock<HashMap<String, TypeId>>,
    /// 可读属性集合。
    readable: RwLock<HashMap<String, bool>>,
    /// 可写属性集合。
    writable: RwLock<HashMap<String, bool>>,
}

impl AbstractPropertyAccessor {
    /// 创建空的 AbstractPropertyAccessor。
    pub fn new() -> Self {
        Self {
            properties: RwLock::new(HashMap::new()),
            property_types: RwLock::new(HashMap::new()),
            readable: RwLock::new(HashMap::new()),
            writable: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个属性。
    ///
    /// # 参数
    ///
    /// - `name` — 属性名称
    /// - `type_id` — 属性类型 ID
    pub fn register_property(&self, name: impl Into<String>, type_id: TypeId) {
        let name = name.into();
        self.property_types
            .write()
            .unwrap()
            .insert(name.clone(), type_id);
        self.readable.write().unwrap().insert(name.clone(), true);
        self.writable.write().unwrap().insert(name, true);
    }

    /// 注册一个只读属性。
    pub fn register_readonly_property(&self, name: impl Into<String>, type_id: TypeId) {
        let name = name.into();
        self.property_types
            .write()
            .unwrap()
            .insert(name.clone(), type_id);
        self.readable.write().unwrap().insert(name.clone(), true);
        self.writable.write().unwrap().insert(name, false);
    }

    /// 检查属性是否已注册。
    pub fn has_property(&self, name: &str) -> bool {
        self.property_types.read().unwrap().contains_key(name)
    }

    /// 获取已注册属性的数量。
    pub fn property_count(&self) -> usize {
        self.property_types.read().unwrap().len()
    }
}

impl Default for AbstractPropertyAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyAccessor for AbstractPropertyAccessor {
    fn get_property_value(
        &self,
        name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查属性是否已注册
        if !self.has_property(name) {
            return Err(format!("属性不存在: '{}'", name).into());
        }

        // 检查是否可读
        let readable = self.readable.read().unwrap();
        if !readable.get(name).copied().unwrap_or(false) {
            return Err(format!("属性不可读: '{}'", name).into());
        }
        drop(readable);

        // 获取属性值
        let props = self.properties.read().unwrap();
        props
            .get(name)
            .cloned()
            .ok_or_else(|| format!("属性值未设置: '{}'", name).into())
    }

    fn set_property_value(
        &self,
        name: &str,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 检查属性是否已注册
        if !self.has_property(name) {
            return Err(format!("属性不存在: '{}'", name).into());
        }

        // 检查是否可写
        let writable = self.writable.read().unwrap();
        if !writable.get(name).copied().unwrap_or(false) {
            return Err(format!("属性不可写: '{}'", name).into());
        }
        drop(writable);

        // 设置属性值
        let mut props = self.properties.write().unwrap();
        props.insert(name.to_string(), value);
        Ok(())
    }

    fn get_property_type(&self, name: &str) -> Option<TypeId> {
        self.property_types.read().unwrap().get(name).copied()
    }

    fn is_readable(&self, name: &str) -> bool {
        self.readable
            .read()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or(false)
    }

    fn is_writable(&self, name: &str) -> bool {
        self.writable
            .read()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or(false)
    }

    fn get_property_names(&self) -> Vec<String> {
        self.property_types
            .read()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_property_access() {
        let accessor = AbstractPropertyAccessor::new();
        accessor.register_property("name", TypeId::of::<String>());

        accessor
            .set_property_value("name", Arc::new("Alice".to_string()))
            .unwrap();
        let value = accessor.get_property_value("name").unwrap();
        assert_eq!(*value.downcast_ref::<String>().unwrap(), "Alice");
    }

    #[test]
    fn test_property_type() {
        let accessor = AbstractPropertyAccessor::new();
        accessor.register_property("count", TypeId::of::<i32>());
        assert_eq!(
            accessor.get_property_type("count"),
            Some(TypeId::of::<i32>())
        );
    }

    #[test]
    fn test_readonly() {
        let accessor = AbstractPropertyAccessor::new();
        accessor.register_readonly_property("ro", TypeId::of::<i32>());
        assert!(accessor.is_readable("ro"));
        assert!(!accessor.is_writable("ro"));
        assert!(accessor.set_property_value("ro", Arc::new(42i32)).is_err());
    }

    #[test]
    fn test_nonexistent_property() {
        let accessor = AbstractPropertyAccessor::new();
        assert!(accessor.get_property_value("missing").is_err());
        assert!(!accessor.has_property("missing"));
    }

    #[test]
    fn test_property_count() {
        let accessor = AbstractPropertyAccessor::new();
        assert_eq!(accessor.property_count(), 0);
        accessor.register_property("a", TypeId::of::<i32>());
        accessor.register_property("b", TypeId::of::<String>());
        assert_eq!(accessor.property_count(), 2);
    }
}
