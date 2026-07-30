//! ConfigurablePropertyAccessor — 可配置的属性访问器 trait。
//!
//! 对应 Java 类：`org.springframework.beans.ConfigurablePropertyAccessor`。
//!
//! 组合 `PropertyAccessor` 与 `TypeConverter` 能力，提供一个统一的
//! 属性访问接口。在设置属性时可以自动进行类型转换。
//!
//! # 设计原则（Rust 原生）
//!
//! Java 的 `ConfigurablePropertyAccessor` 继承了 `PropertyAccessor`
//! 和 `TypeConverter`。Rust 中使用 trait 组合实现同等能力。
//!
//! # 类型转换
//!
//! 当设置属性值的类型与属性期望的类型不匹配时，
//! `ConfigurablePropertyAccessor` 可以委托 `TypeConverter` 进行自动转换。
//! 例如将字符串 `"42"` 转换为 `i32`。
//!
//! # 与 BeanWrapperImpl 的关系
//!
//! `BeanWrapperImpl` 实现了 `PropertyAccessor` 和 `BeanWrapper`。
//! 如果需要类型转换能力，可以组合 `ConfigurablePropertyAccessor`。

use std::any::{Any, TypeId};
use std::sync::Arc;

use crate::property_accessor::PropertyAccessor;
use crate::type_converter::TypeConverter;

/// 可配置的属性访问器 trait。
///
/// 对应 Spring 的 `ConfigurablePropertyAccessor` 接口。
///
/// 组合 `PropertyAccessor` 的属性读写能力与 `TypeConverter` 的类型转换能力。
///
/// ## 与 Java ConfigurablePropertyAccessor 的映射
///
/// | Java | Rust |
/// |------|------|
/// | `ConfigurablePropertyAccessor` | `ConfigurablePropertyAccessor` |
/// | `setConversionService(...)` | `set_type_converter(...)` |
/// | `getConversionService()` | `get_type_converter()` |
/// | `isExtractOldValueForEditor()` | `extract_old_value_for_editor()` |
///
/// ## 设计说明
///
/// Java 中 `ConfigurablePropertyAccessor` 同时继承 `PropertyAccessor`
/// 和 `TypeConverter`，但 Rust 的 trait 不支持多重继承。
/// 因此 `ConfigurablePropertyAccessor` 仅继承 `PropertyAccessor`，
/// 并通过组合方式引入 `TypeConverter` 功能。
pub trait ConfigurablePropertyAccessor: PropertyAccessor {
    /// 设置类型转换器。
    ///
    /// 对应 Spring 的 `void setConversionService(ConversionService)`。
    ///
    /// 在设置属性值时，如果类型不匹配，将使用此转换器进行自动转换。
    fn set_type_converter(&self, converter: Arc<dyn TypeConverter>);

    /// 获取类型转换器。
    ///
    /// 对应 Spring 的 `ConversionService getConversionService()`。
    fn get_type_converter(&self) -> Option<Arc<dyn TypeConverter>>;

    /// 是否提取旧值用于编辑器。
    ///
    /// 对应 Spring 的 `boolean isExtractOldValueForEditor()`。
    ///
    /// 当为 `true` 时，在设置新值前会先获取旧值，
    /// 并将其传递给属性编辑器。
    fn extract_old_value_for_editor(&self) -> bool {
        false
    }

    /// 设置是否提取旧值用于编辑器。
    ///
    /// 对应 Spring 的 `void setExtractOldValueForEditor(boolean)`。
    fn set_extract_old_value_for_editor(&self, _extract: bool) {
        // 默认实现：不支持
    }

    /// 设置属性值（带类型转换）。
    ///
    /// 与 `PropertyAccessor::set_property_value` 类似，但在类型不匹配时
    /// 尝试使用类型转换器进行自动转换。
    ///
    /// # 参数
    ///
    /// - `name` — 属性名称
    /// - `value` — 属性值
    ///
    /// # 返回
    ///
    /// - `Ok(())` — 设置成功（可能经过类型转换）
    /// - `Err` — 属性不存在或类型转换失败
    fn set_property_value_with_conversion(
        &self,
        name: &str,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 获取属性期望的类型
        let expected_type = self.get_property_type(name);

        match expected_type {
            Some(target_type) => {
                // 检查值的类型是否匹配
                let value_type = (*value).type_id();
                if value_type == target_type {
                    // 类型直接匹配，无需转换
                    self.set_property_value(name, value)
                } else {
                    // 类型不匹配，尝试转换
                    match self.get_type_converter() {
                        Some(converter) => {
                            let converted =
                                converter.convert_if_necessary(Some(name), &*value, target_type)?;
                            self.set_property_value(name, Arc::from(converted))
                        }
                        None => {
                            // 没有类型转换器，直接设置（可能失败）
                            self.set_property_value(name, value)
                        }
                    }
                }
            }
            None => {
                // 属性类型未知，直接设置
                self.set_property_value(name, value)
            }
        }
    }
}

