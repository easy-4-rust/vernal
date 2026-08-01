//! 可配置转换服务契约。
//!
//! 对标 Spring `org.springframework.core.convert.support.ConfigurableConversionService`。

use std::any::TypeId;

use crate::convert::converter::ConverterRegistry;
use crate::convert::{ConversionError, Convertible};

/// 可配置转换服务契约。
///
/// 对应 Java: org.springframework.core.convert.support.ConfigurableConversionService
///
/// Spring 语义：`ConversionService + ConverterRegistry` 的组合接口——既能按
/// 类型对查询/转换，也能在运行时增删转换器。
pub trait ConfigurableConversionService: ConverterRegistry {
    /// 类型化转换（对标 Spring `ConversionService.convert`）。
    ///
    /// # 错误
    ///
    /// 转换失败时返回 [`ConversionError`]。
    fn convert_typed<T: Convertible>(&self, value: &str) -> Result<T, ConversionError>;

    /// 运行时注册表转换（按 `TypeId` 对）。
    ///
    /// # 错误
    ///
    /// 类型对未注册时返回 [`ConversionError`]。
    fn convert_erased(
        &self,
        value: &str,
        source_type: TypeId,
        target_type: TypeId,
    ) -> Result<String, ConversionError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert::support::GenericConversionService;

    #[test]
    fn generic_service_satisfies_contract() {
        // D 类（重构安全）：`GenericConversionService` 实现该契约
        fn assert_configurable<T: ConfigurableConversionService>() {}
        assert_configurable::<GenericConversionService>();
    }

    #[test]
    fn default_service_satisfies_contract() {
        fn assert_configurable<T: ConfigurableConversionService>() {}
        assert_configurable::<crate::convert::support::DefaultConversionService>();
    }
}
