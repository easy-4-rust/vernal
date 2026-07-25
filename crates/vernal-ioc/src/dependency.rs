//! 组件依赖选择器对象。

use std::{
    any::{TypeId, type_name},
    fmt,
};

use crate::Qualifier;

const TRAIT_BINDING_FLAG: u8 = 1 << 0;
const MULTIPLE_FLAG: u8 = 1 << 1;
const OPTIONAL_FLAG: u8 = 1 << 2;
const DEFERRED_FLAG: u8 = 1 << 3;

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
    flags: u8,
}

impl Dependency {
    /// 选择唯一注册的 `T` 类型组件。
    #[must_use]
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            flags: 0,
        }
    }

    /// 选择具有指定限定符的 `T` 类型组件。
    #[must_use]
    pub fn qualified<T: 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            flags: 0,
        }
    }

    /// 选择指定 Trait Object 的唯一或 Primary 实现。
    #[must_use]
    pub fn trait_of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            flags: TRAIT_BINDING_FLAG,
        }
    }

    /// 按限定符选择指定 Trait Object 的精确实现。
    #[must_use]
    pub fn trait_qualified<T: ?Sized + 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            flags: TRAIT_BINDING_FLAG,
        }
    }

    /// 选择由类型安全 Trait Provider 延迟解析的唯一或 Primary 实现。
    #[must_use]
    pub fn trait_provider_of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            flags: TRAIT_BINDING_FLAG | DEFERRED_FLAG,
        }
    }

    /// 选择由类型安全 Trait Provider 延迟解析的命名实现。
    #[must_use]
    pub fn trait_provider_qualified<T: ?Sized + 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            flags: TRAIT_BINDING_FLAG | DEFERRED_FLAG,
        }
    }

    /// 声明允许没有 Trait Binding 的可选 Trait Provider。
    #[must_use]
    pub fn optional_trait_provider_of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            flags: TRAIT_BINDING_FLAG | OPTIONAL_FLAG | DEFERRED_FLAG,
        }
    }

    /// 声明允许没有精确命名绑定的可选 Trait Provider。
    #[must_use]
    pub fn optional_trait_provider_qualified<T: ?Sized + 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            flags: TRAIT_BINDING_FLAG | OPTIONAL_FLAG | DEFERRED_FLAG,
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
            flags: TRAIT_BINDING_FLAG | MULTIPLE_FLAG,
        }
    }

    /// 选择由类型安全 Provider 延迟解析的唯一 `T` 组件。
    ///
    /// 目标仍会在注册表冻结时完成存在性与唯一性校验，但不会形成 eager 构造边。
    #[must_use]
    pub fn provider_of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            flags: DEFERRED_FLAG,
        }
    }

    /// 选择由类型安全 Provider 延迟解析的带限定符 `T` 组件。
    #[must_use]
    pub fn provider_qualified<T: 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            flags: DEFERRED_FLAG,
        }
    }

    /// 声明允许没有候选定义的可选 Provider。
    #[must_use]
    pub fn optional_provider_of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: None,
            flags: OPTIONAL_FLAG | DEFERRED_FLAG,
        }
    }

    /// 声明允许没有候选定义的带限定符可选 Provider。
    #[must_use]
    pub fn optional_provider_qualified<T: 'static>(qualifier: Qualifier) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            qualifier: Some(qualifier),
            flags: OPTIONAL_FLAG | DEFERRED_FLAG,
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
        self.has_flag(TRAIT_BINDING_FLAG)
    }

    /// 返回该选择器是否需要全部 Trait 实现。
    pub(crate) fn is_multiple(&self) -> bool {
        self.has_flag(MULTIPLE_FLAG)
    }

    /// 返回该选择器是否允许没有候选定义。
    pub(crate) fn is_optional(&self) -> bool {
        self.has_flag(OPTIONAL_FLAG)
    }

    /// 返回该依赖是否只在 Provider 调用时解析。
    pub(crate) fn is_deferred(&self) -> bool {
        self.has_flag(DEFERRED_FLAG)
    }

    /// 判断指定内部语义位是否存在。
    fn has_flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_multiple() {
            write!(formatter, "all<{}>", self.type_name)
        } else {
            let wrapper = match (
                self.is_trait_binding(),
                self.is_deferred(),
                self.is_optional(),
            ) {
                (true, true, true) => Some("optional_trait_provider"),
                (true, true, false) => Some("trait_provider"),
                (_, true, true) => Some("optional_provider"),
                (_, true, false) => Some("provider"),
                (_, false, true) => Some("optional"),
                (_, false, false) => None,
            };
            match (wrapper, &self.qualifier) {
                (Some(wrapper), Some(qualifier)) => {
                    write!(formatter, "{wrapper}<{}@{qualifier}>", self.type_name)
                }
                (Some(wrapper), None) => write!(formatter, "{wrapper}<{}>", self.type_name),
                (None, Some(qualifier)) => write!(formatter, "{}@{qualifier}", self.type_name),
                (None, None) => formatter.write_str(self.type_name),
            }
        }
    }
}
