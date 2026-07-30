//! ScopeConfigurer — 作用域配置器。
#[derive(Clone, Debug, Default)]
pub struct ScopeConfigurer;
impl ScopeConfigurer {
    pub fn new() -> Self { Self }
    pub fn configure(&self, _scope_name: &str) {}
}
