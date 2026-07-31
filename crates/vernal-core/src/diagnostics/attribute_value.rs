//! Span 属性值。
//!
//! 对标 OpenTelemetry `Attribute` 与 Spring `KeyValues`。
//! 表示 Span 携带的键值对属性值,支持常见的标量类型。

use std::fmt;

/// Span 属性值。
///
/// 对标 OpenTelemetry 属性类型:
/// - 字符串
/// - 整数(64-bit)
/// - 浮点数(64-bit)
/// - 布尔
///
/// vernal-core 不支持数组类型(对标 `OTel` `array<string>` 等),
/// 留给上层 `vernal-observability` crate 处理。
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// 字符串值。
    String(std::borrow::Cow<'static, str>),
    /// 64-bit 有符号整数。
    Int(i64),
    /// 64-bit 浮点数。
    Float(f64),
    /// 布尔值。
    Bool(bool),
}

impl AttributeValue {
    /// 创建字符串属性值(从 `&'static str`)。
    #[must_use]
    pub fn string(value: &'static str) -> Self {
        Self::String(std::borrow::Cow::Borrowed(value))
    }

    /// 创建字符串属性值(从 `String`,拥有所有权)。
    #[must_use]
    pub fn string_owned(value: String) -> Self {
        Self::String(std::borrow::Cow::Owned(value))
    }

    /// 是否为字符串类型。
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// 获取字符串值(如果是字符串类型)。
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// 获取整数值(如果是整数类型)。
    #[must_use]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// 获取浮点数值(如果是浮点类型)。
    #[must_use]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// 获取布尔值(如果是布尔类型)。
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl From<&'static str> for AttributeValue {
    fn from(s: &'static str) -> Self {
        Self::string(s)
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        Self::string_owned(s)
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<i32> for AttributeValue {
    fn from(i: i32) -> Self {
        Self::Int(i64::from(i))
    }
}

impl From<u64> for AttributeValue {
    fn from(u: u64) -> Self {
        Self::Int(u as i64)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl fmt::Display for AttributeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Int(i) => write!(f, "{i}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::Bool(b) => write!(f, "{b}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_from_static_str() {
        let v = AttributeValue::from("hello");
        assert!(v.is_string());
        assert_eq!(v.as_string(), Some("hello"));
    }

    #[test]
    fn string_from_owned_string() {
        let v = AttributeValue::from("world".to_string());
        assert!(v.is_string());
        assert_eq!(v.as_string(), Some("world"));
    }

    #[test]
    fn int_from_i64() {
        let v = AttributeValue::from(42_i64);
        assert_eq!(v.as_int(), Some(42));
        assert!(v.as_string().is_none());
    }

    #[test]
    fn int_from_i32() {
        let v = AttributeValue::from(100_i32);
        assert_eq!(v.as_int(), Some(100));
    }

    #[test]
    fn int_from_u64() {
        let v = AttributeValue::from(1_000_000_u64);
        assert_eq!(v.as_int(), Some(1_000_000));
    }

    #[test]
    fn float_from_f64() {
        let v = AttributeValue::from(3.14_f64);
        assert_eq!(v.as_float(), Some(3.14));
    }

    #[test]
    fn bool_from_bool() {
        let v = AttributeValue::from(true);
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn type_checkers_work() {
        let s = AttributeValue::string("x");
        let i = AttributeValue::Int(0);
        let f = AttributeValue::Float(0.0);
        let b = AttributeValue::Bool(false);
        assert!(s.is_string());
        assert!(!i.is_string());
        assert!(!f.is_string());
        assert!(!b.is_string());
    }

    #[test]
    fn display_outputs_value() {
        assert_eq!(AttributeValue::string("hi").to_string(), "hi");
        assert_eq!(AttributeValue::Int(-1).to_string(), "-1");
        assert_eq!(AttributeValue::Float(2.5).to_string(), "2.5");
        assert_eq!(AttributeValue::Bool(true).to_string(), "true");
    }

    #[test]
    fn equality_works() {
        assert_eq!(AttributeValue::Int(1), AttributeValue::Int(1));
        assert_ne!(AttributeValue::Int(1), AttributeValue::Int(2));
        assert_eq!(
            AttributeValue::string("a"),
            AttributeValue::string_owned("a".to_string())
        );
    }

    #[test]
    fn display_string_owned_variant() {
        // 对标 OpenTelemetry: owned string 属性的 Display 输出
        let v = AttributeValue::string_owned("owned value".to_string());
        assert_eq!(v.to_string(), "owned value");
    }

    #[test]
    fn display_float_negative() {
        let v = AttributeValue::Float(-3.14);
        assert_eq!(v.to_string(), "-3.14");
    }

    #[test]
    fn display_bool_false() {
        let v = AttributeValue::Bool(false);
        assert_eq!(v.to_string(), "false");
    }

    #[test]
    fn as_int_returns_none_for_non_int_types() {
        // 对标 OpenTelemetry: 属性类型不匹配时返回 None
        // 覆盖行 63: as_int() 的 _ => None 分支
        assert!(AttributeValue::string("s").as_int().is_none());
        assert!(AttributeValue::Float(1.0).as_int().is_none());
        assert!(AttributeValue::Bool(true).as_int().is_none());
    }

    #[test]
    fn as_float_returns_none_for_non_float_types() {
        // 覆盖行 72: as_float() 的 _ => None 分支
        assert!(AttributeValue::string("s").as_float().is_none());
        assert!(AttributeValue::Int(42).as_float().is_none());
        assert!(AttributeValue::Bool(false).as_float().is_none());
    }

    #[test]
    fn as_bool_returns_none_for_non_bool_types() {
        // 覆盖行 81: as_bool() 的 _ => None 分支
        assert!(AttributeValue::string("s").as_bool().is_none());
        assert!(AttributeValue::Int(1).as_bool().is_none());
        assert!(AttributeValue::Float(1.0).as_bool().is_none());
    }
}
