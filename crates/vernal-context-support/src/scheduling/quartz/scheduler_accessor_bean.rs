//! 调度器访问器 Bean — 对标 `SchedulerAccessorBean`。

/// 调度器访问器 Bean。
pub struct SchedulerAccessorBean {
    scheduler_name: Option<String>,
    overwrite_existing_jobs: bool,
}
impl SchedulerAccessorBean {
    /// 创建调度器访问器 Bean。
    pub fn new() -> Self {
        Self {
            scheduler_name: None,
            overwrite_existing_jobs: false,
        }
    }
    /// 设置调度器名称。
    pub fn set_scheduler_name(&mut self, name: String) {
        self.scheduler_name = Some(name);
    }
    /// 获取调度器名称。
    pub fn scheduler_name(&self) -> Option<&str> {
        self.scheduler_name.as_deref()
    }
    /// 设置是否覆盖已存在的任务。
    pub fn set_overwrite_existing_jobs(&mut self, overwrite: bool) {
        self.overwrite_existing_jobs = overwrite;
    }
    /// 获取是否覆盖已存在的任务。
    pub fn overwrite_existing_jobs(&self) -> bool {
        self.overwrite_existing_jobs
    }
    /// 属性设置完成后的回调。
    pub fn after_properties_set(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
impl Default for SchedulerAccessorBean {
    fn default() -> Self {
        Self::new()
    }
}
