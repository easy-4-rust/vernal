//! Scheduled — 调度注解处理。
/// 调度配置。
#[derive(Clone, Debug)]
pub struct ScheduledConfig {
    pub cron: Option<String>,
    pub fixed_delay: Option<u64>,
    pub fixed_rate: Option<u64>,
}
impl ScheduledConfig {
    pub fn new() -> Self { Self { cron: None, fixed_delay: None, fixed_rate: None } }
    pub fn with_cron(mut self, cron: impl Into<String>) -> Self { self.cron = Some(cron.into()); self }
    pub fn with_fixed_delay(mut self, delay: u64) -> Self { self.fixed_delay = Some(delay); self }
}
