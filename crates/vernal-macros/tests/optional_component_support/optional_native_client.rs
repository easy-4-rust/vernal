//! Option 构造注入测试原生客户端对象。

use super::OptionalDerivedPort;

/// 模拟 Tokio 应用已经构造完成、可直接进入 Vernal 图的原生客户端。
pub(crate) struct OptionalNativeClient(String);

impl OptionalNativeClient {
    /// 创建指定名称的原生客户端。
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// 返回客户端稳定名称。
    pub(crate) fn name(&self) -> &str {
        &self.0
    }
}

impl OptionalDerivedPort for OptionalNativeClient {
    fn name(&self) -> &str {
        self.name()
    }
}
