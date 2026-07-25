//! 应用模块装配错误对象。

use std::{error::Error, fmt};

use vernal_core::BoxError;
use vernal_beans::DefinitionError;

use crate::{ConditionError, EnvironmentError};

/// 描述显式应用模块在声明、配置、环境预检或原子提交阶段的失败。
///
/// `Display` 与 `Debug` 仅包含静态模块身份和错误类别，不展开配置值、属性键或
/// 外部系统响应。底层错误仍可通过 [`Error::source`] 显式读取。
#[non_exhaustive]
pub enum ApplicationModuleError {
    /// 模块名为空、包含空白或控制字符。
    InvalidName {
        /// 调用方提供的静态模块名。
        name: &'static str,
    },
    /// 同一应用已经成功安装过同名模块。
    DuplicateName {
        /// 重复的静态模块名。
        name: &'static str,
    },
    /// 模块没有声明任何可提交内容。
    Empty {
        /// 空模块的静态名称。
        name: &'static str,
    },
    /// 模块自身配置过程返回错误。
    Configuration {
        /// 失败模块的静态名称。
        module: &'static str,
        /// 仅供显式错误链读取的原始错误。
        source: BoxError,
    },
    /// 模块属性来源或 Profile 与应用现有 Environment 冲突。
    Environment {
        /// 失败模块的静态名称。
        module: &'static str,
        /// 结构化环境错误。
        source: EnvironmentError,
    },
    /// 模块携带的条件组件声明非法或与应用中已有名称冲突。
    Condition {
        /// 失败外层模块的静态名称。
        module: &'static str,
        /// 结构化条件模块错误。
        source: ConditionError,
    },
    /// 模块 Definition 或 Trait Binding 无法原子提交。
    Definition {
        /// 失败模块的静态名称。
        module: &'static str,
        /// 结构化 `IoC` 定义错误。
        source: DefinitionError,
    },
}

impl fmt::Display for ApplicationModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { .. } => formatter.write_str("application module name is invalid"),
            Self::DuplicateName { name } => {
                write!(
                    formatter,
                    "application module is already registered: {name}"
                )
            }
            Self::Empty { name } => write!(formatter, "application module is empty: {name}"),
            Self::Configuration { module, .. } => {
                write!(formatter, "application module {module} failed to configure")
            }
            Self::Environment { module, .. } => {
                write!(
                    formatter,
                    "application module {module} has an invalid environment contribution"
                )
            }
            Self::Condition { module, .. } => {
                write!(
                    formatter,
                    "application module {module} has an invalid conditional contribution"
                )
            }
            Self::Definition { module, .. } => {
                write!(
                    formatter,
                    "application module {module} could not commit its component bundle"
                )
            }
        }
    }
}

impl fmt::Debug for ApplicationModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { name } => formatter
                .debug_struct("InvalidName")
                .field("name", name)
                .finish(),
            Self::DuplicateName { name } => formatter
                .debug_struct("DuplicateName")
                .field("name", name)
                .finish(),
            Self::Empty { name } => formatter.debug_struct("Empty").field("name", name).finish(),
            Self::Configuration { module, .. } => formatter
                .debug_struct("Configuration")
                .field("module", module)
                .field("source", &"<redacted>")
                .finish(),
            Self::Environment { module, .. } => formatter
                .debug_struct("Environment")
                .field("module", module)
                .field("source", &"<redacted>")
                .finish(),
            Self::Condition { module, .. } => formatter
                .debug_struct("Condition")
                .field("module", module)
                .field("source", &"<redacted>")
                .finish(),
            Self::Definition { module, .. } => formatter
                .debug_struct("Definition")
                .field("module", module)
                .field("source", &"<redacted>")
                .finish(),
        }
    }
}

impl Error for ApplicationModuleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Configuration { source, .. } => Some(source.as_ref()),
            Self::Environment { source, .. } => Some(source),
            Self::Condition { source, .. } => Some(source),
            Self::Definition { source, .. } => Some(source),
            Self::InvalidName { .. } | Self::DuplicateName { .. } | Self::Empty { .. } => None,
        }
    }
}
