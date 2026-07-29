use crate::lifecycle::Lifecycle;
use std::sync::Arc;

pub struct LifecycleMember {
    pub phase: i32,
    pub auto_startup: bool,
    pub lifecycle: Arc<dyn Lifecycle>,
}
#[derive(Default)]
pub struct LifecycleGroup {
    members: Vec<LifecycleMember>,
}
impl LifecycleGroup {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, lifecycle: Arc<dyn Lifecycle>, phase: i32, auto_startup: bool) {
        self.members.push(LifecycleMember {
            phase,
            auto_startup,
            lifecycle,
        });
    }
    pub fn len(&self) -> usize {
        self.members.len()
    }
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
    pub fn start(&self, auto_only: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut members: Vec<_> = self.members.iter().collect();
        members.sort_by_key(|m| m.phase);
        for m in members {
            if !auto_only || m.auto_startup {
                m.lifecycle.start()?;
            }
        }
        Ok(())
    }
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut members: Vec<_> = self.members.iter().collect();
        members.sort_by_key(|m| std::cmp::Reverse(m.phase));
        for m in members {
            if m.lifecycle.is_running() {
                m.lifecycle.stop()?;
            }
        }
        Ok(())
    }
    pub fn is_running(&self) -> bool {
        self.members.iter().any(|m| m.lifecycle.is_running())
    }
}
