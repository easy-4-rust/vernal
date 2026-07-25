//! 组件唯一标识对象。

use std::{
    any::{TypeId, type_name},
    fmt,
};

use crate::Qualifier;

/// 注册表中一个组件定义的稳定身份。
///
/// `TypeId` 用于运行时快速匹配，完整类型名用于错误诊断，可选限定符用于区分
/// 同类型的多个实现。该对象不保存组件实例，因此可以安全地作为缓存键。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentKey {
    pub(crate) type_id: TypeId,
    type_name: &'static str,
    qualifier: Option<Qualifier>,
}

impl ComponentKey {
    /// 为类型 `T` 创建无限定符标识。
    #[must_use]
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
        }
    }

    /// 为类型 `T` 创建带限定符标识。
    #[must_use]
    pub fn qualified<T: 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
        }
    }

    /// 在组件定义构建阶段设置限定符。
    pub(crate) fn with_qualifier(mut self, qualifier: Qualifier) -> Self {
        self.qualifier = Some(qualifier);
        self
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

impl fmt::Display for ComponentKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(qualifier) => write!(formatter, "{}@{qualifier}", self.type_name),
            None => formatter.write_str(self.type_name),
        }
    }
}
