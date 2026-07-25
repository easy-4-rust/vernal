//! 空应用模块测试对象。

use vernal_context::{ApplicationModule, ApplicationModuleRegistrar};
use vernal_core::BoxError;

/// 不声明任何贡献，用于验证空模块 fail-closed。
pub struct EmptyModule;

impl ApplicationModule for EmptyModule {
    /// 返回稳定模块名。
    fn name(&self) -> &'static str {
        "empty.module"
    }

    /// 成功返回但不写入 Registrar。
    fn configure(self, _registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        Ok(())
    }
}
