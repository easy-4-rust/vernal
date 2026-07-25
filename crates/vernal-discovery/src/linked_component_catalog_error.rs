//! 链接期组件目录错误对象。

use std::{error::Error, fmt, sync::Arc};

/// 发现分组选择或静态注册元数据违反 fail-closed 合同时的结构化错误。
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LinkedComponentCatalogError {
    /// 调用方没有选择任何分组。
    EmptySelection,
    /// 分组名为空或包含空白、控制字符。
    InvalidGroup {
        /// 只用于定位声明的分组文本。
        group: Arc<str>,
    },
    /// 请求的分组没有任何链接期注册项。
    MissingGroup {
        /// 未找到的分组名。
        group: Arc<str>,
    },
    /// 手工提交的稳定声明名为空或包含空白、控制字符。
    InvalidRegistration {
        /// 注册项所属分组。
        group: Arc<str>,
        /// 非法稳定声明名。
        name: Arc<str>,
    },
    /// 同一分组出现两个相同稳定声明名。
    DuplicateRegistration {
        /// 冲突所属分组。
        group: Arc<str>,
        /// 冲突的稳定声明名。
        name: Arc<str>,
    },
}

impl fmt::Display for LinkedComponentCatalogError {
    /// 输出不包含组件字段值、配置或实例地址的稳定诊断。
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySelection => {
                formatter.write_str("linked component discovery requires at least one group")
            }
            Self::InvalidGroup { group } => {
                write!(
                    formatter,
                    "linked component discovery group `{group}` is invalid"
                )
            }
            Self::MissingGroup { group } => {
                write!(
                    formatter,
                    "linked component discovery group `{group}` has no registrations"
                )
            }
            Self::InvalidRegistration { group, name } => {
                write!(
                    formatter,
                    "linked component registration `{name}` in group `{group}` is invalid"
                )
            }
            Self::DuplicateRegistration { group, name } => {
                write!(
                    formatter,
                    "linked component registration `{name}` is duplicated in group `{group}`"
                )
            }
        }
    }
}

impl Error for LinkedComponentCatalogError {}
