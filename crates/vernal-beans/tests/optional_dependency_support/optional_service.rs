//! 立即可选依赖测试服务对象。

use super::OptionalPort;

/// 同时作为具体组件和 Trait Binding 目标的原生 Rust 对象。
pub(crate) struct OptionalService(String);

impl OptionalService {
    /// 创建持有指定稳定值的测试服务。
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// 返回服务持有的稳定值。
    pub(crate) fn value(&self) -> &str {
        &self.0
    }
}

impl OptionalPort for OptionalService {
    fn value(&self) -> &str {
        self.value()
    }
}
