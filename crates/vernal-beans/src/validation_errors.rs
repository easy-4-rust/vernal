//! ValidationErrors — 验证错误集合。
/// 验证错误集合。
#[derive(Clone, Debug, Default)]
pub struct ValidationErrors {
    pub errors: Vec<String>,
}
impl ValidationErrors {
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, error: impl Into<String>) { self.errors.push(error.into()); }
    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }
    pub fn count(&self) -> usize { self.errors.len() }
}
