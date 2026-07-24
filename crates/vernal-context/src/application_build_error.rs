//! Vernal 应用构建错误对象。

use std::{error::Error, fmt};

use tokio::runtime::TryCurrentError;
use vernal_ioc::{DefinitionError, GraphError};

use crate::{ConditionError, ContextError};

/// 高层应用建造器在捕获运行时、评估条件、注册组件、冻结依赖图或创建 Context
/// 时的错误。
#[derive(Debug)]
#[non_exhaustive]
pub enum ApplicationBuildError {
    /// 当前线程不在 Tokio Runtime 中，无法捕获应用运行时句柄。
    TokioRuntimeUnavailable {
        /// Tokio 返回的原始诊断。
        source: TryCurrentError,
    },
    /// 内建组件或业务组件定义无效。
    Definition {
        /// `IoC` 定义错误。
        source: DefinitionError,
    },
    /// 组件依赖图无法冻结。
    Graph {
        /// `IoC` 图校验错误。
        source: GraphError,
    },
    /// 应用上下文自身无法完成构建。
    Context {
        /// Context 构建错误。
        source: ContextError,
    },
    /// 条件组件模块声明无效或无法完成构建期评估。
    Condition {
        /// 条件装配错误。
        source: ConditionError,
    },
}

impl fmt::Display for ApplicationBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TokioRuntimeUnavailable { source } => {
                write!(formatter, "Tokio runtime is unavailable: {source}")
            }
            Self::Definition { source } => {
                write!(
                    formatter,
                    "application component definition is invalid: {source}"
                )
            }
            Self::Graph { source } => {
                write!(
                    formatter,
                    "application dependency graph is invalid: {source}"
                )
            }
            Self::Context { source } => {
                write!(formatter, "application context cannot be built: {source}")
            }
            Self::Condition { source } => {
                write!(
                    formatter,
                    "application conditional component assembly failed: {source}"
                )
            }
        }
    }
}

impl Error for ApplicationBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::TokioRuntimeUnavailable { source } => Some(source),
            Self::Definition { source } => Some(source),
            Self::Graph { source } => Some(source),
            Self::Context { source } => Some(source),
            Self::Condition { source } => Some(source),
        }
    }
}

impl From<DefinitionError> for ApplicationBuildError {
    fn from(source: DefinitionError) -> Self {
        Self::Definition { source }
    }
}

impl From<GraphError> for ApplicationBuildError {
    fn from(source: GraphError) -> Self {
        Self::Graph { source }
    }
}

impl From<ContextError> for ApplicationBuildError {
    fn from(source: ContextError) -> Self {
        Self::Context { source }
    }
}

impl From<ConditionError> for ApplicationBuildError {
    fn from(source: ConditionError) -> Self {
        Self::Condition { source }
    }
}
