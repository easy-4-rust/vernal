//! ClassUtils — 类工具。
/// 类工具。
pub struct ClassUtils;
impl ClassUtils {
    pub fn get_short_name(class_name: &str) -> &str {
        class_name.rsplit("::").next().unwrap_or(class_name)
    }
    pub fn get_package_name(class_name: &str) -> &str {
        class_name.rsplitn(2, "::").nth(1).unwrap_or("")
    }
}
