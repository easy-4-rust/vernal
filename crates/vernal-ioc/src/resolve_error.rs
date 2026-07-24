//! 组件解析错误对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;

use crate::ComponentKey;

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
            Self::Construction { component, source } => {
                write!(formatter, "failed to construct {component}: {source}")
            }
            Self::CircularRuntime { path } => {
                write!(formatter, "runtime dependency cycle: {}", path.join(" -> "))
            }
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
