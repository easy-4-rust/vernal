//! Rocket Request Guard 的 Vernal 拒绝对象。

use std::{error::Error, fmt};

use vernal_ioc::ResolveError;

/// 保存 Managed State、请求 Scope 与组件解析失败的结构化原因。
#[derive(Debug)]
pub enum RocketRejection {
    /// Rocket Managed State 中没有应用上下文。
    MissingContext,
    /// 请求本地缓存中没有请求作用域。
    MissingRequestScope,
    /// `IoC` 组件解析失败。
    ComponentResolution {
        /// 原始解析错误，仅供服务端错误链使用。
        source: Box<ResolveError>,
    },
}

impl RocketRejection {
    /// 创建组件解析拒绝。
    #[must_use]
    pub fn component_resolution(source: ResolveError) -> Self {
        Self::ComponentResolution {
            source: Box::new(source),
        }
    }

    /// 返回可以安全发送给客户端的消息。
    #[must_use]
    pub const fn safe_message(&self) -> &'static str {
        match self {
            Self::MissingContext => "Vernal application context is unavailable",
            Self::MissingRequestScope => "Vernal request scope is unavailable",
            Self::ComponentResolution { .. } => "Vernal component resolution failed",
        }
    }
}

impl fmt::Display for RocketRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => {
                formatter.write_str("Rocket Managed State has no Vernal context")
            }
            Self::MissingRequestScope => {
                formatter.write_str("Rocket request has no Vernal request scope")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Rocket component resolution failed: {source}")
            }
        }
    }
}

impl Error for RocketRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}
