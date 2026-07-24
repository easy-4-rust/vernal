//! Salvo Depot 解析拒绝对象。

use std::{error::Error, fmt};

use salvo::{Scribe, http::StatusCode, writing::Text};
use vernal_ioc::ResolveError;

/// 将 Vernal 基础设施失败映射为脱敏的 Salvo 响应。
#[derive(Debug)]
pub enum SalvoRejection {
    /// Depot 中没有应用上下文。
    MissingContext,
    /// Depot 中没有请求作用域。
    MissingRequestScope,
    /// `IoC` 组件解析失败。
    ComponentResolution {
        /// 原始解析错误，仅供服务端错误链使用。
        source: Box<ResolveError>,
    },
}

impl SalvoRejection {
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

impl fmt::Display for SalvoRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Salvo Depot has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Salvo Depot has no Vernal request scope")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Salvo component resolution failed: {source}")
            }
        }
    }
}

impl Error for SalvoRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl Scribe for SalvoRejection {
    fn render(self, response: &mut salvo::Response) {
        response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        response.render(Text::Plain(self.safe_message()));
    }
}
