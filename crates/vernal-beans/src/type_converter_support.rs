//! TypeConverterSupport — 对应 Java 类：org.springframework.beans.TypeConverterSupport。
//!
//! 对应 Spring beans 包。
//!
//! `TypeConverterSupport` 是 `TypeConverter` 接口的基础实现，
//! 提供类型转换的委托机制。在 Spring 中，它将实际转换工作
//! 委托给 `TypeConverterDelegate`。
//!
//! ## 设计说明
//!
//! - `TypeConverterSupport` 是抽象类（Java 中不可直接实例化）
//! - 在 vernal 中，它作为具体实现类，提供所有基本类型的转换支持
//! - 支持严格模式和宽松模式

use std::any::Any;

use crate::type_converter::TypeConverter;
use crate::type_mismatch_exception::TypeMismatchException;

/// TypeConverterSupport — Spring 风格的类型转换器基础实现。
///
/// 对应 Java 类：`org.springframework.beans.TypeConverterSupport`。
///
/// 提供类型转换的基础实现，将转换请求委托给内部的转换逻辑。
/// 支持将值从一种类型转换为目标类型。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `TypeConverterSupport` | `TypeConverterSupport` |
/// | `convertIfNecessary(Object, Class)` | `convert_if_necessary(value, target)` |
/// | `TypeConverterDelegate` | 内置转换逻辑 |
#[derive(Debug, Clone, Default)]
pub struct TypeConverterSupport {
    /// 是否启用严格类型检查
    strict_type_check: bool,
}

impl TypeConverterSupport {
    /// 创建新的 TypeConverterSupport。
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带有严格类型检查的 TypeConverterSupport。
    pub fn with_strict_type_check(strict: bool) -> Self {
        Self {
            strict_type_check: strict,
        }
    }

    /// 是否启用严格类型检查。
    pub fn is_strict_type_check(&self) -> bool {
        self.strict_type_check
    }

    /// 设置严格类型检查模式。
    pub fn set_strict_type_check(&mut self, strict: bool) {
        self.strict_type_check = strict;
    }

    /// 尝试将值转换为目标类型。
    ///
    /// 对应 Spring 的 `TypeConverter.convertIfNecessary`。
    ///
    /// # 参数
    /// - `value` — 要转换的值
    /// - `target_type_name` — 目标类型的名称
    ///
    /// # 返回
    /// - `Ok(Some(converted))` — 转换成功
    /// - `Ok(None)` — 值为 None，无需转换
    /// - `Err(...)` — 转换失败
    pub fn convert_if_necessary(
        &self,
        value: Option<Box<dyn Any>>,
        target_type_name: &str,
    ) -> Result<Option<Box<dyn Any>>, TypeMismatchException> {
        match value {
            None => Ok(None),
            Some(val) => {
                // 尝试基本类型转换
                if let Some(s) = val.downcast_ref::<String>() {
                    return self.convert_string(s, target_type_name);
                }
                if let Some(&n) = val.downcast_ref::<i32>() {
                    return self.convert_number(n as f64, target_type_name);
                }
                if let Some(&n) = val.downcast_ref::<i64>() {
                    return self.convert_number(n as f64, target_type_name);
                }
                if let Some(&n) = val.downcast_ref::<f64>() {
                    return self.convert_number(n, target_type_name);
                }
                if let Some(&b) = val.downcast_ref::<bool>() {
                    return self.convert_from_bool(b, target_type_name);
                }
                // 如果目标类型与源类型匹配，直接返回
                Ok(Some(val))
            }
        }
    }

