//! 组件依赖选择器对象。

use std::{
    any::{TypeId, type_name},
    fmt,
};

use crate::Qualifier;

/// 组件定义显式声明的一项依赖。
///
/// 无限定符依赖要求候选类型唯一；带限定符依赖只匹配完全相同的
/// `ComponentKey`。Vernal 在冻结注册表时验证选择结果，运行时工厂只能解析
/// 已声明依赖，从而避免隐藏的 Service Locator 依赖。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub(crate) type_id: TypeId,
    type_name: &'static str,
    qualifier: Option<Qualifier>,
}

impl Dependency {
    /// 选择唯一注册的 `T` 类型组件。
    #[must_use]
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
        }
    }

    /// 选择具有指定限定符的 `T` 类型组件。
    #[must_use]
    pub fn qualified<T: 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
        }
    }

    /// 返回用于诊断的完整 Rust 类型名。
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

impl fmt::Display for Dependency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(qualifier) => write!(formatter, "{}@{qualifier}", self.type_name),
            None => formatter.write_str(self.type_name),
        }
    }
}
