//! Vernal 应用构建错误对象。

use std::{error::Error, fmt};

use tokio::runtime::TryCurrentError;
use vernal_aop::{InvocationPlanCatalogInitializationError, OperationMetadataConflictError};
use vernal_ioc::{ComponentKey, DefinitionError, GraphError, ResolveError};

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
    /// 由 `IoC` 管理的拦截器组件无法完成解析。
    AdvisorResolution {
        /// 发生失败的拦截器组件稳定身份。
        component: ComponentKey,
        /// Container 返回的结构化解析错误。
        source: Box<ResolveError>,
    },
    /// 由 `IoC` 管理的本地拦截器组件无法完成解析。
    LocalAdvisorResolution {
        /// 发生失败的本地拦截器组件稳定身份。
        component: ComponentKey,
        /// Container 返回的结构化解析错误。
        source: Box<ResolveError>,
    },
    /// 组件化 Advisor 使用了无法表达应用级计划生命周期的作用域。
    AdvisorScope {
        /// 作用域不合法的拦截器组件身份。
        component: ComponentKey,
        /// 稳定、无业务数据的作用域名称。
        scope: &'static str,
    },
    /// 同一稳定操作身份出现冲突的标签或限定符声明。
    OperationMetadata {
        /// AOP 计划编译返回的元数据冲突。
        source: OperationMetadataConflictError,
    },
    /// AOP 调用计划目录违反一次封存合同。
    AopCatalogInitialization {
        /// 目录返回的一次封存错误。
        source: InvocationPlanCatalogInitializationError,
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
            Self::AdvisorResolution { component, source } => {
                write!(
                    formatter,
                    "application advisor component {component} cannot be resolved: {source}"
                )
            }
            Self::LocalAdvisorResolution { component, source } => {
                write!(
                    formatter,
                    "application local advisor component {component} cannot be resolved: {source}"
                )
            }
            Self::AdvisorScope { component, scope } => {
                write!(
                    formatter,
                    "application advisor component {component} must be singleton, not {scope}"
                )
            }
            Self::OperationMetadata { source } => {
                write!(
                    formatter,
                    "application operation metadata is inconsistent: {source}"
                )
            }
            Self::AopCatalogInitialization { source } => {
                write!(
                    formatter,
                    "application AOP catalog cannot be initialized: {source}"
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
            Self::AdvisorResolution { source, .. }
            | Self::LocalAdvisorResolution { source, .. } => Some(source.as_ref()),
            Self::AdvisorScope { .. } => None,
            Self::OperationMetadata { source } => Some(source),
            Self::AopCatalogInitialization { source } => Some(source),
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

impl From<InvocationPlanCatalogInitializationError> for ApplicationBuildError {
    fn from(source: InvocationPlanCatalogInitializationError) -> Self {
        Self::AopCatalogInitialization { source }
    }
}

impl From<OperationMetadataConflictError> for ApplicationBuildError {
    fn from(source: OperationMetadataConflictError) -> Self {
        Self::OperationMetadata { source }
    }
}
