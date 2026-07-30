//! ObjectUtils — 对象工具。
/// 对象工具。
pub struct ObjectUtils;
impl ObjectUtils {
    pub fn is_null(obj: &dyn std::any::Any) -> bool { obj.is::<()>() }
    pub fn identity_string(obj: &dyn std::any::Any) -> String {
        format!("{:p}", obj as *const dyn std::any::Any)
    }
}
