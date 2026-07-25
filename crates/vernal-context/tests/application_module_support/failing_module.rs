//! 配置失败应用模块测试对象。

use std::{io, sync::Arc};

use vernal_context::{
    ApplicationModule, ApplicationModuleRegistrar, MapPropertySource, PropertySource,
};
use vernal_core::BoxError;
use vernal_ioc::ComponentDefinition;

use super::ModuleProbe;

/// 在暂存部分贡献后主动失败，用于验证配置阶段整体回滚与错误脱敏。
pub struct FailingModule {
    probe: Arc<ModuleProbe>,
}

impl FailingModule {
    /// 创建携带尚未注册观测器的失败模块。
    #[must_use]
    pub fn new(probe: Arc<ModuleProbe>) -> Self {
        Self { probe }
    }
}

impl ApplicationModule for FailingModule {
    /// 返回稳定模块名。
    fn name(&self) -> &'static str {
        "failing.module"
    }

    /// 暂存组件和属性来源后返回包含敏感测试文本的错误。
    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        let source: Arc<dyn PropertySource> = Arc::new(MapPropertySource::new(
            "failing-source",
            [("failing.enabled", "true")],
        )?);
        registrar
            .register(ComponentDefinition::shared_arc(self.probe))
            .property_source_last(source);
        Err(Box::new(io::Error::other(
            "secret-module-configuration-response",
        )))
    }
}
