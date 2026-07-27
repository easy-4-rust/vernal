//! MIME 类型工具常量。
//!
//! 对标 Spring `org.springframework.util.MimeTypeUtils`。

use super::MimeType;

/// MIME 类型工具常量。
///
/// 对标 Spring `MimeTypeUtils`。
pub struct MimeTypeUtils;

impl MimeTypeUtils {
    /// `*/*`(任意类型)。对标 Spring `ALL`。
    pub fn all() -> MimeType {
        MimeType::new("*", "*")
    }

    /// `application/json`。对标 Spring `APPLICATION_JSON`。
    pub fn application_json() -> MimeType {
        MimeType::new("application", "json")
    }

    /// `application/xml`。对标 Spring `APPLICATION_XML`。
    pub fn application_xml() -> MimeType {
        MimeType::new("application", "xml")
    }

    /// `application/octet-stream`。对标 Spring `APPLICATION_OCTET_STREAM`。
    pub fn application_octet_stream() -> MimeType {
        MimeType::new("application", "octet-stream")
    }

    /// `application/x-www-form-urlencoded`。对标 Spring `APPLICATION_FORM_URLENCODED`。
    pub fn application_form_urlencoded() -> MimeType {
        MimeType::new("application", "x-www-form-urlencoded")
    }

    /// `application/pdf`。
    pub fn application_pdf() -> MimeType {
        MimeType::new("application", "pdf")
    }

    /// `text/html`。对标 Spring `TEXT_HTML`。
    pub fn text_html() -> MimeType {
        MimeType::new("text", "html")
    }

    /// `text/plain`。对标 Spring `TEXT_PLAIN`。
    pub fn text_plain() -> MimeType {
        MimeType::new("text", "plain")
    }

    /// `text/xml`。对标 Spring `TEXT_XML`。
    pub fn text_xml() -> MimeType {
        MimeType::new("text", "xml")
    }

    /// `text/css`。
    pub fn text_css() -> MimeType {
        MimeType::new("text", "css")
    }

    /// `text/javascript`。
    pub fn text_javascript() -> MimeType {
        MimeType::new("text", "javascript")
    }

    /// `image/jpeg`。
    pub fn image_jpeg() -> MimeType {
        MimeType::new("image", "jpeg")
    }

    /// `image/png`。
    pub fn image_png() -> MimeType {
        MimeType::new("image", "png")
    }

    /// `image/gif`。
    pub fn image_gif() -> MimeType {
        MimeType::new("image", "gif")
    }

    /// `multipart/form-data`。
    pub fn multipart_form_data() -> MimeType {
        MimeType::new("multipart", "form-data")
    }

    /// 把字符串解析为 MimeType。
    ///
    /// 对标 Spring `MimeTypeUtils.parseMimeType(String)`。
    pub fn parse_mime_type(s: &str) -> Result<MimeType, super::InvalidMimeType> {
        MimeType::parse(s)
    }

    /// 把字符串解析为多个 MimeType(用 `,` 分隔)。
    ///
    /// 对标 Spring `MimeTypeUtils.parseMimeTypes(String)`。
    pub fn parse_mime_types(s: &str) -> Vec<MimeType> {
        s.split(',')
            .filter_map(|item| MimeType::parse(item.trim()).ok())
            .collect()
    }

    /// 把多个 MimeType 拼接为字符串。
    ///
    /// 对标 Spring `MimeTypeUtils.toString(Collection<MimeType>)`。
    pub fn mime_types_to_string(types: &[MimeType]) -> String {
        types
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_is_wildcard() {
        let m = MimeTypeUtils::all();
        assert_eq!(m.type_(), "*");
        assert_eq!(m.subtype(), "*");
    }

    #[test]
    fn application_json_constant() {
        let m = MimeTypeUtils::application_json();
        assert_eq!(m.type_(), "application");
        assert_eq!(m.subtype(), "json");
    }

    #[test]
    fn text_html_constant() {
        let m = MimeTypeUtils::text_html();
        assert_eq!(m.type_(), "text");
        assert_eq!(m.subtype(), "html");
    }

    #[test]
    fn all_constants_return_correct_values() {
        assert_eq!(
            MimeTypeUtils::application_xml().to_string(),
            "application/xml"
        );
        assert_eq!(
            MimeTypeUtils::application_octet_stream().to_string(),
            "application/octet-stream"
        );
        assert_eq!(
            MimeTypeUtils::application_form_urlencoded().to_string(),
            "application/x-www-form-urlencoded"
        );
        assert_eq!(
            MimeTypeUtils::application_pdf().to_string(),
            "application/pdf"
        );
        assert_eq!(MimeTypeUtils::text_plain().to_string(), "text/plain");
        assert_eq!(MimeTypeUtils::text_xml().to_string(), "text/xml");
        assert_eq!(MimeTypeUtils::text_css().to_string(), "text/css");
        assert_eq!(
            MimeTypeUtils::text_javascript().to_string(),
            "text/javascript"
        );
        assert_eq!(MimeTypeUtils::image_jpeg().to_string(), "image/jpeg");
        assert_eq!(MimeTypeUtils::image_png().to_string(), "image/png");
        assert_eq!(MimeTypeUtils::image_gif().to_string(), "image/gif");
        assert_eq!(
            MimeTypeUtils::multipart_form_data().to_string(),
            "multipart/form-data"
        );
    }

    #[test]
    fn parse_mime_type_basic() {
        let m = MimeTypeUtils::parse_mime_type("application/json").unwrap();
        assert_eq!(m.subtype(), "json");
    }

    #[test]
    fn parse_mime_types_multi() {
        let types = MimeTypeUtils::parse_mime_types("application/json, text/html");
        assert_eq!(types.len(), 2);
        assert_eq!(types[0].subtype(), "json");
        assert_eq!(types[1].subtype(), "html");
    }

    #[test]
    fn parse_mime_types_with_whitespace() {
        let types = MimeTypeUtils::parse_mime_types("application/json , text/html");
        assert_eq!(types.len(), 2);
    }

    #[test]
    fn mime_types_to_string_round_trip() {
        let types = vec![
            MimeTypeUtils::application_json(),
            MimeTypeUtils::text_html(),
        ];
        let s = MimeTypeUtils::mime_types_to_string(&types);
        let parsed = MimeTypeUtils::parse_mime_types(&s);
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn mime_types_to_string_empty_returns_empty() {
        let types: Vec<MimeType> = vec![];
        assert_eq!(MimeTypeUtils::mime_types_to_string(&types), "");
    }
}
