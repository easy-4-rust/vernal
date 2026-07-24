//! Actix Web 响应 Body 错误对象。

use std::{error::Error, fmt};

use vernal_web::ScopeError;

/// 区分 Actix 原生 Body 错误与请求 Scope 关闭错误。
#[derive(Debug)]
pub enum ActixBodyError {
    /// 原生响应 Body 返回错误。
    Upstream(Box<dyn Error>),
    /// Body 结束后关闭请求 Scope 失败。
    Scope(ScopeError),
}

impl fmt::Display for ActixBodyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Upstream(error) => write!(formatter, "Actix body upstream error: {error}"),
            Self::Scope(error) => write!(formatter, "Actix request scope error: {error}"),
        }
    }
}

impl Error for ActixBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Upstream(error) => Some(error.as_ref()),
            Self::Scope(error) => Some(error),
        }
    }
}
