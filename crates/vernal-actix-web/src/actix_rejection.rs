//! Actix Web 提取与中间件拒绝对象。

use std::{error::Error, fmt};

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use vernal_beans::ResolveError;
use vernal_web::ScopeError;

/// 将 Vernal 基础设施失败映射为脱敏的 Actix Web 响应。
#[derive(Debug)]
pub enum ActixRejection {
    /// 请求没有应用上下文。
    MissingContext,
    /// 请求没有请求作用域。
    MissingRequestScope,
    /// 请求没有经过严格 Local-AOP 中间件。
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

impl ActixRejection {
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

impl fmt::Display for ActixRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Actix request has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Actix request has no Vernal request scope")
            }
            Self::MissingRequestContext => {
                formatter.write_str("Actix request has no Vernal request context")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Actix component resolution failed: {source}")
            }
            Self::ScopeClose { source } => {
                write!(formatter, "Actix request scope close failed: {source}")
            }
        }
    }
}

impl Error for ActixRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            Self::ScopeClose { source } => Some(source),
            _ => None,
        }
    }
}

impl ResponseError for ActixRejection {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.safe_message())
    }
}
