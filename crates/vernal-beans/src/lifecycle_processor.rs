use crate::{
    lifecycle::{Lifecycle, LifecycleProcessor as LifecycleProcessorTrait},
    lifecycle_group::LifecycleGroup,
};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct DefaultLifecycleProcessor {
    group: Mutex<LifecycleGroup>,
}
impl DefaultLifecycleProcessor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_lifecycle(&self, lifecycle: Arc<dyn Lifecycle>, phase: i32, auto_startup: bool) {
        self.group
            .lock()
            .expect("lifecycle lock poisoned")
            .add(lifecycle, phase, auto_startup);
    }
}
impl Lifecycle for DefaultLifecycleProcessor {
    fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.group
            .lock()
            .expect("lifecycle lock poisoned")
            .start(false)
    }
    fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.group.lock().expect("lifecycle lock poisoned").stop()
    }
    fn is_running(&self) -> bool {
        self.group
            .lock()
            .expect("lifecycle lock poisoned")
            .is_running()
    }
}
impl LifecycleProcessorTrait for DefaultLifecycleProcessor {
    fn on_refresh(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.group
            .lock()
            .expect("lifecycle lock poisoned")
            .start(true)
    }
}
pub type LifecycleProcessor = DefaultLifecycleProcessor;
