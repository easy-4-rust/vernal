//! 链接期组件索引错误对象。
//!
//! 对应 Spring `processor/` 子模块中各种 `IllegalStateException` / `IllegalArgumentException`
//! 的结构化错误聚合。

use std::{error::Error, fmt, sync::Arc};

/// 索引分组选择或静态注册元数据违反 fail-closed 合同时的结构化错误。
///
/// 对应 vernal-context-indexer 的多种失败场景：
/// - 路径选择为空
/// - 分组名非法
/// - 分组无匹配条目
/// - 条目声明名非法
/// - 同名冲突
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkedComponentIndexError {
    /// 调用方没有选择任何分组。
    EmptySelection,
    /// 分组名为空或包含空白、控制字符。
    InvalidGroup {
        /// 只用于定位声明的分组文本。
        group: Arc<str>,
    },
    /// 请求的分组没有任何链接期条目。
    MissingGroup {
        /// 未找到的分组名。
        group: Arc<str>,
    },
    /// 手工提交的稳定声明名为空或包含空白、控制字符。
    InvalidEntry {
        /// 条目所属分组。
        group: Arc<str>,
        /// 非法稳定声明名。
        name: Arc<str>,
    },
    /// 同一分组出现两个相同稳定声明名。
    DuplicateEntry {
        /// 冲突所属分组。
        group: Arc<str>,
        /// 冲突的稳定声明名。
        name: Arc<str>,
    },
}

impl fmt::Display for LinkedComponentIndexError {
    /// 输出不包含组件字段值、配置或实例地址的稳定诊断。
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySelection => {
                formatter.write_str("linked component index requires at least one group")
            }
            Self::InvalidGroup { group } => {
                write!(
                    formatter,
                    "linked component index group `{group}` is invalid"
                )
            }
            Self::MissingGroup { group } => {
                write!(
                    formatter,
                    "linked component index group `{group}` has no entries"
                )
            }
            Self::InvalidEntry { group, name } => {
                write!(
                    formatter,
                    "linked component entry `{name}` in group `{group}` is invalid"
                )
            }
            Self::DuplicateEntry { group, name } => {
                write!(
                    formatter,
                    "linked component entry `{name}` is duplicated in group `{group}`"
                )
            }
        }
    }
}

impl Error for LinkedComponentIndexError {}
