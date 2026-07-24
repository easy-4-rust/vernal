//! Tide Request Extension 解析拒绝对象。

use std::{error::Error, fmt};

use tide::{Response, StatusCode};
use vernal_ioc::ResolveError;

/// 将 Vernal 基础设施失败映射为稳定、脱敏的 Tide 响应。
#[derive(Debug)]
pub enum TideRejection {
    /// Request Extensions 中没有应用上下文。
    MissingContext,
    /// Request Extensions 中没有请求作用域。
    MissingRequestScope,
    /// `IoC` 容器无法解析目标组件。
    ComponentResolution {
        /// 原始解析错误，仅供服务端错误链使用。
        source: Box<ResolveError>,
    },
}

impl TideRejection {
    /// 创建组件解析拒绝。
    #[must_use]
    pub fn component_resolution(source: ResolveError) -> Self {
        Self::ComponentResolution {
            source: Box::new(source),
        }
    }

    /// 返回允许发送给客户端的稳定错误消息。
    #[must_use]
    pub const fn safe_message(&self) -> &'static str {
        match self {
            Self::MissingContext => "Vernal application context is unavailable",
            Self::MissingRequestScope => "Vernal request scope is unavailable",
            Self::ComponentResolution { .. } => "Vernal component resolution failed",
        }
    }

    /// 构造不暴露内部错误链的 Tide 响应。
    #[must_use]
    pub fn response(&self) -> Response {
        let mut response = Response::new(StatusCode::InternalServerError);
        response.set_body(self.safe_message());
        response
    }
}

impl fmt::Display for TideRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Tide request has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Tide request has no Vernal request scope")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Tide component resolution failed: {source}")
            }
        }
    }
}

impl Error for TideRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}
