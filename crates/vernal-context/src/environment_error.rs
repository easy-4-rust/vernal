//! 应用环境解析错误对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;

/// 描述 `PropertySource` 注册、Profile 选择和属性解析期间的结构化失败。
///
/// 错误消息只携带属性键、来源名和目标类型，不包含属性值。这样 JWT 密钥、
/// 数据库连接串等敏感配置即使解析失败，也不会被意外写入启动日志或诊断报告。
#[derive(Clone)]
#[non_exhaustive]
pub enum EnvironmentError {
    /// `PropertySource` 名称为空或包含控制字符。
    InvalidPropertySourceName {
        /// 被拒绝的来源名称。
        name: String,
    },
    /// 同一 Environment 中注册了两个同名 `PropertySource`。
    DuplicatePropertySource {
        /// 冲突的来源名称。
        name: String,
    },
    /// 属性键为空或包含空白字符。
    InvalidPropertyKey {
        /// 被拒绝的属性键。
        key: String,
    },
    /// 同一个 `MapPropertySource` 输入中重复声明属性键。
    DuplicatePropertyKey {
        /// 属性来源名称。
        source_name: String,
        /// 冲突的属性键。
        key: String,
    },
    /// Profile 名称为空或包含空白字符。
    InvalidProfile {
        /// 被拒绝的 Profile 名称。
        profile: String,
    },
    /// 必需属性不存在。
    MissingProperty {
        /// 缺失的属性键。
        key: String,
    },
    /// 属性值无法转换成调用方请求的 Rust 类型。
    InvalidPropertyValue {
        /// 解析失败的属性键。
        key: String,
        /// 提供该值的 `PropertySource` 名称。
        source_name: String,
        /// 调用方请求的 Rust 类型名。
        target_type: &'static str,
    },
    /// 属性值中的占位符缺少闭合符号或键。
    MalformedPlaceholder {
        /// 含有非法占位符的顶层属性键。
        property: String,
    },
    /// 占位符引用的属性不存在，且没有声明默认值。
    UnresolvedPlaceholder {
        /// 正在解析的顶层属性键。
        property: String,
        /// 无法解析的占位符键。
        placeholder: String,
    },
    /// 多个占位符形成循环引用。
    CircularPlaceholder {
        /// 从顶层属性到重复节点的完整路径。
        path: Vec<String>,
    },
    /// 占位符展开超过框架的有界递归深度。
    PlaceholderDepthExceeded {
        /// 正在解析的顶层属性键。
        property: String,
        /// 允许的最大递归深度。
        limit: usize,
    },
    /// 自定义 `PropertySource` 读取底层系统时失败。
    PropertySource {
        /// 发生读取失败的来源名称。
        source_name: String,
        /// 来源实现返回的原始错误。
        source: SharedError,
    },
}

impl fmt::Display for EnvironmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPropertySourceName { name } => {
                write!(formatter, "invalid property source name: {name:?}")
            }
            Self::DuplicatePropertySource { name } => {
                write!(formatter, "duplicate property source: {name}")
            }
            Self::InvalidPropertyKey { key } => {
                write!(formatter, "invalid property key: {key:?}")
            }
            Self::DuplicatePropertyKey { source_name, key } => {
                write!(
                    formatter,
                    "property source {source_name} contains duplicate key {key}"
                )
            }
            Self::InvalidProfile { profile } => {
                write!(formatter, "invalid application profile: {profile:?}")
            }
            Self::MissingProperty { key } => {
                write!(formatter, "required application property is missing: {key}")
            }
            Self::InvalidPropertyValue {
                key,
                source_name,
                target_type,
            } => {
                write!(
                    formatter,
                    "property {key} from {source_name} cannot be parsed as {target_type}"
                )
            }
            Self::MalformedPlaceholder { property } => {
                write!(
                    formatter,
                    "property {property} contains a malformed placeholder"
                )
            }
            Self::UnresolvedPlaceholder {
                property,
                placeholder,
            } => {
                write!(
                    formatter,
                    "property {property} references missing placeholder {placeholder}"
                )
            }
            Self::CircularPlaceholder { path } => {
                write!(
                    formatter,
                    "circular property placeholder path: {}",
                    path.join(" -> ")
                )
            }
            Self::PlaceholderDepthExceeded { property, limit } => {
                write!(
                    formatter,
                    "property {property} exceeds placeholder depth limit {limit}"
                )
            }
            Self::PropertySource {
                source_name,
                source: _,
            } => {
                write!(
                    formatter,
                    "property source {source_name} could not read a value"
                )
            }
        }
    }
}

impl fmt::Debug for EnvironmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug 与 Display 使用同一脱敏边界；需要原始失败的调用方仍可通过
        // `Error::source` 显式访问 source chain。
        fmt::Display::fmt(self, formatter)
    }
}

impl Error for EnvironmentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PropertySource { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
