//! DescriptiveResource — 带描述的资源。
use crate::resource::Resource;

/// 带描述的资源。
#[derive(Clone, Debug)]
pub struct DescriptiveResource { pub name: String, pub desc: String }
impl DescriptiveResource {
    pub fn new(name: impl Into<String>, desc: impl Into<String>) -> Self {
        Self { name: name.into(), desc: desc.into() }
    }
}
impl Resource for DescriptiveResource {
    fn description(&self) -> &str { &self.desc }
}
