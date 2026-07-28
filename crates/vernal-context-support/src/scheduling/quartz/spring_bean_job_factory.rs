//! Spring Bean 任务工厂 — 对标 `SpringBeanJobFactory`。

use super::adaptable_job_factory::AdaptableJobFactory;
use super::quartz_job_bean::QuartzJob;

/// Spring Bean 任务工厂。
pub struct SpringBeanJobFactory {
    ignored_unknown_properties: Vec<String>,
}
impl SpringBeanJobFactory {
    pub fn new() -> Self {
        Self {
            ignored_unknown_properties: Vec::new(),
        }
    }
    pub fn set_ignored_unknown_properties(&mut self, props: Vec<String>) {
        self.ignored_unknown_properties = props;
    }
    pub fn ignored_unknown_properties(&self) -> &[String] {
        &self.ignored_unknown_properties
    }
}
impl Default for SpringBeanJobFactory {
    fn default() -> Self {
        Self::new()
    }
}
