//! LifecycleProcessorImpl — 生命周期处理器实现。
use crate::lifecycle_processor::LifecycleProcessor;

/// 生命周期处理器实现。
#[derive(Clone, Debug, Default)]
pub struct LifecycleProcessorImpl;
impl LifecycleProcessorImpl {
    pub fn new() -> Self { Self }
}
impl LifecycleProcessor for LifecycleProcessorImpl {
    fn on_refresh(&self) {}
    fn on_close(&self) {}
}
