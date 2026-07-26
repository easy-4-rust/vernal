//! 测试上下文。

/// 测试上下文。
///
/// 对标 Spring 的 `TestContext`。
/// 提供测试环境下的 ApplicationContext 管理。
pub struct TestContext {
    /// 测试名称
    name: String,
}

impl TestContext {
    /// 创建测试上下文。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// 获取测试名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
