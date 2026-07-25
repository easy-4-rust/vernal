//! Runner 应用模块测试对象。

use std::sync::Arc;

use vernal_context::{ApplicationModule, ApplicationModuleRegistrar};
use vernal_core::BoxError;
use vernal_ioc::ComponentDefinition;

use super::FirstRunner;

/// 模拟消费方 Bridge 原子贡献 Runner 定义和执行声明。
pub struct RunnerApplicationModule {
    runner: Arc<FirstRunner>,
}

impl RunnerApplicationModule {
    /// 创建持有预构造 Runner 的模块。
    pub fn new(runner: Arc<FirstRunner>) -> Self {
        Self { runner }
    }
}

impl ApplicationModule for RunnerApplicationModule {
    fn name(&self) -> &'static str {
        "test.application-runner"
    }

    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        registrar
            .register(ComponentDefinition::shared_arc(self.runner))
            .application_runner::<FirstRunner>();
        Ok(())
    }
}
