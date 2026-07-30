//! PropertyAccessor — Spring 风格的属性访问器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.PropertyAccessor`。
//!
//! 定义了对 Bean 属性进行读写操作的统一接口。所有属性访问器
//! （包括 BeanWrapper）都实现了此 trait。
//!
//! # 设计原则（Rust 原生，非 Java 反射）
//!
//! - Java 使用反射访问字段 → Rust 使用 `Arc<dyn Any + Send + Sync>` + downcast
//! - Java `PropertyDescriptor` → Rust 使用 `TypeId` 进行类型标识
//! - 属性值通过 `Arc<dyn Any + Send + Sync>` 实现类型擦除与共享
//!
//! # 属性访问语义
//!
//! - `get_property_value` — 获取属性值，返回类型擦除的 `Arc<dyn Any>`
//! - `set_property_value` — 设置属性值
//! - `get_property_type` — 获取属性的 `TypeId`
//! - `is_readable` / `is_writable` — 查询属性可读/可写性
//! - `get_property_names` — 列出所有已知属性名

use std::any::{Any, TypeId};
use std::sync::Arc;

/// Spring 风格的属性访问器 trait。
///
/// 对应 Spring 的 `PropertyAccessor` 接口。
///
/// 定义了对 Bean 属性进行读写操作的统一接口。
/// 所有属性访问器（包括 `BeanWrapper`）都实现此 trait。
///
/// ## 与 Java PropertyAccessor 的映射
///
/// | Java | Rust |
/// |------|------|
/// | `getPropertyValue(String)` | `get_property_value(&str)` |
/// | `setPropertyValue(String, Object)` | `set_property_value(&str, Arc<dyn Any>)` |
/// | `getPropertyType(String)` | `get_property_type(&str) -> Option<TypeId>` |
/// | `isReadableProperty(String)` | `is_readable(&str) -> bool` |
/// | `isWritableProperty(String)` | `is_writable(&str) -> bool` |
///
/// ## 错误处理
///
/// Java 中属性访问失败会抛出 `BeansException`。
/// Rust 中返回 `Result`，错误类型为 `Box<dyn Error + Send + Sync>`。
pub trait PropertyAccessor: Send + Sync {
    /// 获取指定属性的值。
    ///
    /// 对应 Spring 的 `Object getPropertyValue(String propertyName)`。
    ///
    /// # 参数
    ///
    /// - `name` — 属性名称，支持嵌套路径（如 `"address.city"`）
    ///
    /// # 返回
    ///
    /// - `Ok(value)` — 属性值（类型擦除）
    /// - `Err` — 属性不存在、不可读或访问失败
    fn get_property_value(
        &self,
        name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;

    /// 设置指定属性的值。
    ///
    /// 对应 Spring 的 `void setPropertyValue(String propertyName, Object value)`。
    ///
    /// # 参数
    ///
    /// - `name` — 属性名称，支持嵌套路径（如 `"address.city"`）
    /// - `value` — 要设置的值（类型擦除）
    ///
    /// # 返回
    ///
    /// - `Ok(())` — 设置成功
    /// - `Err` — 属性不存在、不可写或类型不匹配
    fn set_property_value(
        &self,
        name: &str,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 获取指定属性的类型。
    ///
    /// 对应 Spring 的 `Class<?> getPropertyType(String propertyName)`。
    ///
    /// 在 Rust 中使用 `TypeId` 替代 Java 的 `Class<?>`。
    ///
    /// # 返回
    ///
    /// - `Some(TypeId)` — 属性类型
    /// - `None` — 属性不存在
    fn get_property_type(&self, name: &str) -> Option<TypeId>;

    /// 检查指定属性是否可读。
    ///
    /// 对应 Spring 的 `boolean isReadableProperty(String propertyName)`。
    fn is_readable(&self, name: &str) -> bool;

    /// 检查指定属性是否可写。
    ///
    /// 对应 Spring 的 `boolean isWritableProperty(String propertyName)`。
    fn is_writable(&self, name: &str) -> bool;

    /// 获取所有已注册的属性名称。
    ///
    /// 对应 Spring 的 `String[] getPropertyDescriptors()` 的简化版本。
    ///
    /// 返回当前访问器已知的所有属性名称列表。
    fn get_property_names(&self) -> Vec<String>;
}
