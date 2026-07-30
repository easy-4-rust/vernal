//! RedirectView — 重定向视图。
/// 重定向视图。
#[derive(Clone, Debug)]
pub struct RedirectView {
    pub url: String,
    pub status_code: u16,
}
impl RedirectView {
    pub fn new(url: impl Into<String>) -> Self { Self { url: url.into(), status_code: 302 } }
    pub fn with_status_code(mut self, code: u16) -> Self { self.status_code = code; self }
    pub fn get_url(&self) -> &str { &self.url }
    pub fn get_status_code(&self) -> u16 { self.status_code }
}
