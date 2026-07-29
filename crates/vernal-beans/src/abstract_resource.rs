//! AbstractResource — 资源基类实现。
use crate::resource::Resource;

/// 资源基类实现。
#[derive(Clone, Debug)]
pub struct AbstractResource {
    pub name: String,
    pub data: Vec<u8>,
}
impl AbstractResource {
    pub fn new(name: impl Into<String>) -> Self { Self { name: name.into(), data: Vec::new() } }
    pub fn from_bytes(name: impl Into<String>, data: Vec<u8>) -> Self { Self { name: name.into(), data } }
}
impl Resource for AbstractResource {
    fn exists(&self) -> bool { true }
    fn is_readable(&self) -> bool { true }
    fn description(&self) -> &str { &self.name }
    fn read_to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> { Ok(self.data.clone()) }
}
