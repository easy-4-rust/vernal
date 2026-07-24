//! Trait 绑定选择键对象。

use std::{
    any::{TypeId, type_name},
    fmt,
};

use crate::Qualifier;

/// 由 Trait `TypeId` 与可选限定符组成的稳定绑定选择键。
///
/// Rust Trait Object 不能通过 `Any` 直接向下转换，因此 Vernal 使用单独的选择键
/// 查找绑定，再由绑定完成具体组件到 `Arc<dyn Trait>` 的静态转换。键只保存类型
/// 身份和诊断名称，不保存实例，也不依赖字符串执行运行时类型查找。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TraitKey {
    pub(crate) type_id: TypeId,
    type_name: &'static str,
    qualifier: Option<Qualifier>,
}

impl TraitKey {
    /// 为 Trait Object 类型创建无限定符选择键。
    #[must_use]
    pub fn of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
        }
    }

    /// 为 Trait Object 类型创建带限定符选择键。
    #[must_use]
    pub fn qualified<T: ?Sized + 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
        }
    }

    /// 在绑定构建阶段设置限定符。
    pub(crate) fn with_qualifier(mut self, qualifier: Qualifier) -> Self {
        self.qualifier = Some(qualifier);
        self
    }

    /// 返回用于诊断的完整 Trait Object 类型名。
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// 返回可选限定符。
    #[must_use]
    pub fn qualifier(&self) -> Option<&Qualifier> {
        self.qualifier.as_ref()
    }
}

impl fmt::Display for TraitKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(qualifier) => write!(formatter, "{}@{qualifier}", self.type_name),
            None => formatter.write_str(self.type_name),
        }
    }
}
