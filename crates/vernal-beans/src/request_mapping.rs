//! RequestMapping — 请求映射配置。
/// 请求映射配置。
#[derive(Clone, Debug)]
pub struct RequestMapping {
    pub path: String,
    pub method: String,
}
impl RequestMapping {
    pub fn new(path: impl Into<String>, method: impl Into<String>) -> Self {
        Self { path: path.into(), method: method.into() }
    }
    pub fn path(&self) -> &str { &self.path }
    pub fn method(&self) -> &str { &self.method }
}
