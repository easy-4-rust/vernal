//! BeanWrapper — Spring 风格的 Bean 包装器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.BeanWrapper`。
//!
//! `BeanWrapper` 扩展了 `PropertyAccessor`，提供了对 Bean 实例的
//! 包装和批量属性设置能力。它是 Spring 属性绑定的核心接口。
//!
//! # 设计原则（Rust 原生）
//!
//! - Java BeanWrapper 使用反射操作字段 → Rust 使用 `Arc<dyn Any>` + downcast
//! - Java `getWrappedInstance()` 返回 Object → Rust 返回 `&dyn Any`
//! - Java `getWrappedClass()` 返回 `Class<?>` → Rust 返回 `TypeId`
//! - 批量属性设置通过遍历 HashMap 实现
//!
//! # 典型用法
//!
//! ```rust,no_run
//! use std::any::Any;
//! use std::collections::HashMap;
//! use std::sync::Arc;
//! use vernal_beans::bean_wrapper::BeanWrapper;
//!
//! // BeanWrapper 通常通过 BeanWrapperImpl 创建
//! // let wrapper = BeanWrapperImpl::new(my_instance);
//! // wrapper.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
//! // let name = wrapper.get_property_value("name").unwrap();
//! // let name = name.downcast_ref::<String>().unwrap();
//! // assert_eq!(name, "Alice");
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::property_accessor::PropertyAccessor;

/// Spring 风格的 Bean 包装器 trait。
///
/// 对应 Spring 的 `BeanWrapper` 接口。
///
/// 扩展 `PropertyAccessor`，增加了：
/// - 获取被包装的实例引用
/// - 获取被包装实例的类型
/// - 批量设置属性值
///
/// ## 与 Java BeanWrapper 的映射
///
/// | Java | Rust |
/// |------|------|
/// | `getWrappedInstance()` | `get_wrapped_instance()` |
/// | `getWrappedClass()` | `get_wrapped_class()` |
/// | `setPropertyValues(PropertyValues)` | `set_property_values(&HashMap)` |
///
/// ## 类型安全
///
/// Java 的 BeanWrapper 在运行时通过反射做类型检查。
/// Rust 中使用 `TypeId` 进行编译后类型标识，通过 `downcast_ref`
/// 在运行时安全地还原具体类型。
pub trait BeanWrapper: PropertyAccessor {
    /// 获取被包装的 Bean 实例引用。
    ///
    /// 对应 Spring 的 `Object getWrappedInstance()`。
    ///
    /// 返回一个 `&dyn Any` 引用，可以通过 `downcast_ref::<T>()`
    /// 尝试还原为具体类型。
    fn get_wrapped_instance(&self) -> &dyn Any;

    /// 获取被包装 Bean 的类型标识。
    ///
    /// 对应 Spring 的 `Class<?> getWrappedClass()`。
    ///
    /// 在 Rust 中使用 `TypeId` 替代 Java 的 `Class<?>`。
    fn get_wrapped_class(&self) -> TypeId;

    /// 批量设置属性值。
    ///
    /// 对应 Spring 的 `void setPropertyValues(PropertyValues pvs)`。
    ///
    /// 遍历 HashMap 中的所有键值对，逐个调用 `set_property_value`。
    /// 如果某个属性设置失败，返回错误（不支持部分成功语义）。
    ///
    /// # 参数
    ///
    /// - `values` — 属性名到属性值的映射
    ///
    /// # 返回
    ///
    /// - `Ok(())` — 所有属性设置成功
    /// - `Err` — 某个属性设置失败（包含属性名和原因）
    fn set_property_values(
        &self,
        values: &HashMap<String, Arc<dyn Any + Send + Sync>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (name, value) in values {
            self.set_property_value(name, Arc::clone(value))?;
        }
        Ok(())
    }

    /// 检查是否为嵌套属性路径。
    ///
    /// 辅助方法：判断属性名是否包含 `'.'` 分隔符。
    ///
    /// # 示例
    ///
    /// - `"name"` → `false`
    /// - `"address.city"` → `true`
    fn is_nested_property(&self, name: &str) -> bool {
        name.contains('.')
    }
}
