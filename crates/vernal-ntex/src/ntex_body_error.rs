//! Ntex 响应 Body 错误对象。

use std::{error::Error, fmt};

use vernal_web::ScopeError;

/// 表示 Ntex 响应流结束后，请求 Scope 的异步释放失败。
///
/// Ntex 的 [`ntex::http::body::MessageBody`] 以
/// `Rc<dyn Error>` 作为流错误，因此这里保留具体的 [`ScopeError`]，
/// 既让服务器端错误链可追踪，也避免把内部组件信息直接写入客户端响应。
#[derive(Debug)]
pub struct NtexBodyError {
    source: ScopeError,
}

impl NtexBodyError {
    /// 创建请求 Scope 关闭错误。
    #[must_use]
    pub const fn scope_close(source: ScopeError) -> Self {
        Self { source }
    }
}

impl fmt::Display for NtexBodyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Ntex request scope cleanup failed: {}",
            self.source
        )
    }
}

impl Error for NtexBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
