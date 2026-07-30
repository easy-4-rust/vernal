//! HttpStatus — HTTP 状态码。
/// HTTP 状态码。
#[derive(Clone, Debug, PartialEq)]
pub struct HttpStatus {
    pub code: u16,
    pub reason: String,
}
impl HttpStatus {
    pub fn new(code: u16, reason: impl Into<String>) -> Self { Self { code, reason: reason.into() } }
    pub fn ok() -> Self { Self::new(200, "OK") }
    pub fn not_found() -> Self { Self::new(404, "Not Found") }
    pub fn internal_server_error() -> Self { Self::new(500, "Internal Server Error") }
    pub fn is_success(&self) -> bool { self.code >= 200 && self.code < 300 }
    pub fn is_error(&self) -> bool { self.code >= 400 }
}
