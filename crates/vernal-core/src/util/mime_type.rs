//! MIME 类型(feature = "mime")。
//!
//! 对标 Spring `org.springframework.util.MimeType`。
//!
//! Spring 的 `MimeType` 是完整的 RFC 2045 抽象,基于 `mime` crate 简化实现。

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// MIME 类型表示。
///
/// 对标 Spring `MimeType`。基于 RFC 2045,包含 type / subtype / parameters。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeType {
    /// 主类型(如 `application` / `text`)
    r#type: String,
    /// 子类型(如 `json` / `html`)
    subtype: String,
    /// 参数(如 `charset=UTF-8`)
    parameters: HashMap<String, String>,
}

/// MIME 类型解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidMimeType {
    /// 原始输入
    pub input: String,
    /// 错误原因
    pub reason: String,
}

impl fmt::Display for InvalidMimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid mime type '{}': {}", self.input, self.reason)
    }
}

impl std::error::Error for InvalidMimeType {}

impl MimeType {
    /// 创建新的 MimeType。
    ///
    /// 对标 Spring `MimeType(String type, String subtype)`。
    #[must_use]
    pub fn new(type_name: impl Into<String>, subtype: impl Into<String>) -> Self {
        Self {
            r#type: type_name.into().to_ascii_lowercase(),
            subtype: subtype.into().to_ascii_lowercase(),
            parameters: HashMap::new(),
        }
    }

    /// 创建带参数的 MimeType。
    ///
    /// 对标 Spring `MimeType(String type, String subtype, Map<String, String> parameters)`。
    #[must_use]
    pub fn with_parameters(
        type_name: impl Into<String>,
        subtype: impl Into<String>,
        parameters: HashMap<String, String>,
    ) -> Self {
        Self {
            r#type: type_name.into().to_ascii_lowercase(),
            subtype: subtype.into().to_ascii_lowercase(),
            parameters,
        }
    }

    /// 获取主类型。
    #[must_use]
    pub fn type_(&self) -> &str {
        &self.r#type
    }

    /// 获取子类型。
    #[must_use]
    pub fn subtype(&self) -> &str {
        &self.subtype
    }

    /// 获取所有参数。
    #[must_use]
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }

    /// 获取指定参数。
    #[must_use]
    pub fn parameter(&self, name: &str) -> Option<&str> {
        self.parameters
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    /// 获取 charset 参数(对标 Spring `MimeType.getCharset()`)。
    #[must_use]
    pub fn charset(&self) -> Option<&str> {
        self.parameter("charset")
    }

    /// 是否为具体类型(对标 Spring `MimeType.isConcrete()`)。
    ///
    /// 子类型不为 `*` 的类型为具体类型。
    #[must_use]
    pub fn is_concrete(&self) -> bool {
        self.subtype != "*" && self.r#type != "*"
    }

    /// 检查是否与另一个 MimeType 兼容(对标 Spring `MimeType.isCompatibleWith(MimeType)`)。
    ///
    /// 兼容规则:
    /// - `*/*` 与任何类型兼容
    /// - `text/*` 与 `text/plain` 等兼容
    /// - `text/plain` 仅与 `text/plain` / `text/*` / `*/*` 兼容
    #[must_use]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        if self.r#type == "*" || other.r#type == "*" {
            return true;
        }
        if self.r#type != other.r#type {
            return false;
        }
        if self.subtype == "*" || other.subtype == "*" {
            return true;
        }
        self.subtype == other.subtype
    }

    /// 检查 type 和 subtype 是否相等(忽略参数,对标 Spring `equalsTypeAndSubtype`)。
    #[must_use]
    pub fn equals_type_and_subtype(&self, other: &Self) -> bool {
        self.r#type == other.r#type && self.subtype == other.subtype
    }

    /// 解析字符串为 MimeType。
    ///
    /// 对标 Spring `MimeType.valueOf(String)`。
    ///
    /// 支持格式:
    /// - `application/json`
    /// - `text/html;charset=UTF-8`
    /// - `application/vnd.api+json;version=1`
    pub fn parse(s: &str) -> Result<Self, InvalidMimeType> {
        let s = s.trim();
        if s.is_empty() {
            return Err(InvalidMimeType {
                input: s.to_string(),
                reason: "empty input".to_string(),
            });
        }

        // 分离主部分与参数
        let (main, params_str) = match s.find(';') {
            Some(idx) => (&s[..idx], Some(&s[idx + 1..])),
            None => (s, None),
        };

        // 解析 type/subtype
        let slash = main.find('/').ok_or_else(|| InvalidMimeType {
            input: s.to_string(),
            reason: "missing '/' separator".to_string(),
        })?;
        let type_name = main[..slash].trim();
        let subtype = main[slash + 1..].trim();

        if type_name.is_empty() || subtype.is_empty() {
            return Err(InvalidMimeType {
                input: s.to_string(),
                reason: "type or subtype is empty".to_string(),
            });
        }

        let mut parameters = HashMap::new();
        if let Some(params) = params_str {
            for pair in params.split(';') {
                let pair = pair.trim();
                if pair.is_empty() {
                    continue;
                }
                let eq = pair.find('=').ok_or_else(|| InvalidMimeType {
                    input: s.to_string(),
                    reason: format!("parameter missing '=': {pair}"),
                })?;
                let key = pair[..eq].trim().to_ascii_lowercase();
                let value = pair[eq + 1..].trim().trim_matches('"').to_string();
                parameters.insert(key, value);
            }
        }

        Ok(Self::with_parameters(type_name, subtype, parameters))
    }
}

