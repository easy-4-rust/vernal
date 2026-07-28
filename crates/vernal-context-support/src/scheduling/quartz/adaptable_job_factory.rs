//! 可适配任务工厂 — 对标 `AdaptableJobFactory`。

use super::quartz_job_bean::{JobExecutionContext, QuartzJob};

/// 可适配任务工厂 trait。
pub trait AdaptableJobFactory: Send + Sync {
    fn new_job(&self, job_class: &str) -> Result<Box<dyn QuartzJob>, String>;
    fn adapt_job(
        &self,
        job_object: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<Box<dyn QuartzJob>, String>;
}
