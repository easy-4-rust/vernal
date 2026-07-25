//! 组件解析错误对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;

use crate::{ComponentKey, ScopeKey, ScopeState, TraitKey};

/// 运行时选择或构造组件失败。
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ResolveError {
    /// 没有定义匹配请求选择器。
    NotFound {
        /// 请求的类型与可选限定符。
        component: String,
        /// 运行时解析路径。
        path: Vec<String>,
    },
    /// 无限定符请求匹配到多个定义。
    Ambiguous {
        /// 请求的组件类型。
        component: String,
        /// 候选组件标识。
        candidates: Vec<String>,
        /// 运行时解析路径。
        path: Vec<String>,
    },
    /// 工厂尝试读取未显式声明的依赖。
    UndeclaredDependency {
        /// 发起解析的组件。
        component: ComponentKey,
        /// 未声明的依赖选择器。
        dependency: String,
    },
    /// 擦除类型后的工厂结果与注册类型不一致。
    TypeMismatch {
        /// 类型不匹配的组件。
        component: ComponentKey,
    },
    /// Trait Binding 的转换结果与声明的 Trait Object 类型不一致。
    TraitBindingTypeMismatch {
        /// 无法恢复的 Trait 选择键。
        binding: TraitKey,
        /// 提供转换输入的具体组件。
        target: ComponentKey,
    },
    /// 组件工厂返回业务错误。
    Construction {
        /// 构造失败的组件。
        component: ComponentKey,
        /// 原始工厂错误。
        source: SharedError,
    },
    /// 图校验后仍在运行时观察到依赖环。
    CircularRuntime {
        /// 闭合的运行时解析路径。
        path: Vec<String>,
    },
    /// Provider 在拥有它的组件工厂返回前被调用。
    ProviderUsedDuringConstruction {
        /// 尚未完成构造的 Provider 消费方。
        component: ComponentKey,
        /// 被提前请求的延迟依赖选择器。
        dependency: String,
    },
    /// 自定义作用域组件在没有匹配 Context 的解析路径中被请求。
    ScopeNotActive {
        /// 无法解析的组件。
        component: ComponentKey,
        /// 组件声明的作用域。
        scope: ScopeKey,
    },
    /// `ScopeContext` 来自另一个 Container。
    ScopeOwnerMismatch {
        /// 调用方传入的最内层作用域。
        scope: ScopeKey,
    },
    /// 目标 Scope 已关闭、正在关闭或被父级取消。
    ScopeUnavailable {
        /// 无法解析的组件。
        component: ComponentKey,
        /// 目标作用域。
        scope: ScopeKey,
        /// 目标作用域当前状态。
        state: ScopeState,
        /// 当前或父作用域是否已经发出取消信号。
        cancelled: bool,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { component, path } => write!(
                formatter,
                "component not found: {component}; path: {}",
                path.join(" -> ")
            ),
            Self::Ambiguous {
                component,
                candidates,
                path,
            } => write!(
                formatter,
                "ambiguous component {component}; candidates: {}; path: {}",
                candidates.join(", "),
                path.join(" -> ")
            ),
            Self::UndeclaredDependency {
                component,
                dependency,
            } => write!(
                formatter,
                "factory for {component} requested undeclared dependency {dependency}"
            ),
            Self::TypeMismatch { component } => {
                write!(formatter, "factory output type mismatch for {component}")
            }
            Self::TraitBindingTypeMismatch { binding, target } => {
                write!(
                    formatter,
                    "trait binding output type mismatch for {binding} from {target}"
                )
            }
            Self::Construction { component, source } => {
                write!(formatter, "failed to construct {component}: {source}")
            }
            Self::CircularRuntime { path } => {
                write!(formatter, "runtime dependency cycle: {}", path.join(" -> "))
            }
            Self::ProviderUsedDuringConstruction {
                component,
                dependency,
            } => write!(
                formatter,
                "component {component} used provider {dependency} before its factory completed"
            ),
            Self::ScopeNotActive { component, scope } => {
                write!(
                    formatter,
                    "component {component} requires active scope {scope}"
                )
            }
            Self::ScopeOwnerMismatch { scope } => {
                write!(
                    formatter,
                    "scope {scope} belongs to a different component container"
                )
            }
            Self::ScopeUnavailable {
                component,
                scope,
                state,
                cancelled,
            } => write!(
                formatter,
                "scope {scope} is unavailable for component {component}; state: {state:?}; cancelled: {cancelled}"
            ),
        }
    }
}

impl Error for ResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Construction { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
