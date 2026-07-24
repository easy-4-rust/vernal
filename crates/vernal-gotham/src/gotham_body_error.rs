//! Gotham 响应 Body 错误对象。

use std::{error::Error, fmt};

use vernal_web::ScopeError;

/// 表示 Gotham 响应流结束后，请求 Scope 的异步释放失败。
#[derive(Debug)]
pub struct GothamBodyError {
    source: ScopeError,
}

impl GothamBodyError {
    /// 创建请求 Scope 关闭错误。
    #[must_use]
    pub const fn scope_close(source: ScopeError) -> Self {
        Self { source }
    }
}

impl fmt::Display for GothamBodyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Gotham request scope cleanup failed: {}",
            self.source
        )
    }
}

impl Error for GothamBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
