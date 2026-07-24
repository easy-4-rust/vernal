//! 组件定义诊断快照对象。

use serde::Serialize;

/// 一个组件定义的只读、可序列化诊断视图。
///
/// 快照只复制稳定类型名、限定符、作用域、构建顺序和依赖选择器，不包含组件
/// 工厂、实例地址或业务数据，因此可以安全用于启动日志、管理端点和测试断言。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ComponentSnapshot {
    key: String,
    type_name: String,
    qualifier: Option<String>,
    scope: String,
    build_order: usize,
    dependencies: Vec<String>,
}

impl ComponentSnapshot {
    /// 由已冻结的注册表元数据创建组件快照。
    pub(crate) fn new(
        key: String,
        type_name: String,
        qualifier: Option<String>,
        scope: String,
        build_order: usize,
        dependencies: Vec<String>,
    ) -> Self {
        Self {
            key,
            type_name,
            qualifier,
            scope,
            build_order,
            dependencies,
        }
    }

    /// 返回包含可选限定符的完整组件标识。
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 返回完整 Rust 类型名。
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// 返回可选限定符。
    #[must_use]
    pub fn qualifier(&self) -> Option<&str> {
        self.qualifier.as_deref()
    }

    /// 返回稳定作用域名称。
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }

    /// 返回依赖优先计划中的零基构建位置。
    #[must_use]
    pub const fn build_order(&self) -> usize {
        self.build_order
    }

    /// 按声明顺序返回脱敏后的依赖选择器。
    #[must_use]
    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}
