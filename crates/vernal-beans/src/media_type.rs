//! MediaType — 媒体类型。
/// 媒体类型。
#[derive(Clone, Debug, PartialEq)]
pub struct MediaType {
    pub type_name: String,
    pub subtype: String,
    pub parameters: std::collections::HashMap<String, String>,
}
impl MediaType {
    pub fn new(type_name: impl Into<String>, subtype: impl Into<String>) -> Self {
        Self { type_name: type_name.into(), subtype: subtype.into(), parameters: std::collections::HashMap::new() }
    }
    pub fn application_json() -> Self { Self::new("application", "json") }
    pub fn text_html() -> Self { Self::new("text", "html") }
    pub fn text_plain() -> Self { Self::new("text", "plain") }
    pub fn to_string(&self) -> String { format!("{}/{}", self.type_name, self.subtype) }
    pub fn matches(&self, other: &MediaType) -> bool { self.type_name == other.type_name && self.subtype == other.subtype }
}
