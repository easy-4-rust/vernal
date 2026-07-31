//! 可适配任务工厂 — 对标 `AdaptableJobFactory`。

use super::quartz_job_bean::QuartzJob;

/// 可适配任务工厂 trait。
pub trait AdaptableJobFactory: Send + Sync {
    /// 根据任务类名创建新任务。
    fn new_job(&self, job_class: &str) -> Result<Box<dyn QuartzJob>, String>;
    /// 将任务对象适配为 Quartz 任务。
    fn adapt_job(
        &self,
        job_object: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Box<dyn QuartzJob>, String>;
}
