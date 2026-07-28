//! 调度器访问器 Bean — 对标 `SchedulerAccessorBean`。

use std::sync::Arc;

/// 调度器访问器 Bean。
pub struct SchedulerAccessorBean {
    scheduler_name: Option<String>,
    overwrite_existing_jobs: bool,
}
impl SchedulerAccessorBean {
    pub fn new() -> Self {
        Self {
            scheduler_name: None,
            overwrite_existing_jobs: false,
        }
    }
    pub fn set_scheduler_name(&mut self, name: String) {
        self.scheduler_name = Some(name);
    }
    pub fn scheduler_name(&self) -> Option<&str> {
        self.scheduler_name.as_deref()
    }
    pub fn set_overwrite_existing_jobs(&mut self, overwrite: bool) {
        self.overwrite_existing_jobs = overwrite;
    }
    pub fn overwrite_existing_jobs(&self) -> bool {
        self.overwrite_existing_jobs
    }
    pub fn after_properties_set(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
impl Default for SchedulerAccessorBean {
    fn default() -> Self {
        Self::new()
    }
}