    /// 将字符串值转换为目标类型。
    fn convert_string(
        &self,
        value: &str,
        target_type_name: &str,
    ) -> Result<Option<Box<dyn Any>>, TypeMismatchException> {
        match target_type_name {
            "String" | "string" | "str" => Ok(Some(Box::new(value.to_string()))),
            "i8" => self.parse_or_err::<i8>(value, target_type_name),
            "i16" => self.parse_or_err::<i16>(value, target_type_name),
            "i32" | "int" => self.parse_or_err::<i32>(value, target_type_name),
            "i64" | "long" => self.parse_or_err::<i64>(value, target_type_name),
            "u8" => self.parse_or_err::<u8>(value, target_type_name),
            "u16" => self.parse_or_err::<u16>(value, target_type_name),
            "u32" => self.parse_or_err::<u32>(value, target_type_name),
            "u64" => self.parse_or_err::<u64>(value, target_type_name),
            "f32" | "float" => self.parse_or_err::<f32>(value, target_type_name),
            "f64" | "double" => self.parse_or_err::<f64>(value, target_type_name),
            "bool" | "boolean" => self.parse_or_err::<bool>(value, target_type_name),
            "char" => {
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(Some(Box::new(c))),
                    _ => Err(TypeMismatchException::with_details(
                        format!("Cannot convert '{}' to char", value),
                        target_type_name.to_string(),
                        value.to_string(),
                    )),
                }
            }
            _ => Err(TypeMismatchException::with_details(
                format!("Unsupported target type: {}", target_type_name),
                target_type_name.to_string(),
                value.to_string(),
            )),
        }
    }

    /// 将数值类型转换为目标类型。
    fn convert_number(
        &self,
        value: f64,
        target_type_name: &str,
    ) -> Result<Option<Box<dyn Any>>, TypeMismatchException> {
        match target_type_name {
            "i32" | "int" => Ok(Some(Box::new(value as i32))),
            "i64" | "long" => Ok(Some(Box::new(value as i64))),
            "f32" | "float" => Ok(Some(Box::new(value as f32))),
            "f64" | "double" => Ok(Some(Box::new(value))),
            "String" | "string" => Ok(Some(Box::new(value.to_string()))),
            "bool" | "boolean" => Ok(Some(Box::new(value != 0.0))),
            _ => Err(TypeMismatchException::with_details(
                format!("Cannot convert number to '{}'", target_type_name),
                target_type_name.to_string(),
                value.to_string(),
            )),
        }
    }

    /// 从布尔值转换为目标类型。
    fn convert_from_bool(
        &self,
        value: bool,
        target_type_name: &str,
    ) -> Result<Option<Box<dyn Any>>, TypeMismatchException> {
        match target_type_name {
            "bool" | "boolean" => Ok(Some(Box::new(value))),
            "String" | "string" => Ok(Some(Box::new(value.to_string()))),
            "i32" | "int" => Ok(Some(Box::new(if value { 1i32 } else { 0i32 }))),
            "i64" | "long" => Ok(Some(Box::new(if value { 1i64 } else { 0i64 }))),
            _ => Err(TypeMismatchException::with_details(
                format!("Cannot convert bool to '{}'", target_type_name),
                target_type_name.to_string(),
                value.to_string(),
            )),
        }
    }

    /// 辅助方法：解析字符串为目标类型，失败时返回错误。
    fn parse_or_err<T: std::str::FromStr + Any + 'static>(
        &self,
        value: &str,
        target_type_name: &str,
    ) -> Result<Option<Box<dyn Any>>, TypeMismatchException> {
        value.parse::<T>().map(|v| Some(Box::new(v) as Box<dyn Any>)).map_err(|_| {
            TypeMismatchException::with_details(
                format!("Cannot convert '{}' to {}", value, target_type_name),
                target_type_name.to_string(),
                value.to_string(),
            )
        })
    }
}

/// 实现 TypeConverter trait。
impl TypeConverter for TypeConverterSupport {
    fn convert_if_necessary(
        &self,
        _property_name: Option<&str>,
        value: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(s) = value.downcast_ref::<String>() {
            let type_name = Self::type_id_to_name(target_type);
            match self.convert_string(s, type_name) {
                Ok(Some(v)) => Self::box_to_send_sync(v),
                Ok(None) => Err(Box::new(TypeMismatchException::new("Conversion returned None"))),
                Err(e) => Err(Box::new(e)),
            }
        } else {
            Err(Box::new(TypeMismatchException::new("Unsupported source type")))
        }
    }
}

