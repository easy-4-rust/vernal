//! 完整应用模块测试对象。

use std::sync::Arc;

use vernal_aop::Operation;
use vernal_context::{
    ApplicationModule, ApplicationModuleRegistrar, MapPropertySource, PropertySource,
};
use vernal_core::BoxError;
use vernal_ioc::ComponentDefinition;

use super::{ModuleProbe, ModuleService};

/// 模拟消费方 Bridge 一次安装组件、生命周期、AOP、Operation 与配置来源。
pub struct CommerceModule {
    probe: Arc<ModuleProbe>,
    operation: Operation,
}

impl CommerceModule {
    /// 创建绑定指定观测器和业务操作的模块。
    #[must_use]
    pub fn new(probe: Arc<ModuleProbe>, operation: Operation) -> Self {
        Self { probe, operation }
    }
}

impl ApplicationModule for CommerceModule {
    /// 使用稳定短名称参与重复模块诊断。
    fn name(&self) -> &'static str {
        "commerce.core"
    }

    /// 在隔离 Registrar 中声明完整模块贡献。
    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        let operation = self.operation;
        let pointcut_operation = operation.clone();
        let source: Arc<dyn PropertySource> = Arc::new(MapPropertySource::new(
            "commerce-module",
            [("commerce.enabled", "true")],
        )?);

        registrar
            .register(ComponentDefinition::shared_arc(self.probe))
            .register_advisor_component::<ModuleService, _>(
                move |candidate: &Operation| candidate == &pointcut_operation,
                0,
            )
            .lifecycle::<ModuleService>()
            .operation(operation)
            .property_source_last(source)
            .active_profile("commerce");
        Ok(())
    }
}
