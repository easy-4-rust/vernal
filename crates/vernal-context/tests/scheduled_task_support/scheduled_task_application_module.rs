//! 周期任务应用模块测试对象。

use std::sync::Arc;

use vernal_context::{ApplicationModule, ApplicationModuleRegistrar};
use vernal_core::BoxError;
use vernal_ioc::ComponentDefinition;

use super::FixedDelayProbeTask;

/// 模拟消费方 Bridge 原子贡献周期任务定义和执行声明。
pub struct ScheduledTaskApplicationModule {
    task: Arc<FixedDelayProbeTask>,
}

impl ScheduledTaskApplicationModule {
    /// 创建持有预构造周期任务的应用模块。
    pub fn new(task: Arc<FixedDelayProbeTask>) -> Self {
        Self { task }
    }
}

impl ApplicationModule for ScheduledTaskApplicationModule {
    fn name(&self) -> &'static str {
        "test.scheduled-task"
    }

    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        registrar
            .register(ComponentDefinition::shared_arc(self.task))
            .scheduled_task::<FixedDelayProbeTask>();
        Ok(())
    }
}
