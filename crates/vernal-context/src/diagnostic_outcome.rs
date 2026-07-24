//! 启动诊断结果对象。

use serde::Serialize;

/// 一条启动观察记录的脱敏结果。
///
/// 诊断快照只保存成功或失败分类，不保存组件返回的错误文本。调用方仍可从
/// `ContextError::source` 获取受控错误链，而公开报告不会意外泄露令牌、连接串
/// 或业务数据。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticOutcome {
    /// 阶段执行成功。
    Succeeded,
    /// 阶段执行失败。
    Failed,
}

impl DiagnosticOutcome {
    /// 返回适合稳定日志、指标标签和测试断言的结果名称。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}
