//! 通用转换服务。
//!
//! 对标 Spring `org.springframework.core.convert.support.GenericConversionService`。

use std::any::TypeId;

use crate::convert::converter::{ConverterRegistry, TypeIdConverterRegistry};
use crate::convert::{ConversionError, Convertible};

/// 通用转换服务。
///
/// 对应 Java: org.springframework.core.convert.support.GenericConversionService
///
/// 对标 Spring 语义：`DefaultConversionService` 的父类，提供注册表驱动的
/// 运行时转换（含 `Convertible` 静态分发兜底）。
pub struct GenericConversionService {
    registry: TypeIdConverterRegistry,
}

impl GenericConversionService {
    /// 创建空转换服务（不注册内置转换器）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: TypeIdConverterRegistry::new(),
        }
    }

    /// 类型化转换（对标 Spring `convert(Object, Class)`）。
    ///
    /// # 错误
    ///
    /// 转换失败时返回 [`ConversionError`]。
    pub fn convert<T: Convertible>(&self, value: &str) -> Result<T, ConversionError> {
        T::from_str_value(value)
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

    /// 暴露运行时注册表。
    #[must_use]
    pub fn registry(&self) -> &TypeIdConverterRegistry {
        &self.registry
    }
}

impl Default for GenericConversionService {
    fn default() -> Self {
        Self::new()
    }
}

impl super::ConfigurableConversionService for GenericConversionService {
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

impl ConverterRegistry for GenericConversionService {
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
    use std::any::TypeId;

    #[test]
    fn typed_conversion_works_without_defaults() {
        // A 类（合同对齐）：GenericConversionService 可独立使用
        let service = GenericConversionService::new();
        assert!(service.convert::<bool>("on").unwrap());
    }

    #[test]
    fn empty_registry_reports_not_registered() {
        // B 类（边界行为）：未注册类型对不可转换
        let service = GenericConversionService::new();
        assert!(!service.registry().can_convert(
            TypeId::of::<String>(),
            TypeId::of::<bool>(),
        ));
    }

    #[test]
    fn default_trait_creates_empty_service() {
        // D 类（重构安全）：Default 构造与 new 等价
        let service = GenericConversionService::default();
        assert!(!service.registry().can_convert(TypeId::of::<String>(), TypeId::of::<bool>()));
    }

    #[test]
    fn configurable_service_delegates() {
        // D 类（重构安全）：ConfigurableConversionService 委托路径
        fn exercise<T: crate::convert::support::ConfigurableConversionService>(service: &T) {
            assert_eq!(service.convert_typed::<u8>("9").unwrap(), 9);
            assert_eq!(
                service
                    .convert_erased("ok", TypeId::of::<String>(), TypeId::of::<String>())
                    .unwrap(),
                "OK"
            );
        }
        let service = GenericConversionService::new();
        service.add_converter(
            TypeId::of::<String>(),
            TypeId::of::<String>(),
            Box::new(|s: &str| Ok(s.to_uppercase())),
        );
        exercise(&service);
    }

    #[test]
    fn runtime_registration_enables_conversion() {
        // D 类（生命周期）：注册后即可转换
        let service = GenericConversionService::new();
        service.add_converter(
            TypeId::of::<String>(),
            TypeId::of::<String>(),
            Box::new(|s: &str| Ok(s.to_uppercase())),
        );
        assert_eq!(
            service
                .convert_erased("hi", TypeId::of::<String>(), TypeId::of::<String>())
                .unwrap(),
            "HI"
        );
    }
}
