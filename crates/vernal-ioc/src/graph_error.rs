//! 依赖图校验错误对象。

use std::{error::Error, fmt};

/// 注册表冻结阶段发现的依赖图错误。
///
/// 每个变体都保留完整、可读的组件路径，调用方无需重新遍历图即可生成启动诊断。
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GraphError {
    /// 已声明依赖没有任何候选定义。
    MissingDependency {
        /// 从当前根组件到缺失选择器的路径。
        path: Vec<String>,
    },
    /// 无限定符依赖匹配到多个候选。
    AmbiguousDependency {
        /// 从当前根组件到歧义选择器的路径。
        path: Vec<String>,
        /// 所有候选组件标识。
        candidates: Vec<String>,
    },
    /// 依赖图中存在闭环。
    Cycle {
        /// 首尾相同的闭合环路径。
        path: Vec<String>,
    },
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDependency { path } => {
                write!(formatter, "missing dependency: {}", path.join(" -> "))
            }
            Self::AmbiguousDependency { path, candidates } => write!(
                formatter,
                "ambiguous dependency: {}; candidates: {}",
                path.join(" -> "),
                candidates.join(", ")
            ),
            Self::Cycle { path } => {
                write!(formatter, "dependency cycle: {}", path.join(" -> "))
            }
        }
    }
}

impl Error for GraphError {}
