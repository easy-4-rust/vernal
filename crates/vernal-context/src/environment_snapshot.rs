//! 应用环境脱敏诊断快照对象。

use serde::Serialize;

/// 只包含 `PropertySource` 名称和 Profile 的应用环境诊断值。
///
/// 快照不会枚举属性键或属性值，因此可以进入可序列化的启动报告，而不会把密码、
/// Token、连接串等配置数据暴露给健康检查或运维日志。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EnvironmentSnapshot {
    property_sources: Vec<String>,
    active_profiles: Vec<String>,
    default_profiles: Vec<String>,
    effective_profiles: Vec<String>,
}

impl EnvironmentSnapshot {
    /// 由冻结环境生成已经拥有全部字符串的诊断快照。
    pub(crate) fn new(
        property_sources: Vec<String>,
        active_profiles: Vec<String>,
        default_profiles: Vec<String>,
        effective_profiles: Vec<String>,
    ) -> Self {
        Self {
            property_sources,
            active_profiles,
            default_profiles,
            effective_profiles,
        }
    }

    /// 返回从高到低的属性来源名称。
    #[must_use]
    pub fn property_sources(&self) -> &[String] {
        &self.property_sources
    }

    /// 返回应用显式启用的 Profile。
    #[must_use]
    pub fn active_profiles(&self) -> &[String] {
        &self.active_profiles
    }

    /// 返回应用声明的默认 Profile。
    #[must_use]
    pub fn default_profiles(&self) -> &[String] {
        &self.default_profiles
    }

    /// 返回生成快照时实际生效的 Profile。
    #[must_use]
    pub fn effective_profiles(&self) -> &[String] {
        &self.effective_profiles
    }
}
