//! URL 类型转换器(feature = "convert-url")。
//!
//! 对标 Spring 的 `StringToURLConverter`,将字符串解析为 [`url::Url`]。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["convert-url"] }
//! ```

use super::{ConversionError, Convertible};

/// URL 转换器。
///
/// 对标 Spring `StringToURLConverter`。
pub struct UrlConverter;

impl Convertible for url::Url {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        url::Url::parse(value).map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: "Url",
            reason: format!("URL 解析失败: {e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_http_url() {
        let url = url::Url::from_str_value("https://example.com/path?q=1").unwrap();
        assert_eq!(url.host_str(), Some("example.com"));
        assert_eq!(url.path(), "/path");
    }

    #[test]
    fn parses_ftp_url() {
        let url = url::Url::from_str_value("ftp://user:pass@ftp.example.com:21/file").unwrap();
        assert_eq!(url.scheme(), "ftp");
        assert_eq!(url.host_str(), Some("ftp.example.com"));
    }

    #[test]
    fn parses_file_url() {
        let url = url::Url::from_str_value("file:///etc/hosts").unwrap();
        assert_eq!(url.scheme(), "file");
    }

    #[test]
    fn rejects_relative_url() {
        // url crate 默认要求绝对 URL
        let err = url::Url::from_str_value("/relative/path").unwrap_err();
        assert_eq!(err.target_type, "Url");
    }

    #[test]
    fn rejects_empty_string() {
        let err = url::Url::from_str_value("").unwrap_err();
        assert_eq!(err.target_type, "Url");
    }

    #[test]
    fn rejects_invalid_scheme() {
        let err = url::Url::from_str_value("not a url").unwrap_err();
        assert_eq!(err.target_type, "Url");
        assert!(err.reason.contains("解析失败"));
    }

    #[test]
    fn url_via_conversion_service() {
        let url: url::Url =
            super::super::ConversionService::convert("https://rust-lang.org").unwrap();
        assert_eq!(url.host_str(), Some("rust-lang.org"));
    }
}
