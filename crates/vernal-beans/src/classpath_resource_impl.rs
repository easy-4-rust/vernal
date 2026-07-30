//! ClassPathResourceImpl — 类路径资源实现。
use crate::resource::Resource;

/// 类路径资源实现。
#[derive(Clone, Debug)]
pub struct ClassPathResourceImpl {
    pub path: String,
}
impl ClassPathResourceImpl {
    pub fn new(path: impl Into<String>) -> Self { Self { path: path.into() } }
    pub fn get_path(&self) -> &str { &self.path }
}
impl Resource for ClassPathResourceImpl {
    fn description(&self) -> &str { &self.path }
}
