//! DataBinding — 数据绑定。
/// 数据绑定结果。
#[derive(Clone, Debug)]
pub struct DataBinder {
    pub target_name: String,
    pub target_type: String,
}
impl DataBinder {
    pub fn new(target_name: impl Into<String>, target_type: impl Into<String>) -> Self {
        Self { target_name: target_name.into(), target_type: target_type.into() }
    }
    pub fn bind(&self) {}
}
