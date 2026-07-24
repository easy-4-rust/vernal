//! 启动阶段观察记录对象。

use serde::Serialize;

use crate::{DiagnosticOutcome, DiagnosticPhase};

/// 单个容器或生命周期步骤的只读、脱敏耗时记录。
///
/// `subject` 只使用框架类型名或组件诊断名，`elapsed_microseconds` 使用单调时钟
/// 计算。对象故意没有 `error_message` 字段，避免管理端点或序列化日志复制业务
/// 错误中的敏感信息。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StartupObservation {
    subject: String,
    phase: DiagnosticPhase,
    outcome: DiagnosticOutcome,
    elapsed_microseconds: u64,
}

impl StartupObservation {
    /// 创建一条完成后的启动观察记录。
    pub(crate) fn new(
        subject: String,
        phase: DiagnosticPhase,
        outcome: DiagnosticOutcome,
        elapsed_microseconds: u64,
    ) -> Self {
        Self {
            subject,
            phase,
            outcome,
            elapsed_microseconds,
        }
    }

    /// 返回被观察的容器或组件诊断名称。
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// 返回执行阶段。
    #[must_use]
    pub const fn phase(&self) -> DiagnosticPhase {
        self.phase
    }

    /// 返回脱敏结果。
    #[must_use]
    pub const fn outcome(&self) -> DiagnosticOutcome {
        self.outcome
    }

    /// 返回该步骤消耗的微秒数。
    #[must_use]
    pub const fn elapsed_microseconds(&self) -> u64 {
        self.elapsed_microseconds
    }
}
