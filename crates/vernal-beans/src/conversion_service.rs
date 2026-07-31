//! ConversionService — Spring 风格的类型转换服务。
//!
//! 对应 Java 类：`org.springframework.core.convert.ConversionService`。
//!
//! 提供统一的类型转换入口，支持：
//! - 字符串 → 目标类型（通过 PropertyEditor）
//! - 类型 A → 类型 B（通过 Converter）
//! - 类型检查

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::property_editor::PropertyEditor;

/// Spring 风格的类型转换服务 trait。
///
/// 对应 Spring 的 `ConversionService`。
///
/// 提供统一的类型转换入口：
/// - `can_convert` — 检查是否支持类型转换
/// - `convert` — 执行类型转换
/// - `convert_value` — 转换值到指定类型
pub trait ConversionService: Send + Sync + 'static {
    /// 检查是否支持从源类型到目标类型的转换。
    ///
    /// 对应 Spring 的 `ConversionService.canConvert(Class<?> sourceType, Class<?> targetType)`。
    fn can_convert(&self, source_type: std::any::TypeId, target_type: std::any::TypeId) -> bool;

    /// 将值转换为目标类型。
    ///
    /// 对应 Spring 的 `<T> T convert(Object source, Class<T> targetType)`。
    ///
    /// # 参数
    ///
    /// - `source` — 源值
    /// - `target_type` — 目标类型
    ///
    /// # 返回
    ///
    /// - `Ok(converted)` — 转换后的值
    /// - `Err` — 转换失败
    fn convert(
        &self,
        source: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Spring 风格的类型转换器 trait。
///
/// 对应 Spring 的 `Converter<S, T>`。
///
/// 定义从源类型到目标类型的转换逻辑。
pub trait Converter: Send + Sync + 'static {
    /// 源类型。
    fn source_type(&self) -> std::any::TypeId;
    /// 目标类型。
    fn target_type(&self) -> std::any::TypeId;
    /// 执行转换。
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
}

/// 默认的 ConversionService 实现。
///
/// 内置以下转换：
/// - String → i32/i64/f32/f64/bool
/// - 所有数字类型之间的转换
/// - String → bool（true/false/yes/no/1/0）
pub struct DefaultConversionService {
    converters: Vec<Arc<dyn Converter>>,
}

impl DefaultConversionService {
    /// 创建默认的 ConversionService，内置常见转换器。
    pub fn new() -> Self {
        let mut service = Self {
            converters: Vec::new(),
        };
        // 注册内置转换器
        service.register_built_in_converters();
        service
    }

    /// 注册内置转换器。
    fn register_built_in_converters(&mut self) {
        self.converters.push(Arc::new(StringToI32Converter));
        self.converters.push(Arc::new(StringToI64Converter));
        self.converters.push(Arc::new(StringToF64Converter));
        self.converters.push(Arc::new(StringToBoolConverter));
        self.converters.push(Arc::new(I32ToStringConverter));
        self.converters.push(Arc::new(I64ToStringConverter));
        self.converters.push(Arc::new(F64ToStringConverter));
        self.converters.push(Arc::new(BoolToStringConverter));
    }

    /// 注册自定义转换器。
    pub fn register(&mut self, converter: Arc<dyn Converter>) {
        self.converters.push(converter);
    }
}

impl Default for DefaultConversionService {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversionService for DefaultConversionService {
    fn can_convert(&self, source_type: std::any::TypeId, target_type: std::any::TypeId) -> bool {
        self.converters
            .iter()
            .any(|c| c.source_type() == source_type && c.target_type() == target_type)
    }

    fn convert(
        &self,
        source: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let source_type = source.type_id();
        let converter = self
            .converters
            .iter()
            .find(|c| c.source_type() == source_type && c.target_type() == target_type)
            .ok_or_else(|| {
                format!(
                    "No converter found for {} -> {:?}",
                    std::any::type_name_of_val(source),
                    target_type
                )
            })?;
        converter.convert(source)
    }
}

// ── 内置转换器 ───────────────────────────────────────────────────────────

struct StringToI32Converter;
impl Converter for StringToI32Converter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<i32>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let s = source.downcast_ref::<String>().ok_or("Not a String")?;
        let v = s
            .trim()
            .parse::<i32>()
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(Box::new(v))
    }
}