impl fmt::Display for MimeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.r#type, self.subtype)?;
        for (k, v) in &self.parameters {
            write!(f, ";{k}={v}")?;
        }
        Ok(())
    }
}

impl FromStr for MimeType {
    type Err = InvalidMimeType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_lowercase_types() {
        let m = MimeType::new("APPLICATION", "JSON");
        assert_eq!(m.type_(), "application");
        assert_eq!(m.subtype(), "json");
    }

    #[test]
    fn parse_simple() {
        let m = MimeType::parse("application/json").unwrap();
        assert_eq!(m.type_(), "application");
        assert_eq!(m.subtype(), "json");
        assert!(m.is_concrete());
    }

    #[test]
    fn parse_with_parameters() {
        let m = MimeType::parse("text/html;charset=UTF-8").unwrap();
        assert_eq!(m.type_(), "text");
        assert_eq!(m.subtype(), "html");
        assert_eq!(m.charset(), Some("UTF-8"));
    }

    #[test]
    fn parse_with_quoted_value() {
        let m = MimeType::parse(r#"application/json;version="1.0""#).unwrap();
        assert_eq!(m.parameter("version"), Some("1.0"));
    }

    #[test]
    fn parse_multiple_parameters() {
        let m = MimeType::parse("multipart/form-data; boundary=abc; charset=utf-8").unwrap();
        assert_eq!(m.parameter("boundary"), Some("abc"));
        assert_eq!(m.parameter("charset"), Some("utf-8"));
    }

    #[test]
    fn parse_wildcard_type() {
        let m = MimeType::parse("*/*").unwrap();
        assert_eq!(m.type_(), "*");
        assert_eq!(m.subtype(), "*");
        assert!(!m.is_concrete());
    }

    #[test]
    fn parse_wildcard_subtype() {
        let m = MimeType::parse("text/*").unwrap();
        assert_eq!(m.type_(), "text");
        assert_eq!(m.subtype(), "*");
        assert!(!m.is_concrete());
    }

    #[test]
    fn parse_invalid_missing_slash() {
        let err = MimeType::parse("application").unwrap_err();
        assert!(err.reason.contains("missing"));
    }

    #[test]
    fn parse_invalid_empty() {
        assert!(MimeType::parse("").is_err());
        assert!(MimeType::parse("   ").is_err());
    }

    #[test]
    fn parse_invalid_empty_type() {
        assert!(MimeType::parse("/json").is_err());
        assert!(MimeType::parse("text/").is_err());
    }

    #[test]
    fn is_compatible_with_wildcard() {
        let wildcard = MimeType::parse("*/*").unwrap();
        let json = MimeType::parse("application/json").unwrap();
        assert!(wildcard.is_compatible_with(&json));
        assert!(json.is_compatible_with(&wildcard));
    }

    #[test]
    fn is_compatible_with_subtype_wildcard() {
        let text_any = MimeType::parse("text/*").unwrap();
        let text_html = MimeType::parse("text/html").unwrap();
        assert!(text_any.is_compatible_with(&text_html));
        assert!(text_html.is_compatible_with(&text_any));
    }

    #[test]
    fn is_compatible_with_exact_match() {
        let a = MimeType::parse("application/json").unwrap();
        let b = MimeType::parse("application/json").unwrap();
        assert!(a.is_compatible_with(&b));
    }

    #[test]
    fn is_compatible_with_different_types() {
        let a = MimeType::parse("application/json").unwrap();
        let b = MimeType::parse("text/html").unwrap();
        assert!(!a.is_compatible_with(&b));
    }

    #[test]
    fn is_compatible_with_different_subtypes() {
        let a = MimeType::parse("application/json").unwrap();
        let b = MimeType::parse("application/xml").unwrap();
        assert!(!a.is_compatible_with(&b));
    }

    #[test]
    fn equals_type_and_subtype_ignores_parameters() {
        let a = MimeType::parse("text/html;charset=utf-8").unwrap();
        let b = MimeType::parse("text/html;charset=ascii").unwrap();
        assert!(a.equals_type_and_subtype(&b));
        assert_ne!(a, b); // PartialEq 包含参数
    }

    #[test]
    fn display_round_trip() {
        let m = MimeType::parse("text/html;charset=UTF-8").unwrap();
        let s = m.to_string();
        let parsed = MimeType::parse(&s).unwrap();
        assert_eq!(m, parsed);
    }

    #[test]
    fn from_str_trait_works() {
        let m: MimeType = "application/json".parse().unwrap();
        assert_eq!(m.type_(), "application");
    }

    #[test]
    fn invalid_mime_type_displays_input() {
        let err = MimeType::parse("bad").unwrap_err();
        assert!(err.to_string().contains("bad"));
    }
}
