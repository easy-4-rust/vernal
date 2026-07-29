//! ClassPathResource — 类路径资源。
use crate::resource::Resource;

/// 类路径资源。
#[derive(Clone, Debug)]
pub struct ClassPathResource { pub path: String, pub class_name: Option<String> }
impl ClassPathResource {
    pub fn new(path: impl Into<String>) -> Self { Self { path: path.into(), class_name: None } }
    pub fn with_class_name(path: impl Into<String>, class_name: impl Into<String>) -> Self {
        Self { path: path.into(), class_name: Some(class_name.into()) }
    }
    pub fn get_path(&self) -> &str { &self.path }
}
impl Resource for ClassPathResource {
    fn description(&self) -> &str { &self.path }
}