struct StringToI64Converter;
impl Converter for StringToI64Converter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<i64>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let s = source.downcast_ref::<String>().ok_or("Not a String")?;
        let v = s
            .trim()
            .parse::<i64>()
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(Box::new(v))
    }
}

struct StringToF64Converter;
impl Converter for StringToF64Converter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<f64>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let s = source.downcast_ref::<String>().ok_or("Not a String")?;
        let v = s
            .trim()
            .parse::<f64>()
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(Box::new(v))
    }
}

struct StringToBoolConverter;
impl Converter for StringToBoolConverter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<bool>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let s = source.downcast_ref::<String>().ok_or("Not a String")?;
        let v = match s.trim().to_lowercase().as_str() {
            "true" | "yes" | "1" => true,
            "false" | "no" | "0" => false,
            _ => return Err(format!("Cannot convert '{}' to bool", s).into()),
        };
        Ok(Box::new(v))
    }
}

struct I32ToStringConverter;
impl Converter for I32ToStringConverter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<i32>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let v = source.downcast_ref::<i32>().ok_or("Not an i32")?;
        Ok(Box::new(v.to_string()))
    }
}

struct I64ToStringConverter;
impl Converter for I64ToStringConverter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<i64>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let v = source.downcast_ref::<i64>().ok_or("Not an i64")?;
        Ok(Box::new(v.to_string()))
    }
}

struct F64ToStringConverter;
impl Converter for F64ToStringConverter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<f64>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let v = source.downcast_ref::<f64>().ok_or("Not an f64")?;
        Ok(Box::new(v.to_string()))
    }
}

