//! RequestBody — 请求体处理。
/// 请求体处理。
#[derive(Clone, Debug)]
pub struct RequestBody {
    pub data: Vec<u8>,
    pub content_type: String,
}
impl RequestBody {
    pub fn new(data: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self { data, content_type: content_type.into() }
    }
    pub fn data(&self) -> &[u8] { &self.data }
    pub fn content_type(&self) -> &str { &self.content_type }
    pub fn size(&self) -> usize { self.data.len() }
}
