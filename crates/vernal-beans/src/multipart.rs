//! Multipart — 多部分文件。
/// 多部分文件。
#[derive(Clone, Debug)]
pub struct MultipartFile {
    pub name: String,
    pub content_type: Option<String>,
    pub data: Vec<u8>,
}
impl MultipartFile {
    pub fn new(name: impl Into<String>) -> Self { Self { name: name.into(), content_type: None, data: Vec::new() } }
    pub fn with_data(mut self, data: Vec<u8>) -> Self { self.data = data; self }
    pub fn with_content_type(mut self, ct: impl Into<String>) -> Self { self.content_type = Some(ct.into()); self }
    pub fn name(&self) -> &str { &self.name }
    pub fn data(&self) -> &[u8] { &self.data }
    pub fn size(&self) -> usize { self.data.len() }
}