impl TypeConverterSupport {
    /// 将 TypeId 映射为类型名。
    fn type_id_to_name(type_id: std::any::TypeId) -> &'static str {
        if type_id == std::any::TypeId::of::<i32>() { "i32" }
        else if type_id == std::any::TypeId::of::<i64>() { "i64" }
        else if type_id == std::any::TypeId::of::<f32>() { "f32" }
        else if type_id == std::any::TypeId::of::<f64>() { "f64" }
        else if type_id == std::any::TypeId::of::<bool>() { "bool" }
        else if type_id == std::any::TypeId::of::<String>() { "String" }
        else { "unknown" }
    }

    /// 将 Box<dyn Any> 转换为 Box<dyn Any + Send + Sync>。
    fn box_to_send_sync(v: Box<dyn Any>) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let v = match v.downcast::<String>() {
            Ok(s) => return Ok(s),
            Err(v) => v,
        };
        let v = match v.downcast::<i8>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<i16>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<i32>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<i64>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<u8>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<u16>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<u32>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<u64>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<f32>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<f64>() {
            Ok(n) => return Ok(n),
            Err(v) => v,
        };
        let v = match v.downcast::<bool>() {
            Ok(b) => return Ok(b),
            Err(v) => v,
        };
        match v.downcast::<char>() {
            Ok(c) => return Ok(c),
            Err(_) => {},
        }
        Err(Box::new(TypeMismatchException::new("Cannot convert value")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_default_has_no_strict_check() {
        let converter = TypeConverterSupport::new();
        assert!(!converter.is_strict_type_check());
    }

    #[test]
    fn with_strict_type_check() {
        let converter = TypeConverterSupport::with_strict_type_check(true);
        assert!(converter.is_strict_type_check());
    }

    #[test]
    fn convert_none_returns_none() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(None, "String").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn convert_string_to_i32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("42".to_string())), "i32")
            .unwrap();
        assert!(result.is_some());
        let val = result.unwrap().downcast::<i32>().unwrap();
        assert_eq!(*val, 42);
    }

    #[test]
    fn convert_string_to_bool() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("true".to_string())), "bool")
            .unwrap();
        assert!(result.is_some());
        let val = result.unwrap().downcast::<bool>().unwrap();
        assert!(*val);
    }

    #[test]
    fn convert_invalid_string_to_i32_fails() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new("abc".to_string())), "i32");
        assert!(result.is_err());
    }

    #[test]
    fn set_strict_type_check() {
        let mut converter = TypeConverterSupport::new();
        converter.set_strict_type_check(true);
        assert!(converter.is_strict_type_check());
    }

    #[test]
    fn convert_i32_to_i64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i32)), "i64")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn convert_i32_to_f64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i32)), "f64")
            .unwrap();
        assert!(result.is_some());
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_f64_to_string() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(3.14f64)), "String")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "3.14");
    }

    #[test]
    fn convert_bool_to_string() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(true)), "String")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "true");
    }

    #[test]
    fn convert_string_to_f32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("1.5".to_string())), "f32")
            .unwrap();
        assert!(result.is_some());
        let val = *result.unwrap().downcast::<f32>().unwrap();
        assert!((val - 1.5f32).abs() < f32::EPSILON);
    }

    #[test]
    fn convert_string_to_i8() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("127".to_string())), "i8")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast::<i8>().unwrap(), 127i8);
    }

    #[test]
    fn convert_string_to_char() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("A".to_string())), "char")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast::<char>().unwrap(), 'A');
    }

    #[test]
    fn convert_bool_to_i32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(true)), "i32")
            .unwrap();
        assert!(result.is_some());
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 1);
    }

    // ── Additional string conversion coverage ────────────────────────────

    #[test]
    fn convert_string_to_i16() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("1000".to_string())), "i16")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i16>().unwrap(), 1000i16);
    }

    #[test]
    fn convert_string_to_i64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("42".to_string())), "i64")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn convert_string_to_u8() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("255".to_string())), "u8")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<u8>().unwrap(), 255u8);
    }

    #[test]
    fn convert_string_to_u16() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("65535".to_string())), "u16")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<u16>().unwrap(), 65535u16);
    }

    #[test]
    fn convert_string_to_u32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("42".to_string())), "u32")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<u32>().unwrap(), 42u32);
    }

    #[test]
    fn convert_string_to_u64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("42".to_string())), "u64")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<u64>().unwrap(), 42u64);
    }

    #[test]
    fn convert_string_to_f64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("3.14".to_string())), "f64")
            .unwrap();
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    // ── String alias coverage ────────────────────────────────────────────

    #[test]
    fn convert_string_to_string_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("hello".to_string())), "String")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn convert_string_to_string_lowercase() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("hello".to_string())), "string")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn convert_string_to_str_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("hello".to_string())), "str")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    // ── Type name alias coverage ─────────────────────────────────────────

    #[test]
    fn convert_string_to_int_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("42".to_string())), "int")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn convert_string_to_long_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("42".to_string())), "long")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn convert_string_to_float_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("1.5".to_string())), "float")
            .unwrap();
        let val = *result.unwrap().downcast::<f32>().unwrap();
        assert!((val - 1.5f32).abs() < f32::EPSILON);
    }

    #[test]
    fn convert_string_to_double_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("3.14".to_string())), "double")
            .unwrap();
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_string_to_boolean_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new("true".to_string())), "boolean")
            .unwrap();
        assert!(*result.unwrap().downcast::<bool>().unwrap());
    }

    // ── Invalid string conversions ───────────────────────────────────────

    #[test]
    fn convert_invalid_string_to_i8() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new("abc".to_string())), "i8");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_string_to_u8() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new("abc".to_string())), "u8");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_string_to_f32() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new("abc".to_string())), "f32");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_char() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new("abc".to_string())), "char");
        assert!(result.is_err());
    }

    #[test]
    fn convert_unsupported_target_type() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new("abc".to_string())), "CustomType");
        assert!(result.is_err());
    }

    // ── Number conversions ───────────────────────────────────────────────

    #[test]
    fn convert_i32_to_i32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i32)), "i32")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 42i32);
    }

    #[test]
    fn convert_i32_to_float() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i32)), "float")
            .unwrap();
        let val = *result.unwrap().downcast::<f32>().unwrap();
        assert!((val - 42.0f32).abs() < f32::EPSILON);
    }

    #[test]
    fn convert_i32_to_string() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i32)), "String")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "42");
    }

    #[test]
    fn convert_i32_to_bool() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i32)), "bool")
            .unwrap();
        assert!(*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn convert_i32_zero_to_bool() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(0i32)), "bool")
            .unwrap();
        assert!(!*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn convert_i32_to_unsupported() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new(42i32)), "CustomType");
        assert!(result.is_err());
    }

    // ── i64 conversions ──────────────────────────────────────────────────

    #[test]
    fn convert_i64_to_i32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i64)), "i32")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 42i32);
    }

    #[test]
    fn convert_i64_to_i64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i64)), "i64")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn convert_i64_to_f64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i64)), "f64")
            .unwrap();
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_i64_to_string() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42i64)), "String")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "42");
    }

    // ── f64 conversions ──────────────────────────────────────────────────

    #[test]
    fn convert_f64_to_i32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(3.14f64)), "i32")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 3i32);
    }

    #[test]
    fn convert_f64_to_i64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(3.14f64)), "i64")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 3i64);
    }

    #[test]
    fn convert_f64_to_f32() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(3.14f64)), "f32")
            .unwrap();
        let val = *result.unwrap().downcast::<f32>().unwrap();
        assert!((val - 3.14f32).abs() < 0.01);
    }

    #[test]
    fn convert_f64_to_f64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(3.14f64)), "f64")
            .unwrap();
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_f64_to_bool() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(3.14f64)), "bool")
            .unwrap();
        assert!(*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn convert_f64_zero_to_bool() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(0.0f64)), "bool")
            .unwrap();
        assert!(!*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn convert_f64_to_unsupported() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new(3.14f64)), "CustomType");
        assert!(result.is_err());
    }

    // ── Bool conversions ─────────────────────────────────────────────────

    #[test]
    fn convert_bool_true_to_string() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(true)), "String")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "true");
    }

    #[test]
    fn convert_bool_false_to_string() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(false)), "String")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "false");
    }

    #[test]
    fn convert_bool_to_bool() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(true)), "bool")
            .unwrap();
        assert!(*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn convert_bool_to_boolean_alias() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(true)), "boolean")
            .unwrap();
        assert!(*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn convert_bool_to_i64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(true)), "i64")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 1i64);
    }

    #[test]
    fn convert_bool_false_to_i64() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(false)), "i64")
            .unwrap();
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 0i64);
    }

    #[test]
    fn convert_bool_to_unsupported() {
        let converter = TypeConverterSupport::new();
        let result = converter.convert_if_necessary(Some(Box::new(true)), "CustomType");
        assert!(result.is_err());
    }

    // ── Passthrough for matching types ───────────────────────────────────

    #[test]
    fn passthrough_matching_type() {
        let converter = TypeConverterSupport::new();
        let result = converter
            .convert_if_necessary(Some(Box::new(42u32)), "u32")
            .unwrap();
        // u32 is not String/i32/i64/f64/bool, so it passes through
        assert!(result.is_some());
    }

    // ── TypeConverter trait (via UFCS to disambiguate from inherent method) ──

    #[test]
    fn type_converter_trait_string_to_i32() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = "42".to_string();
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<i32>());
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn type_converter_trait_string_to_i64() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = "42".to_string();
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<i64>());
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn type_converter_trait_string_to_f32() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = "1.5".to_string();
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<f32>());
        assert!(result.is_ok());
        let val = *result.unwrap().downcast::<f32>().unwrap();
        assert!((val - 1.5f32).abs() < f32::EPSILON);
    }

    #[test]
    fn type_converter_trait_string_to_f64() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = "3.14".to_string();
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<f64>());
        assert!(result.is_ok());
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn type_converter_trait_string_to_bool() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = "true".to_string();
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<bool>());
        assert!(result.is_ok());
        assert!(*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn type_converter_trait_string_to_string() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = "hello".to_string();
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<String>());
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn type_converter_trait_non_string_source() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = 42i32;
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn type_converter_trait_invalid_conversion() {
        use crate::type_converter::TypeConverter;
        let converter = TypeConverterSupport::new();
        let value = "abc".to_string();
        let result = TypeConverter::convert_if_necessary(&converter, None, &value, std::any::TypeId::of::<i32>());
        assert!(result.is_err());
    }

    // ── type_id_to_name ──────────────────────────────────────────────────

    #[test]
    fn type_id_to_name_i32() {
        assert_eq!(TypeConverterSupport::type_id_to_name(std::any::TypeId::of::<i32>()), "i32");
    }

    #[test]
    fn type_id_to_name_i64() {
        assert_eq!(TypeConverterSupport::type_id_to_name(std::any::TypeId::of::<i64>()), "i64");
    }

    #[test]
    fn type_id_to_name_f32() {
        assert_eq!(TypeConverterSupport::type_id_to_name(std::any::TypeId::of::<f32>()), "f32");
    }

    #[test]
    fn type_id_to_name_f64() {
        assert_eq!(TypeConverterSupport::type_id_to_name(std::any::TypeId::of::<f64>()), "f64");
    }

    #[test]
    fn type_id_to_name_bool() {
        assert_eq!(TypeConverterSupport::type_id_to_name(std::any::TypeId::of::<bool>()), "bool");
    }

    #[test]
    fn type_id_to_name_string() {
        assert_eq!(TypeConverterSupport::type_id_to_name(std::any::TypeId::of::<String>()), "String");
    }

    #[test]
    fn type_id_to_name_unknown() {
        assert_eq!(TypeConverterSupport::type_id_to_name(std::any::TypeId::of::<Vec<u8>>()), "unknown");
    }

    // ── box_to_send_sync ─────────────────────────────────────────────────

    #[test]
    fn box_to_send_sync_i8() {
        let v: Box<dyn std::any::Any> = Box::new(42i8);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i8>().unwrap(), 42i8);
    }

    #[test]
    fn box_to_send_sync_i16() {
        let v: Box<dyn std::any::Any> = Box::new(1000i16);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i16>().unwrap(), 1000i16);
    }

    #[test]
    fn box_to_send_sync_i32() {
        let v: Box<dyn std::any::Any> = Box::new(42i32);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i32>().unwrap(), 42i32);
    }

    #[test]
    fn box_to_send_sync_i64() {
        let v: Box<dyn std::any::Any> = Box::new(42i64);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn box_to_send_sync_u8() {
        let v: Box<dyn std::any::Any> = Box::new(255u8);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u8>().unwrap(), 255u8);
    }

    #[test]
    fn box_to_send_sync_u16() {
        let v: Box<dyn std::any::Any> = Box::new(65535u16);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u16>().unwrap(), 65535u16);
    }

    #[test]
    fn box_to_send_sync_u32() {
        let v: Box<dyn std::any::Any> = Box::new(42u32);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u32>().unwrap(), 42u32);
    }

    #[test]
    fn box_to_send_sync_u64() {
        let v: Box<dyn std::any::Any> = Box::new(42u64);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u64>().unwrap(), 42u64);
    }

    #[test]
    fn box_to_send_sync_f32() {
        let v: Box<dyn std::any::Any> = Box::new(1.5f32);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<f32>().unwrap(), 1.5f32);
    }

    #[test]
    fn box_to_send_sync_f64() {
        let v: Box<dyn std::any::Any> = Box::new(3.14f64);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert!((*result.downcast::<f64>().unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn box_to_send_sync_bool() {
        let v: Box<dyn std::any::Any> = Box::new(true);
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert!(*result.downcast::<bool>().unwrap());
    }

    #[test]
    fn box_to_send_sync_char() {
        let v: Box<dyn std::any::Any> = Box::new('x');
        let result = TypeConverterSupport::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<char>().unwrap(), 'x');
    }

    #[test]
    fn box_to_send_sync_unsupported() {
        struct Unsupported;
        let v: Box<dyn std::any::Any> = Box::new(Unsupported);
        let result = TypeConverterSupport::box_to_send_sync(v);
        assert!(result.is_err());
    }
}
