//! 应用 Runner 脱敏失败对象。

use std::{error::Error, fmt};

use vernal_core::SharedError;

/// 保存一次性 Runner 的静态身份与原始错误链，同时约束默认输出。
///
/// `Display` 和 `Debug` 只暴露 Runner 名称，不复制业务错误正文；运维或测试需要
/// 根因时必须显式沿 [`Error::source`] 访问。这样 Token、连接串或下游响应不会因
/// Context 被普通日志格式化而意外泄露。
#[derive(Clone)]
pub struct ApplicationRunnerFailure {
    runner: &'static str,
    source: SharedError,
}

impl ApplicationRunnerFailure {
    /// 创建保留原始根因但默认脱敏的 Runner 失败。
    #[must_use]
    pub(crate) const fn new(runner: &'static str, source: SharedError) -> Self {
        Self { runner, source }
    }

    /// 返回 Runner 提供的低基数静态诊断名称。
    #[must_use]
    pub const fn runner(&self) -> &'static str {
        self.runner
    }
}

impl fmt::Display for ApplicationRunnerFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "application runner {} failed", self.runner)
    }
}

impl fmt::Debug for ApplicationRunnerFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationRunnerFailure")
            .field("runner", &self.runner)
            .field("source", &"[redacted; use Error::source]")
            .finish()
    }
}

impl Error for ApplicationRunnerFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}
