//! Poem 提取与请求生命周期拒绝对象。

use std::{error::Error, fmt};

use poem::{Body, Response, error::ResponseError, http::StatusCode};
use vernal_beans::ResolveError;
use vernal_web::ScopeError;

/// 将 Vernal 基础设施失败映射为脱敏的 Poem 响应。
#[derive(Debug)]
pub enum PoemRejection {
    /// 请求没有应用上下文。
    MissingContext,
    /// 请求没有请求作用域。
    MissingRequestScope,
    /// 请求没有经过严格 AOP Endpoint。
    MissingRequestContext,
    /// `IoC` 组件解析失败。
    ComponentResolution {
        /// 原始解析错误，仅供服务端错误链使用。
        source: Box<ResolveError>,
    },
    /// 请求 Scope 关闭失败。
    ScopeClose {
        /// 原始关闭错误。
        source: ScopeError,
    },
}

impl PoemRejection {
    /// 创建组件解析拒绝。
    #[must_use]
    pub fn component_resolution(source: ResolveError) -> Self {
        Self::ComponentResolution {
            source: Box::new(source),
        }
    }

    /// 创建请求 Scope 关闭拒绝。
    #[must_use]
    pub const fn scope_close(source: ScopeError) -> Self {
        Self::ScopeClose { source }
    }

    /// 返回可以安全发送给客户端的消息。
    #[must_use]
    pub const fn safe_message(&self) -> &'static str {
        match self {
            Self::MissingContext => "Vernal application context is unavailable",
            Self::MissingRequestScope => "Vernal request scope is unavailable",
            Self::MissingRequestContext => "Vernal request context is unavailable",
            Self::ComponentResolution { .. } => "Vernal component resolution failed",
            Self::ScopeClose { .. } => "Vernal request scope cleanup failed",
        }
    }
}

impl fmt::Display for PoemRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Poem request has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Poem request has no Vernal request scope")
            }
            Self::MissingRequestContext => {
                formatter.write_str("Poem request has no Vernal request context")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Poem component resolution failed: {source}")
            }
            Self::ScopeClose { source } => {
                write!(formatter, "Poem request scope close failed: {source}")
            }
        }
    }
}

impl Error for PoemRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            Self::ScopeClose { source } => Some(source),
            _ => None,
        }
    }
}

impl ResponseError for PoemRejection {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn as_response(&self) -> Response {
        Response::builder()
            .status(self.status())
            .body(Body::from_string(self.safe_message().to_owned()))
    }
}
