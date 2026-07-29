//! 应用模块环境贡献对象。

use std::sync::Arc;

use crate::{ApplicationEnvironmentBuilder, EnvironmentError, PropertySource};

/// 保存一个尚未提交到应用 Environment 的确定性变更。
///
/// 贡献先应用到当前 Environment Builder 的隔离克隆；全部成功后才与组件模块一同
/// 提交，避免属性来源冲突时已经写入 Definition 或 Advisor。
pub enum ModuleEnvironmentContribution {
    /// 在最高优先级加入属性来源。
    First(Arc<dyn PropertySource>),
    /// 在最低优先级加入属性来源。
    Last(Arc<dyn PropertySource>),
    /// 启用一个 Active Profile。
    ActiveProfile(String),
    /// 加入一个 Default Profile。
    DefaultProfile(String),
}

impl ModuleEnvironmentContribution {
    /// 把单项贡献应用到隔离环境建造器。
    pub fn apply(
        self,
        environment: &mut ApplicationEnvironmentBuilder,
    ) -> Result<(), EnvironmentError> {
        match self {
            Self::First(source) => {
                environment.add_first(source)?;
            }
            Self::Last(source) => {
                environment.add_last(source)?;
            }
            Self::ActiveProfile(profile) => {
                environment.active_profile(profile)?;
            }
            Self::DefaultProfile(profile) => {
                environment.default_profile(profile)?;
            }
        }
        Ok(())
    }
}
