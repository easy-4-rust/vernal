//! SimpleTypeConverter — 对应 Java 类：org.springframework.beans.SimpleTypeConverter。
//!
//! 对应 Spring beans 包。
//!
//! 在 Spring 中，`SimpleTypeConverter` 是 `TypeConverterSupport` 的直接子类，
//! 是最简单的类型转换器实现。它不依赖 BeanWrapper 或 PropertyEditor 机制，
//! 只使用内置的类型转换逻辑（包括 JavaBeans PropertyEditor 和 ConversionService）。
//!
//! ## 与 BeanWrapperImpl 的区别
//!
//! - `SimpleTypeConverter` — 轻量级，仅做类型转换，不涉及 Bean 属性访问
//! - `BeanWrapperImpl` — 完整的 Bean 包装器，包含属性读写 + 类型转换
//!
//! ## 使用场景
//!
//! - 独立的类型转换需求（不涉及 Bean 属性绑定）
//! - 测试环境中的简单类型转换
//! - Service 层的值对象转换

use std::any::Any;

use crate::type_converter::TypeConverter;
use crate::type_mismatch_exception::TypeMismatchException;

/// SimpleTypeConverter — Spring 风格的简单类型转换器。
///
/// 对应 Java 类：`org.springframework.beans.SimpleTypeConverter`。
///
/// 最简单的 TypeConverter 实现，不涉及 Bean 属性访问。
/// 支持基本类型之间的转换：数值类型、布尔类型、字符串。
///
/// ## Java 对比
///
/// | Java | Rust |
/// |------|------|
/// | `SimpleTypeConverter` | `SimpleTypeConverter` |
/// | `convertIfNecessary(Object, Class)` | `convert(value, target_type_name)` |
/// | 使用 PropertyEditor | 使用内置解析逻辑 |
#[derive(Debug, Clone)]
pub struct SimpleTypeConverter {
    /// 转换失败时是否抛出异常（false 则返回 None）
    strict: bool,
    /// 自定义转换错误消息前缀
    error_prefix: String,
}

impl SimpleTypeConverter {
    /// 创建新的 SimpleTypeConverter。
    pub fn new() -> Self {
        Self {
            strict: true,
            error_prefix: "Failed to convert".to_string(),
        }
    }

    /// 创建严格模式的转换器。
    pub fn strict() -> Self {
        Self::new()
    }

    /// 创建宽松模式的转换器（转换失败返回 None 而非错误）。
    pub fn lenient() -> Self {
        Self {
            strict: false,
            error_prefix: "Failed to convert".to_string(),
        }
    }

