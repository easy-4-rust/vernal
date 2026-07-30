//! ResponseBody — 响应体处理。
/// 响应体处理。
#[derive(Clone, Debug)]
pub struct ResponseBody {
    pub content: Vec<u8>,
    pub content_type: String,
}
impl ResponseBody {
    pub fn new(content: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self { content, content_type: content_type.into() }
    }
    pub fn content(&self) -> &[u8] { &self.content }
    pub fn content_type(&self) -> &str { &self.content_type }
    pub fn size(&self) -> usize { self.content.len() }
}
