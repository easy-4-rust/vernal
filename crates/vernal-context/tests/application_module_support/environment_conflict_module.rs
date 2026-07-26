//! 环境冲突应用模块测试对象。

use std::sync::Arc;

use vernal_aop::Operation;
use vernal_beans::ComponentDefinition;
use vernal_context::{
    ApplicationModule, ApplicationModuleRegistrar, MapPropertySource, PropertySource,
};
use vernal_core::BoxError;

use super::ModuleProbe;

/// 同时暂存组件、Operation 与冲突属性来源，用于验证环境预检早于真实提交。
pub struct EnvironmentConflictModule {
    probe: Arc<ModuleProbe>,
    operation: Operation,
}

impl EnvironmentConflictModule {
    /// 创建使用指定未注册观测器和操作的冲突模块。
    #[must_use]
    pub fn new(probe: Arc<ModuleProbe>, operation: Operation) -> Self {
        Self { probe, operation }
    }
}

impl ApplicationModule for EnvironmentConflictModule {
    /// 返回稳定模块名。
    fn name(&self) -> &'static str {
        "environment.conflict"
    }

    /// 声明一个预期与应用现有来源重名的完整暂存集合。
    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        let source: Arc<dyn PropertySource> = Arc::new(MapPropertySource::new(
            "shared-source",
            [("module.value", "hidden")],
        )?);
        registrar
            .register(ComponentDefinition::shared_arc(self.probe))
            .operation(self.operation)
            .property_source_last(source);
        Ok(())
    }
}
