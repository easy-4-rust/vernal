//! Profile — Spring 风格的 Profile。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Profile {
    pub name: String,
    pub is_default: bool,
}
impl Profile {
    pub fn new(name: impl Into<String>) -> Self { Self { name: name.into(), is_default: false } }
    pub fn new_default(name: impl Into<String>) -> Self { Self { name: name.into(), is_default: true } }
    pub fn name(&self) -> &str { &self.name }
    pub fn is_default(&self) -> bool { self.is_default }
    pub fn set_default(&mut self, default: bool) { self.is_default = default; }
}
