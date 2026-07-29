//! InputStreamResource — 输入流资源。
use crate::resource::Resource;

/// 输入流资源。
#[derive(Clone, Debug)]
pub struct InputStreamResource { pub data: Vec<u8>, pub desc: String }
impl InputStreamResource {
    pub fn new(data: Vec<u8>, desc: impl Into<String>) -> Self { Self { data, desc: desc.into() } }
    pub fn content_len(&self) -> usize { self.data.len() }
    pub fn is_open(&self) -> bool { true }
}
impl Resource for InputStreamResource {
    fn exists(&self) -> bool { true }
    fn description(&self) -> &str { &self.desc }
    fn read_to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> { Ok(self.data.clone()) }
}
