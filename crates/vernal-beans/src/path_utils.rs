//! PathUtils — 路径工具。
/// 路径工具。
pub struct PathUtils;
impl PathUtils {
    pub fn clean_path(path: &str) -> String {
        path.replace("//", "/")
    }
    pub fn get_filename(path: &str) -> Option<&str> {
        path.rsplit('/').next()
    }
    pub fn get_extension(path: &str) -> Option<&str> {
        path.rsplit('.').next()
    }
}
