//! HTTP Body 错误对象。

use std::{error::Error, fmt};

use vernal_core::BoxError;

/// 读取或适配 HTTP Body 帧时的结构化失败。
#[derive(Debug)]
#[non_exhaustive]
pub enum HttpBodyError {
    /// 请求、连接或所属 Scope 已被取消。
    Cancelled,
    /// 有限 Body 收集结果超过调用方明确设置的上限。
    LimitExceeded {
        /// 允许的最大字节数。
        limit: usize,
        /// 检测到超限时已观察到的字节数。
        observed: usize,
    },
    /// 上游框架或传输 Body 返回错误。
    Transport {
        /// 原始传输错误。
        source: BoxError,
    },
}

impl HttpBodyError {
    /// 将任意线程安全错误包装为传输错误。
    pub fn transport(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Transport {
            source: Box::new(error),
        }
    }
}

impl fmt::Display for HttpBodyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("HTTP body cancelled"),
            Self::LimitExceeded { limit, observed } => {
                write!(
                    formatter,
                    "HTTP body limit exceeded: limit={limit}, observed={observed}"
                )
            }
            Self::Transport { source } => write!(formatter, "HTTP body transport error: {source}"),
        }
    }
}

impl Error for HttpBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Transport { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}
