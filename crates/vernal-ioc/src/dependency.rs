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
    trait_binding: bool,
    multiple: bool,
}

impl Dependency {
    /// 选择唯一注册的 `T` 类型组件。
    #[must_use]
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            trait_binding: false,
            multiple: false,
        }
    }

    /// 选择具有指定限定符的 `T` 类型组件。
    #[must_use]
    pub fn qualified<T: 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            trait_binding: false,
            multiple: false,
        }
    }

    /// 选择指定 Trait Object 的唯一或 Primary 实现。
    #[must_use]
    pub fn trait_of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            trait_binding: true,
            multiple: false,
        }
    }

    /// 按限定符选择指定 Trait Object 的精确实现。
    #[must_use]
    pub fn trait_qualified<T: ?Sized + 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            trait_binding: true,
            multiple: false,
        }
    }

    /// 选择指定 Trait Object 的全部实现。
    ///
    /// 全部实现依赖允许零个候选，解析结果为空集合；一旦存在绑定，依赖图会为
    /// 每个目标组件建立显式边。
    #[must_use]
    pub fn all_traits_of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            trait_binding: true,
            multiple: true,
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

    /// 返回该选择器是否通过 Trait Binding 定位组件。
    pub(crate) fn is_trait_binding(&self) -> bool {
        self.trait_binding
    }

    /// 返回该选择器是否需要全部 Trait 实现。
    pub(crate) fn is_multiple(&self) -> bool {
        self.multiple
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.multiple {
            write!(formatter, "all<{}>", self.type_name)
        } else {
            match &self.qualifier {
                Some(qualifier) => write!(formatter, "{}@{qualifier}", self.type_name),
                None => formatter.write_str(self.type_name),
            }
        }
    }
}
