//! UrlResourceImpl — URL 资源实现。
use crate::resource::Resource;

/// URL 资源实现。
#[derive(Clone, Debug)]
pub struct UrlResourceImpl {
    pub url: String,
}
impl UrlResourceImpl {
    pub fn new(url: impl Into<String>) -> Self { Self { url: url.into() } }
    pub fn url_str(&self) -> &str { &self.url }
    pub fn scheme(&self) -> Option<&str> { self.url.splitn(2, ':').next() }
    pub fn filename(&self) -> Option<&str> { self.url.rsplit('/').next() }
}
impl Resource for UrlResourceImpl {
    fn description(&self) -> &str { &self.url }
}
