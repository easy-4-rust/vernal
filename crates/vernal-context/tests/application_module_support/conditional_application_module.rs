//! 条件应用模块合同测试对象。

use std::sync::Arc;

use vernal_context::{
    ApplicationModule, ApplicationModuleRegistrar, ConditionalComponentModule, MapPropertySource,
    PropertyCondition, PropertySource,
};
use vernal_core::BoxError;
use vernal_beans::ComponentDefinition;

use super::ModuleProbe;

/// 同时贡献 `PropertySource` 与一个或多个条件组件模块的显式应用模块。
///
/// 测试可以通过条件模块名称集合构造成功、非法名称和同批重复名称场景；外层模块名、
/// 属性来源名与组件身份保持不变，从而证明失败后这些身份都可以被完整重试。
pub struct ConditionalApplicationModule {
    probe: Arc<ModuleProbe>,
    conditional_names: Vec<&'static str>,
}

impl ConditionalApplicationModule {
    /// 创建携带指定条件模块名称集合的测试应用模块。
    #[must_use]
    pub fn new(probe: Arc<ModuleProbe>, conditional_names: Vec<&'static str>) -> Self {
        Self {
            probe,
            conditional_names,
        }
    }
}

impl ApplicationModule for ConditionalApplicationModule {
    fn name(&self) -> &'static str {
        "conditional.application"
    }

    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        let source: Arc<dyn PropertySource> = Arc::new(MapPropertySource::new(
            "conditional-application-source",
            [("bridge.enabled", "true")],
        )?);
        registrar.property_source_last(source);

        // 每个条件模块都读取由同一外层模块贡献的最终 Environment。Vernal 只在
        // build 时求值，但声明身份会在外层模块原子提交前完成统一预检。
        for conditional_name in self.conditional_names {
            let condition = PropertyCondition::having_value("bridge.enabled", "true")?;
            let mut conditional = ConditionalComponentModule::new(conditional_name, condition);
            conditional.register(ComponentDefinition::shared_arc(Arc::clone(&self.probe)));
            registrar.conditional(conditional);
        }
        Ok(())
    }
}
