//! Tonic 请求能力错误对象。

use std::{error::Error, fmt};

use tonic::Status;
use vernal_ioc::ResolveError;

use crate::TonicStatusMapper;

/// 表示从 Tonic Request 获取 Vernal 能力时的结构化失败。
#[derive(Debug)]
pub enum TonicRequestError {
    /// 请求没有应用上下文。
    MissingContext,
    /// 请求没有请求作用域。
    MissingRequestScope,
    /// 请求没有 Vernal 请求上下文。
    MissingRequestContext,
    /// 请求没有 Tonic 方法元数据。
    MissingGrpcMethod,
    /// `IoC` 组件解析失败。
    ComponentResolution {
        /// 原始解析错误，仅进入服务端错误链。
        source: Box<ResolveError>,
    },
}

impl TonicRequestError {
    /// 创建组件解析错误。
    #[must_use]
    pub fn component_resolution(source: ResolveError) -> Self {
        Self::ComponentResolution {
            source: Box::new(source),
        }
    }
}

impl fmt::Display for TonicRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingContext => formatter.write_str("Tonic request has no Vernal context"),
            Self::MissingRequestScope => {
                formatter.write_str("Tonic request has no Vernal request scope")
            }
            Self::MissingRequestContext => {
                formatter.write_str("Tonic request has no Vernal request context")
            }
            Self::MissingGrpcMethod => {
                formatter.write_str("Tonic request has no gRPC method metadata")
            }
            Self::ComponentResolution { source } => {
                write!(formatter, "Tonic component resolution failed: {source}")
            }
        }
    }
}

impl Error for TonicRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ComponentResolution { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<TonicRequestError> for Status {
    fn from(error: TonicRequestError) -> Self {
        TonicStatusMapper::from_request_error(&error)
    }
}