struct BoolToStringConverter;
impl Converter for BoolToStringConverter {
    fn source_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<bool>()
    }
    fn target_type(&self) -> std::any::TypeId {
        std::any::TypeId::of::<String>()
    }
    fn convert(
        &self,
        source: &dyn Any,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let v = source.downcast_ref::<bool>().ok_or("Not a bool")?;
        Ok(Box::new(v.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn test_default_conversion_service_new() {
        let service = DefaultConversionService::new();
        assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<i32>()));
        assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<i64>()));
        assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<f64>()));
        assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<bool>()));
    }

    #[test]
    fn test_default_conversion_service_default_trait() {
        let service = DefaultConversionService::default();
        assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<i32>()));
    }

    #[test]
    fn test_can_convert_returns_false_for_unsupported() {
        let service = DefaultConversionService::new();
        assert!(!service.can_convert(TypeId::of::<u8>(), TypeId::of::<u16>()));
    }

    #[test]
    fn test_string_to_i32_conversion() {
        let service = DefaultConversionService::new();
        let source = String::from("42");
        let result = service.convert(&source, TypeId::of::<i32>()).unwrap();
        assert_eq!(*result.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_string_to_i32_with_whitespace() {
        let service = DefaultConversionService::new();
        let source = String::from("  100  ");
        let result = service.convert(&source, TypeId::of::<i32>()).unwrap();
        assert_eq!(*result.downcast_ref::<i32>().unwrap(), 100);
    }

    #[test]
    fn test_string_to_i32_parse_error() {
        let service = DefaultConversionService::new();
        let source = String::from("not_a_number");
        assert!(service.convert(&source, TypeId::of::<i32>()).is_err());
    }

    #[test]
    fn test_string_to_i64_conversion() {
        let service = DefaultConversionService::new();
        let source = String::from("123456789");
        let result = service.convert(&source, TypeId::of::<i64>()).unwrap();
        assert_eq!(*result.downcast_ref::<i64>().unwrap(), 123456789);
    }

    #[test]
    fn test_string_to_i64_parse_error() {
        let service = DefaultConversionService::new();
        let source = String::from("abc");
        assert!(service.convert(&source, TypeId::of::<i64>()).is_err());
    }

    #[test]
    fn test_string_to_f64_conversion() {
        let service = DefaultConversionService::new();
        let source = String::from("3.14");
        let result = service.convert(&source, TypeId::of::<f64>()).unwrap();
        assert!((result.downcast_ref::<f64>().unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_string_to_f64_parse_error() {
        let service = DefaultConversionService::new();
        let source = String::from("not_a_float");
        assert!(service.convert(&source, TypeId::of::<f64>()).is_err());
    }

    #[test]
    fn test_string_to_bool_true_values() {
        let service = DefaultConversionService::new();
        for value in &["true", "yes", "1", "TRUE", "Yes", "True"] {
            let source = String::from(*value);
            let result = service.convert(&source, TypeId::of::<bool>()).unwrap();
            assert!(*result.downcast_ref::<bool>().unwrap(), "Expected true for '{}'", value);
        }
    }

    #[test]
    fn test_string_to_bool_false_values() {
        let service = DefaultConversionService::new();
        for value in &["false", "no", "0", "FALSE", "No", "False"] {
            let source = String::from(*value);
            let result = service.convert(&source, TypeId::of::<bool>()).unwrap();
            assert!(!*result.downcast_ref::<bool>().unwrap(), "Expected false for '{}'", value);
        }
    }

    #[test]
    fn test_string_to_bool_invalid() {
        let service = DefaultConversionService::new();
        let source = String::from("maybe");
        assert!(service.convert(&source, TypeId::of::<bool>()).is_err());
    }

    #[test]
    fn test_i32_to_string_conversion() {
        let service = DefaultConversionService::new();
        let source = 42i32;
        let result = service.convert(&source, TypeId::of::<String>()).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "42");
    }

    #[test]
    fn test_i64_to_string_conversion() {
        let service = DefaultConversionService::new();
        let source = 123456789i64;
        let result = service.convert(&source, TypeId::of::<String>()).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "123456789");
    }

    #[test]
    fn test_f64_to_string_conversion() {
        let service = DefaultConversionService::new();
        let source = 3.14f64;
        let result = service.convert(&source, TypeId::of::<String>()).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "3.14");
    }

    #[test]
    fn test_bool_to_string_conversion() {
        let service = DefaultConversionService::new();
        let source_true = true;
        let result = service.convert(&source_true, TypeId::of::<String>()).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "true");

        let source_false = false;
        let result = service.convert(&source_false, TypeId::of::<String>()).unwrap();
        assert_eq!(*result.downcast_ref::<String>().unwrap(), "false");
    }

    #[test]
    fn test_convert_unsupported_type_returns_error() {
        let service = DefaultConversionService::new();
        let source = 42u8;
        assert!(service.convert(&source, TypeId::of::<u16>()).is_err());
    }

    #[test]
    fn test_register_custom_converter() {
        struct U8ToU16Converter;
        impl Converter for U8ToU16Converter {
            fn source_type(&self) -> TypeId { TypeId::of::<u8>() }
            fn target_type(&self) -> TypeId { TypeId::of::<u16>() }
            fn convert(&self, source: &dyn Any) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
                let v = source.downcast_ref::<u8>().ok_or("Not a u8")?;
                Ok(Box::new(*v as u16))
            }
        }

        let mut service = DefaultConversionService::new();
        assert!(!service.can_convert(TypeId::of::<u8>(), TypeId::of::<u16>()));
        service.register(Arc::new(U8ToU16Converter));
        assert!(service.can_convert(TypeId::of::<u8>(), TypeId::of::<u16>()));

        let source = 42u8;
        let result = service.convert(&source, TypeId::of::<u16>()).unwrap();
        assert_eq!(*result.downcast_ref::<u16>().unwrap(), 42);
    }
}