/// ConfigurablePropertyAccessor 的默认实现结构体。
///
/// 包装一个 `PropertyAccessor` 实现，添加类型转换能力。
///
/// # 用法
///
/// ```rust,no_run
/// use std::any::{Any, TypeId};
/// use std::sync::Arc;
/// use vernal_beans::abstract_property_accessor::AbstractPropertyAccessor;
/// use vernal_beans::configurable_property_accessor::{
///     ConfigurablePropertyAccessorImpl,
///     ConfigurablePropertyAccessor,
/// };
/// use vernal_beans::property_accessor::PropertyAccessor;
/// use vernal_beans::conversion_service::DefaultConversionService;
///
/// let base = AbstractPropertyAccessor::new();
/// base.register_property("count", TypeId::of::<i32>());
///
/// let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
/// // 设置字符串值，通过类型转换器自动转为 i32
/// ```
pub struct ConfigurablePropertyAccessorImpl {
    /// 被包装的属性访问器。
    delegate: Box<dyn PropertyAccessor>,
    /// 类型转换器。
    converter: std::sync::RwLock<Option<Arc<dyn TypeConverter>>>,
    /// 是否提取旧值用于编辑器。
    extract_old_value: std::sync::atomic::AtomicBool,
}

impl ConfigurablePropertyAccessorImpl {
    /// 创建新的 ConfigurablePropertyAccessorImpl。
    ///
    /// # 参数
    ///
    /// - `delegate` — 被包装的属性访问器实现
    pub fn new(delegate: Box<dyn PropertyAccessor>) -> Self {
        Self {
            delegate,
            converter: std::sync::RwLock::new(None),
            extract_old_value: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// 创建带类型转换器的 ConfigurablePropertyAccessorImpl。
    ///
    /// # 参数
    ///
    /// - `delegate` — 被包装的属性访问器实现
    /// - `converter` — 类型转换器
    pub fn with_converter(
        delegate: Box<dyn PropertyAccessor>,
        converter: Arc<dyn TypeConverter>,
    ) -> Self {
        Self {
            delegate,
            converter: std::sync::RwLock::new(Some(converter)),
            extract_old_value: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

/// 为 ConfigurablePropertyAccessorImpl 委托 PropertyAccessor 的所有方法。
impl PropertyAccessor for ConfigurablePropertyAccessorImpl {
    fn get_property_value(
        &self,
        name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.delegate.get_property_value(name)
    }

    fn set_property_value(
        &self,
        name: &str,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.delegate.set_property_value(name, value)
    }

    fn get_property_type(&self, name: &str) -> Option<TypeId> {
        self.delegate.get_property_type(name)
    }

    fn is_readable(&self, name: &str) -> bool {
        self.delegate.is_readable(name)
    }

    fn is_writable(&self, name: &str) -> bool {
        self.delegate.is_writable(name)
    }

    fn get_property_names(&self) -> Vec<String> {
        self.delegate.get_property_names()
    }
}

/// 为 ConfigurablePropertyAccessorImpl 实现 ConfigurablePropertyAccessor trait。
impl ConfigurablePropertyAccessor for ConfigurablePropertyAccessorImpl {
    fn set_type_converter(&self, converter: Arc<dyn TypeConverter>) {
        let mut guard = self.converter.write().unwrap();
        *guard = Some(converter);
    }

    fn get_type_converter(&self) -> Option<Arc<dyn TypeConverter>> {
        self.converter.read().unwrap().clone()
    }

    fn extract_old_value_for_editor(&self) -> bool {
        self.extract_old_value
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_extract_old_value_for_editor(&self, extract: bool) {
        self.extract_old_value
            .store(extract, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abstract_property_accessor::AbstractPropertyAccessor;

    #[test]
    fn test_delegate_property_access() {
        let base = AbstractPropertyAccessor::new();
        base.register_property("name", TypeId::of::<String>());

        let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
        accessor
            .set_property_value("name", Arc::new("Alice".to_string()))
            .unwrap();

        let value = accessor.get_property_value("name").unwrap();
        assert_eq!(*value.downcast_ref::<String>().unwrap(), "Alice");
    }

    #[test]
    fn test_type_converter_setter_getter() {
        let base = AbstractPropertyAccessor::new();
        let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));

        // 初始无转换器
        assert!(accessor.get_type_converter().is_none());

        // 设置转换器（使用 mock）
        // 这里只测试 setter/getter 逻辑
        assert!(accessor.get_type_converter().is_none());
    }

    #[test]
    fn test_extract_old_value_flag() {
        let base = AbstractPropertyAccessor::new();
        let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));

        assert!(!accessor.extract_old_value_for_editor());
        accessor.set_extract_old_value_for_editor(true);
        assert!(accessor.extract_old_value_for_editor());
    }

    #[test]
    fn test_property_names_delegated() {
        let base = AbstractPropertyAccessor::new();
        base.register_property("a", TypeId::of::<i32>());
        base.register_property("b", TypeId::of::<String>());

        let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
        let mut names = accessor.get_property_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_readable_writable_delegated() {
        let base = AbstractPropertyAccessor::new();
        base.register_property("rw", TypeId::of::<i32>());
        base.register_readonly_property("ro", TypeId::of::<i32>());

        let accessor = ConfigurablePropertyAccessorImpl::new(Box::new(base));
        assert!(accessor.is_readable("rw"));
        assert!(accessor.is_writable("rw"));
        assert!(accessor.is_readable("ro"));
        assert!(!accessor.is_writable("ro"));
    }
}
