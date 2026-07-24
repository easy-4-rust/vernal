//! Warp Filter 的 Vernal 拒绝对象。

use std::{error::Error, fmt};

use vernal_ioc::ResolveError;
use warp::{Rejection, Reply, http::StatusCode, reject::Reject};

/// 将 Vernal 扩展缺失与组件解析错误保存为 Warp 原生 Rejection。
#[derive(Debug)]
pub enum WarpRejection {
    /// Request Extensions 中没有应用上下文。
    MissingContext,
    /// Request Extensions 中没有请求作用域。
    MissingRequestScope,
    /// `IoC` 组件解析失败。
    ComponentResolution {
        /// 原始解析错误，仅供服务端错误链使用。
        source: Box<ResolveError>,
    },
}

impl WarpRejection {
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

    /// 将 Vernal Rejection 恢复成稳定、脱敏的 Warp Reply。
    ///
    /// 非 Vernal 拒绝继续交给调用方后续的 recover 链。
    ///
    /// # Errors
    ///
    /// 输入不是 Vernal 拒绝时原样返回，使后续 recover 逻辑仍可处理它。
    #[allow(clippy::unused_async)] // Warp 的 recover 函数合同要求返回 Future。
    pub async fn recover(rejection: Rejection) -> Result<impl Reply, Rejection> {
        if let Some(error) = rejection.find::<Self>() {
            Ok(warp::reply::with_status(
                error.safe_message(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        } else {
            Err(rejection)
        }
    }
}

impl fmt::Display for WarpRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => {
                formatter.write_str("Warp request has no Vernal application context")
            }
            Self::MissingRequestScope => {
                formatter.write_str("Warp request has no Vernal request scope")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Warp component resolution failed: {source}")
            }
        }
    }
}

impl Error for WarpRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl Reject for WarpRejection {}
