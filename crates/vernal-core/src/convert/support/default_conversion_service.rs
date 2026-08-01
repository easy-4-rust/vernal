//! 默认转换服务。
//!
//! 对标 Spring `org.springframework.core.convert.support.DefaultConversionService`。

use std::any::TypeId;

use crate::convert::converter::{ConverterRegistry, TypeIdConverterRegistry};
use crate::convert::{ConversionError, Convertible};

/// 默认转换服务。
///
/// 对应 Java: org.springframework.core.convert.support.DefaultConversionService
///
/// 对标 Spring 语义：持有运行时转换器注册表，`addDefaultConverters()` 注册内置
/// 转换器（字符串 ↔ 标量）；类型化转换走 [`Convertible`] 静态分发，运行时
/// 注册的转换器走注册表查找。
pub struct DefaultConversionService {
    registry: TypeIdConverterRegistry,
}

impl DefaultConversionService {
    /// 创建并注册内置默认转换器。
    #[must_use]
    pub fn new() -> Self {
        let service = Self {
            registry: TypeIdConverterRegistry::new(),
        };
        service.add_default_converters();
        service
    }

    /// 注册内置转换器（对标 Spring `addDefaultConverters`）。
    ///
    /// 当前注册：`String → bool`（`true`/`1`/`yes`/`on` 等规范化为 `true`/`false`）。
    pub fn add_default_converters(&self) {
        self.registry.add_converter(
            TypeId::of::<String>(),
            TypeId::of::<bool>(),
            Box::new(|s: &str| {
                let value = bool::from_str_value(s)?;
                Ok(value.to_string())
            }),
        );
    }

    /// 类型化转换（对标 Spring `convert(Object, Class)`）。
    ///
    /// # 错误
    ///
    /// 转换失败时返回 [`ConversionError`]。
    pub fn convert<T: Convertible>(&self, value: &str) -> Result<T, ConversionError> {
        T::from_str_value(value)
    }

    /// 判定类型对是否可转换（对标 Spring `canConvert`）。
    #[must_use]
    pub fn can_convert<T: Convertible>(&self) -> bool {
        let _ = std::any::type_name::<T>();
        true
    }

    /// 按类型对做运行时注册表转换。
    ///
    /// # 错误
    ///
    /// 类型对未注册时返回 [`ConversionError`]。
    pub fn convert_erased(
        &self,
        value: &str,
        source_type: TypeId,
        target_type: TypeId,
    ) -> Result<String, ConversionError> {
        self.registry.convert(value, source_type, target_type)
    }

    /// 暴露运行时注册表（对标 Spring 的 `ConverterRegistry` 能力）。
    #[must_use]
    pub fn registry(&self) -> &TypeIdConverterRegistry {
        &self.registry
    }

    /// 获取全局共享实例（对标 Spring `getSharedInstance()`）。
    #[must_use]
    pub fn get_shared_instance() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<DefaultConversionService> =
            std::sync::OnceLock::new();
        INSTANCE.get_or_init(DefaultConversionService::new)
    }
}

impl Default for DefaultConversionService {
    fn default() -> Self {
        Self::new()
    }
}

impl super::ConfigurableConversionService for DefaultConversionService {
    fn convert_typed<T: Convertible>(&self, value: &str) -> Result<T, ConversionError> {
        self.convert(value)
    }

    fn convert_erased(
        &self,
        value: &str,
        source_type: TypeId,
        target_type: TypeId,
    ) -> Result<String, ConversionError> {
        self.convert_erased(value, source_type, target_type)
    }
}

impl ConverterRegistry for DefaultConversionService {
    fn add_converter(
        &self,
        source_type: TypeId,
        target_type: TypeId,
        converter: crate::convert::converter::ErasedConverter,
    ) {
        self.registry.add_converter(source_type, target_type, converter);
    }

    fn remove_convertible(&self, source_type: TypeId, target_type: TypeId) {
        self.registry.remove_convertible(source_type, target_type);
    }

    fn can_convert(&self, source_type: TypeId, target_type: TypeId) -> bool {
        self.registry.can_convert(source_type, target_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_conversion_works() {
        // A 类（合同对齐）：对标 Spring `convert("42", Integer.class)`
        let service = DefaultConversionService::new();
        assert_eq!(service.convert::<i32>("42").unwrap(), 42);
    }

    #[test]
    fn default_converters_are_registered() {
        // A 类（合同对齐）：对标 Spring `addDefaultConverters` 后的 canConvert
        let service = DefaultConversionService::new();
        assert!(service.can_convert::<bool>());
        let normalized = service
            .convert_erased("yes", TypeId::of::<String>(), TypeId::of::<bool>())
            .unwrap();
        assert_eq!(normalized, "true");
    }

    #[test]
    fn unregistered_pair_returns_error() {
        // C 类（错误路径）：对标 Spring `ConverterNotFoundException`
        let service = DefaultConversionService::new();
        let err = service
            .convert_erased("x", TypeId::of::<String>(), TypeId::of::<f64>())
            .unwrap_err();
        assert!(err.reason.contains("no converter registered"));
    }

    #[test]
    fn shared_instance_is_singleton() {
        // D 类（生命周期）：对标 Spring `getSharedInstance()` 单例
        let a = DefaultConversionService::get_shared_instance();
        let b = DefaultConversionService::get_shared_instance();
        assert!(std::ptr::eq(a, b));
    }

    #[test]
    fn registry_accessor_and_default_trait() {
        // D 类（重构安全）：registry() 暴露与 Default 构造等价
        let service = DefaultConversionService::default();
        assert!(service.registry().can_convert(TypeId::of::<String>(), TypeId::of::<bool>()));
    }

    #[test]
    fn configurable_service_delegates() {
        // D 类（重构安全）：ConfigurableConversionService 委托路径
        fn exercise<T: crate::convert::support::ConfigurableConversionService>(service: &T) {
            assert_eq!(service.convert_typed::<i32>("5").unwrap(), 5);
            assert_eq!(
                service
                    .convert_erased("1", TypeId::of::<String>(), TypeId::of::<bool>())
                    .unwrap(),
                "true"
            );
        }
        let service = DefaultConversionService::new();
        exercise(&service);
    }

    #[test]
    fn registry_can_be_extended_at_runtime() {
        // B 类（边界行为）：对标 Spring 运行时 addConverter
        let service = DefaultConversionService::new();
        service.add_converter(
            TypeId::of::<String>(),
            TypeId::of::<String>(),
            Box::new(|s: &str| Ok(format!("<{s}>"))),
        );
        assert_eq!(
            service
                .convert_erased("x", TypeId::of::<String>(), TypeId::of::<String>())
                .unwrap(),
            "<x>"
        );
    }
}
