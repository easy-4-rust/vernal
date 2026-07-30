//! Conditional — 条件注解处理。
/// 条件注册器。
#[derive(Clone, Debug, Default)]
pub struct ConditionalRegistry;
impl ConditionalRegistry {
    pub fn new() -> Self { Self }
    pub fn register_condition(&self, _bean_name: &str, _condition_name: &str) {}
}
