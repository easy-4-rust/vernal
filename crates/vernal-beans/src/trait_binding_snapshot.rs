//! Trait 绑定诊断快照对象。

use serde::Serialize;

/// 一个 Trait Binding 的只读、可序列化诊断视图。
///
/// 视图不保存 upcast 闭包或实例，只描述 Trait 选择键、具体目标和 Primary
/// 语义，因而不会把运行时实现细节或业务状态带入诊断输出。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TraitBindingSnapshot {
    key: String,
    trait_name: String,
    qualifier: Option<String>,
    target: String,
    primary: bool,
}

impl TraitBindingSnapshot {
    /// 由已冻结的 Trait Binding 元数据创建快照。
    pub(crate) fn new(
        key: String,
        trait_name: String,
        qualifier: Option<String>,
        target: String,
        primary: bool,
    ) -> Self {
        Self {
            key,
            trait_name,
            qualifier,
            target,
            primary,
        }
    }

    /// 返回包含可选限定符的完整 Trait 选择键。
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 返回完整 Trait Object 类型名。
    #[must_use]
    pub fn trait_name(&self) -> &str {
        &self.trait_name
    }

    /// 返回可选绑定限定符。
    #[must_use]
    pub fn qualifier(&self) -> Option<&str> {
        self.qualifier.as_deref()
    }

    /// 返回绑定指向的具体组件标识。
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// 返回该绑定是否为无限定符单值解析的首选实现。
    #[must_use]
    pub const fn is_primary(&self) -> bool {
        self.primary
    }
}
