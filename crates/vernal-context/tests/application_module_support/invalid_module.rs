//! 非法名称应用模块测试对象。

use vernal_context::{ApplicationModule, ApplicationModuleRegistrar};
use vernal_core::BoxError;

/// 返回带空白名称，用于验证配置函数执行前的身份校验。
pub struct InvalidModule;

impl ApplicationModule for InvalidModule {
    /// 故意返回非法静态名称。
    fn name(&self) -> &'static str {
        "invalid module"
    }

    /// 名称校验应阻止本方法执行。
    fn configure(self, _registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError> {
        unreachable!("invalid module name must fail before configuration")
    }
}