    /// 设置错误消息前缀。
    pub fn with_error_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.error_prefix = prefix.into();
        self
    }

    /// 是否为严格模式。
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// 设置严格模式。
    pub fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// 将字符串值转换为目标类型。
    ///
    /// # 参数
    /// - `value` — 源字符串值
    /// - `target_type_name` — 目标类型名（如 "i32", "f64", "bool"）
    ///
    /// # 返回
    /// - `Ok(Some(converted))` — 转换成功
    /// - `Ok(None)` — 源值为 None 或宽松模式下转换失败
    /// - `Err(...)` — 严格模式下转换失败
    pub fn convert(
        &self,
        value: Option<&str>,
        target_type_name: &str,
    ) -> Result<Option<Box<dyn Any>>, TypeMismatchException> {
        match value {
            None => Ok(None),
            Some(s) => self.convert_string(s, target_type_name),
        }
    }

    /// 将 &str 转换为指定类型。
    fn convert_string(
        &self,
        value: &str,
        target_type_name: &str,
    ) -> Result<Option<Box<dyn Any>>, TypeMismatchException> {
        let result: Result<Option<Box<dyn Any>>, String> = match target_type_name {
            "String" | "string" | "str" => Ok(Some(Box::new(value.to_string()) as Box<dyn Any>)),
            "i8" => value
                .parse::<i8>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "i16" => value
                .parse::<i16>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "i32" | "int" => value
                .parse::<i32>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "i64" | "long" => value
                .parse::<i64>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "i128" => value
                .parse::<i128>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "u8" => value
                .parse::<u8>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "u16" => value
                .parse::<u16>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "u32" => value
                .parse::<u32>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "u64" => value
                .parse::<u64>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "u128" => value
                .parse::<u128>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "f32" | "float" => value
                .parse::<f32>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "f64" | "double" => value
                .parse::<f64>()
                .map(|v| Some(Box::new(v) as Box<dyn Any>))
                .map_err(|e| e.to_string()),
            "bool" | "boolean" => {
                Self::parse_bool(value).map(|v| Some(Box::new(v) as Box<dyn Any>))
            }
            "char" => {
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(Some(Box::new(c) as Box<dyn Any>)),
                    _ => Err(format!(
                        "Cannot convert '{}' to char: expected single character",
                        value
                    )),
                }
            }
            _ => Err(format!(
                "{}: unsupported target type '{}'",
                self.error_prefix, target_type_name
            )),
        };

        match result {
            Ok(v) => Ok(v),
            Err(msg) => {
                if self.strict {
                    Err(TypeMismatchException::with_details(
                        format!("{}: {}", self.error_prefix, msg),
                        target_type_name.to_string(),
                        value.to_string(),
                    ))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// 解析布尔值，支持多种格式。
    ///
    /// 支持 "true"/"false"、"1"/"0"、"yes"/"no"、"on"/"off"。
    fn parse_bool(value: &str) -> Result<bool, String> {
        match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(format!("Cannot convert '{}' to bool", value)),
        }
    }
}

impl Default for SimpleTypeConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 TypeConverter trait。
impl TypeConverter for SimpleTypeConverter {
    fn convert_if_necessary(
        &self,
        _property_name: Option<&str>,
        value: &dyn Any,
        target_type: std::any::TypeId,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // 尝试将 value 作为字符串处理
        if let Some(s) = value.downcast_ref::<String>() {
            let type_name = if target_type == std::any::TypeId::of::<i32>() {
                "i32"
            } else if target_type == std::any::TypeId::of::<i64>() {
                "i64"
            } else if target_type == std::any::TypeId::of::<f64>() {
                "f64"
            } else if target_type == std::any::TypeId::of::<bool>() {
                "bool"
            } else if target_type == std::any::TypeId::of::<String>() {
                "String"
            } else {
                return Err(Box::new(TypeMismatchException::new(
                    "Unsupported target type",
                )));
            };
            match self.convert(Some(s), type_name) {
                Ok(Some(v)) => Self::box_to_send_sync(v),
                Ok(None) => Err(Box::new(TypeMismatchException::new(
                    "Conversion returned None",
                ))),
                Err(e) => Err(Box::new(e)),
            }
        } else if let Some(s) = value.downcast_ref::<&str>() {
            let type_name = if target_type == std::any::TypeId::of::<String>() {
                "String"
            } else {
                return Err(Box::new(TypeMismatchException::new(
                    "Unsupported target type",
                )));
            };
            match self.convert(Some(s), type_name) {
                Ok(Some(v)) => Self::box_to_send_sync(v),
                Ok(None) => Err(Box::new(TypeMismatchException::new(
                    "Conversion returned None",
                ))),
                Err(e) => Err(Box::new(e)),
            }
        } else {
            // 对于非字符串源值，暂不支持直接转换
            Err(Box::new(TypeMismatchException::new(
                "SimpleTypeConverter only supports String source values",
            )))
        }
    }
}

impl SimpleTypeConverter {
    /// 将 Box<dyn Any> 转换为 Box<dyn Any + Send + Sync>。
    ///
    /// 由于 Rust 类型系统的限制，需要通过 downcast 和 re-box 来实现。
    /// downcast 在失败时返回原始 Box，因此可以链式调用。
    fn box_to_send_sync(
        v: Box<dyn Any>,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        // downcast 返回 Result<Box<T>, Box<dyn Any>>，失败时返回原始值
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
        let v = match v.downcast::<i128>() {
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
        let v = match v.downcast::<u128>() {
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
            Err(_) => {}
        }
        Err(Box::new(TypeMismatchException::new(
            "Cannot convert value to Send + Sync",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_string_to_i32() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("42"), "i32").unwrap();
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn convert_string_to_f64() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("3.14"), "f64").unwrap();
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_string_to_bool_true() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("true"), "bool").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), true);
    }

    #[test]
    fn convert_string_to_bool_yes() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("yes"), "bool").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), true);
    }

    #[test]
    fn convert_string_to_bool_zero() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("0"), "bool").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), false);
    }

    #[test]
    fn convert_none_returns_none() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(None, "i32").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn strict_mode_returns_error_on_failure() {
        let converter = SimpleTypeConverter::strict();
        let result = converter.convert(Some("abc"), "i32");
        assert!(result.is_err());
    }

    #[test]
    fn lenient_mode_returns_none_on_failure() {
        let converter = SimpleTypeConverter::lenient();
        let result = converter.convert(Some("abc"), "i32").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn convert_string_to_i8() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("127"), "i8").unwrap();
        assert_eq!(*result.unwrap().downcast::<i8>().unwrap(), 127);
    }

    #[test]
    fn convert_string_to_u64() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("999"), "u64").unwrap();
        assert_eq!(*result.unwrap().downcast::<u64>().unwrap(), 999);
    }

    #[test]
    fn convert_string_to_char() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("A"), "char").unwrap();
        assert_eq!(*result.unwrap().downcast::<char>().unwrap(), 'A');
    }

    #[test]
    fn convert_invalid_char_fails() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("abc"), "char");
        assert!(result.is_err());
    }

    #[test]
    fn unsupported_type_fails() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("value"), "CustomType");
        assert!(result.is_err());
    }

    // ── Additional numeric type coverage ─────────────────────────────────

    #[test]
    fn convert_string_to_i16() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("1000"), "i16").unwrap();
        assert_eq!(*result.unwrap().downcast::<i16>().unwrap(), 1000i16);
    }

    #[test]
    fn convert_string_to_i128() {
        let converter = SimpleTypeConverter::new();
        let result = converter
            .convert(Some("12345678901234567890"), "i128")
            .unwrap();
        assert_eq!(
            *result.unwrap().downcast::<i128>().unwrap(),
            12345678901234567890i128
        );
    }

    #[test]
    fn convert_string_to_u8() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("255"), "u8").unwrap();
        assert_eq!(*result.unwrap().downcast::<u8>().unwrap(), 255u8);
    }

    #[test]
    fn convert_string_to_u16() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("65535"), "u16").unwrap();
        assert_eq!(*result.unwrap().downcast::<u16>().unwrap(), 65535u16);
    }

    #[test]
    fn convert_string_to_u32() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("4294967295"), "u32").unwrap();
        assert_eq!(*result.unwrap().downcast::<u32>().unwrap(), 4294967295u32);
    }

    #[test]
    fn convert_string_to_u128() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("999"), "u128").unwrap();
        assert_eq!(*result.unwrap().downcast::<u128>().unwrap(), 999u128);
    }

    #[test]
    fn convert_string_to_f32() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("1.5"), "f32").unwrap();
        let val = *result.unwrap().downcast::<f32>().unwrap();
        assert!((val - 1.5f32).abs() < f32::EPSILON);
    }

    // ── String alias coverage ────────────────────────────────────────────

    #[test]
    fn convert_string_to_string_alias() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("hello"), "String").unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn convert_string_to_string_lowercase() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("hello"), "string").unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn convert_string_to_str_alias() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("hello"), "str").unwrap();
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    // ── Type name alias coverage ─────────────────────────────────────────

    #[test]
    fn convert_string_to_int_alias() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("42"), "int").unwrap();
        assert_eq!(*result.unwrap().downcast::<i32>().unwrap(), 42);
    }

    #[test]
    fn convert_string_to_long_alias() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("42"), "long").unwrap();
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn convert_string_to_float_alias() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("1.5"), "float").unwrap();
        let val = *result.unwrap().downcast::<f32>().unwrap();
        assert!((val - 1.5f32).abs() < f32::EPSILON);
    }

    #[test]
    fn convert_string_to_double_alias() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("3.14"), "double").unwrap();
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn convert_string_to_boolean_alias() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("true"), "boolean").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), true);
    }

    // ── parse_bool variants ──────────────────────────────────────────────

    #[test]
    fn convert_bool_on() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("on"), "bool").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), true);
    }

    #[test]
    fn convert_bool_off() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("off"), "bool").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), false);
    }

    #[test]
    fn convert_bool_one() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("1"), "bool").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), true);
    }

    #[test]
    fn convert_bool_no() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("no"), "bool").unwrap();
        assert_eq!(*result.unwrap().downcast::<bool>().unwrap(), false);
    }

    #[test]
    fn convert_bool_invalid() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("maybe"), "bool");
        assert!(result.is_err());
    }

    // ── Error cases for numeric types ────────────────────────────────────

    #[test]
    fn convert_invalid_i8() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("999"), "i8");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_i16() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("abc"), "i16");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_u8() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("abc"), "u8");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_u32() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("abc"), "u32");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_u64() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("abc"), "u64");
        assert!(result.is_err());
    }

    #[test]
    fn convert_invalid_f32() {
        let converter = SimpleTypeConverter::new();
        let result = converter.convert(Some("abc"), "f32");
        assert!(result.is_err());
    }

    // ── Configuration methods ────────────────────────────────────────────

    #[test]
    fn is_strict_default() {
        let converter = SimpleTypeConverter::new();
        assert!(converter.is_strict());
    }

    #[test]
    fn set_strict() {
        let mut converter = SimpleTypeConverter::new();
        converter.set_strict(false);
        assert!(!converter.is_strict());
    }

    #[test]
    fn with_error_prefix() {
        let converter = SimpleTypeConverter::new().with_error_prefix("Custom");
        let result = converter.convert(Some("abc"), "i32");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("Custom"));
    }

    #[test]
    fn default_trait() {
        let converter = SimpleTypeConverter::default();
        assert!(converter.is_strict());
    }

    #[test]
    fn lenient_is_not_strict() {
        let converter = SimpleTypeConverter::lenient();
        assert!(!converter.is_strict());
    }

    // ── TypeConverter trait ───────────────────────────────────────────────

    #[test]
    fn type_converter_trait_string_to_i32() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = "42".to_string();
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<i32>());
        assert!(result.is_ok());
        let val = result.unwrap().downcast::<i32>().unwrap();
        assert_eq!(*val, 42);
    }

    #[test]
    fn type_converter_trait_string_to_i64() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = "42".to_string();
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<i64>());
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn type_converter_trait_string_to_f64() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = "3.14".to_string();
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<f64>());
        assert!(result.is_ok());
        let val = *result.unwrap().downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn type_converter_trait_string_to_bool() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = "true".to_string();
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<bool>());
        assert!(result.is_ok());
        assert!(*result.unwrap().downcast::<bool>().unwrap());
    }

    #[test]
    fn type_converter_trait_string_to_string() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = "hello".to_string();
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<String>());
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn type_converter_trait_unsupported_target_type() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = "hello".to_string();
        let result =
            converter.convert_if_necessary(None, &value, std::any::TypeId::of::<Vec<u8>>());
        assert!(result.is_err());
    }

    #[test]
    fn type_converter_trait_str_ref_to_string() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value: &str = "hello";
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<String>());
        assert!(result.is_ok());
        assert_eq!(*result.unwrap().downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn type_converter_trait_str_ref_unsupported_target() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value: &str = "hello";
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<i32>());
        assert!(result.is_err());
    }

    #[test]
    fn type_converter_trait_non_string_source() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = 42i32;
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<String>());
        assert!(result.is_err());
    }

    #[test]
    fn type_converter_trait_invalid_string_to_i32() {
        use crate::type_converter::TypeConverter;
        let converter = SimpleTypeConverter::new();
        let value = "abc".to_string();
        let result = converter.convert_if_necessary(None, &value, std::any::TypeId::of::<i32>());
        assert!(result.is_err());
    }

    // ── box_to_send_sync coverage ────────────────────────────────────────

    #[test]
    fn box_to_send_sync_i8() {
        let v: Box<dyn std::any::Any> = Box::new(42i8);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i8>().unwrap(), 42i8);
    }

    #[test]
    fn box_to_send_sync_i16() {
        let v: Box<dyn std::any::Any> = Box::new(1000i16);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i16>().unwrap(), 1000i16);
    }

    #[test]
    fn box_to_send_sync_i32() {
        let v: Box<dyn std::any::Any> = Box::new(42i32);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i32>().unwrap(), 42i32);
    }

    #[test]
    fn box_to_send_sync_i64() {
        let v: Box<dyn std::any::Any> = Box::new(42i64);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i64>().unwrap(), 42i64);
    }

    #[test]
    fn box_to_send_sync_i128() {
        let v: Box<dyn std::any::Any> = Box::new(42i128);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<i128>().unwrap(), 42i128);
    }

    #[test]
    fn box_to_send_sync_u8() {
        let v: Box<dyn std::any::Any> = Box::new(255u8);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u8>().unwrap(), 255u8);
    }

    #[test]
    fn box_to_send_sync_u16() {
        let v: Box<dyn std::any::Any> = Box::new(65535u16);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u16>().unwrap(), 65535u16);
    }

    #[test]
    fn box_to_send_sync_u32() {
        let v: Box<dyn std::any::Any> = Box::new(42u32);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u32>().unwrap(), 42u32);
    }

    #[test]
    fn box_to_send_sync_u64() {
        let v: Box<dyn std::any::Any> = Box::new(42u64);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u64>().unwrap(), 42u64);
    }

    #[test]
    fn box_to_send_sync_u128() {
        let v: Box<dyn std::any::Any> = Box::new(42u128);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<u128>().unwrap(), 42u128);
    }

    #[test]
    fn box_to_send_sync_f32() {
        let v: Box<dyn std::any::Any> = Box::new(1.5f32);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<f32>().unwrap(), 1.5f32);
    }

    #[test]
    fn box_to_send_sync_f64() {
        let v: Box<dyn std::any::Any> = Box::new(3.14f64);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        let val = *result.downcast::<f64>().unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn box_to_send_sync_bool() {
        let v: Box<dyn std::any::Any> = Box::new(true);
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert!(*result.downcast::<bool>().unwrap());
    }

    #[test]
    fn box_to_send_sync_char() {
        let v: Box<dyn std::any::Any> = Box::new('x');
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<char>().unwrap(), 'x');
    }

    #[test]
    fn box_to_send_sync_string() {
        let v: Box<dyn std::any::Any> = Box::new("hello".to_string());
        let result = SimpleTypeConverter::box_to_send_sync(v).unwrap();
        assert_eq!(*result.downcast::<String>().unwrap(), "hello");
    }

    #[test]
    fn box_to_send_sync_unsupported_type() {
        struct Unsupported;
        let v: Box<dyn std::any::Any> = Box::new(Unsupported);
        let result = SimpleTypeConverter::box_to_send_sync(v);
        assert!(result.is_err());
    }
}
