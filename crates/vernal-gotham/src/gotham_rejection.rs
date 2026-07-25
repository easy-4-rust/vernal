//! Gotham State 解析与中间件拒绝对象。

use std::{error::Error, fmt};

use gotham::{
    helpers::http::{Body, response::create_response},
    state::State,
};
use http::{Response, StatusCode};
use vernal_beans::ResolveError;
use vernal_web::ScopeError;

/// 将 Vernal 基础设施失败映射为稳定、脱敏的 Gotham 响应。
#[derive(Debug)]
pub enum GothamRejection {
    /// Gotham State 中没有应用上下文。
    MissingContext,
    /// Gotham State 中没有请求作用域。
    MissingRequestScope,
    /// Gotham State 中没有严格 AOP 创建的请求上下文。
    MissingRequestContext,
    /// `IoC` 容器无法解析目标组件。
    ComponentResolution {
        /// 原始解析错误，仅供服务端错误链使用。
        source: Box<ResolveError>,
    },
    /// 请求 Scope 关闭失败。
    ScopeClose {
        /// 原始关闭错误，仅供服务端错误链使用。
        source: ScopeError,
    },
}

impl GothamRejection {
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

    /// 返回允许发送给客户端的稳定错误消息。
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

    /// 构造不暴露内部错误链的 Gotham 响应。
    #[must_use]
    pub fn response(&self, state: &State) -> Response<Body> {
        create_response(
            state,
            StatusCode::INTERNAL_SERVER_ERROR,
            gotham::mime::TEXT_PLAIN,
            self.safe_message(),
        )
    }
}

impl fmt::Display for GothamRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Gotham State has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Gotham State has no Vernal request scope")
            }
            Self::MissingRequestContext => {
                formatter.write_str("Gotham State has no Vernal request context")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Gotham component resolution failed: {source}")
            }
            Self::ScopeClose { source } => {
                write!(formatter, "Gotham request scope close failed: {source}")
            }
        }
    }
}

impl Error for GothamRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            Self::ScopeClose { source } => Some(source),
            _ => None,
        }
    }
}
