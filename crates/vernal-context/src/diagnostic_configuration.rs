//! 应用诊断配置集合对象。

use crate::SubsystemStatus;

/// 高层应用建造器传给 Context 的静态诊断元数据集合。
///
/// 该内部对象把 feature、Adapter、外部依赖与告警代码作为一个原子值传递，
/// 避免 Context 资源构造函数不断扩张。所有内容在应用构建后保持不可变。
#[derive(Default)]
pub(crate) struct DiagnosticConfiguration {
    enabled_features: Vec<String>,
    adapters: Vec<SubsystemStatus>,
    external_dependencies: Vec<SubsystemStatus>,
    warnings: Vec<String>,
}

impl DiagnosticConfiguration {
    /// 创建已经完成排序和去重的诊断配置。
    pub(crate) fn new(
        enabled_features: Vec<String>,
        adapters: Vec<SubsystemStatus>,
        external_dependencies: Vec<SubsystemStatus>,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            enabled_features,
            adapters,
            external_dependencies,
            warnings,
        }
    }

    /// 返回应用显式声明的 feature 名称。
    pub(crate) fn enabled_features(&self) -> &[String] {
        &self.enabled_features
    }

    /// 返回 Web/RPC Adapter 的脱敏状态。
    pub(crate) fn adapters(&self) -> &[SubsystemStatus] {
        &self.adapters
    }

    /// 返回外部依赖的脱敏状态。
    pub(crate) fn external_dependencies(&self) -> &[SubsystemStatus] {
        &self.external_dependencies
    }

    /// 返回应用构建阶段登记的脱敏告警代码。
    pub(crate) fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
