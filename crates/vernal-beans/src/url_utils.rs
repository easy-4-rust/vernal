//! UrlUtils — URL 工具。
/// URL 工具。
pub struct UrlUtils;
impl UrlUtils {
    pub fn get_filename(url: &str) -> Option<&str> { url.rsplit('/').next() }
    pub fn get_protocol(url: &str) -> Option<&str> { url.splitn(2, ':').next() }
}
