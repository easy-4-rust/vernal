//! 定义冲突应用模块测试对象。

use std::sync::Arc;

use vernal_aop::Operation;
use vernal_context::{
    ApplicationModule, ApplicationModuleRegistrar, MapPropertySource, PropertySource,
};
use vernal_core::BoxError;
use vernal_beans::ComponentDefinition;

use super::ModuleProbe;

/// 在冲突与有效两种模式间复用同一模块名，验证失败不占用身份或提交环境。
pub struct DefinitionConflictModule {
    probe: Option<Arc<ModuleProbe>>,
    operation: Option<Operation>,
}

impl DefinitionConflictModule {
    /// 创建会重复注册观测器定义的冲突模式。
    #[must_use]
    pub fn conflicting(probe: Arc<ModuleProbe>, operation: Operation) -> Self {
        Self {
            probe: Some(probe),
            operation: Some(operation),
        }
    }

    /// 创建只安装同名属性来源的有效重试模式。
    #[must_use]
    pub const fn valid_retry() -> Self {
        Self {
            probe: None,
            operation: None,
        }
    }
}

impl ApplicationModule for DefinitionConflictModule {
    /// 两种模式故意共享名称，以验证失败安装不保留名称。
    fn name(&self) -> &'static str {
        "definition.retry"
    }

    /// 暂存属性来源，并在冲突模式额外声明重复 Definition 与 Operation。
    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        let source: Arc<dyn PropertySource> = Arc::new(MapPropertySource::new(
            "definition-retry-source",
            [("definition.retry", "true")],
        )?);
        registrar.property_source_last(source);
        if let Some(probe) = self.probe {
            registrar.register(ComponentDefinition::shared_arc(probe));
        }
        if let Some(operation) = self.operation {
            registrar.operation(operation);
        }
        Ok(())
    }
}
