//! 应用子系统状态对象。

use serde::Serialize;

use crate::DiagnosticState;

/// 一个 Adapter 或外部依赖的只读、脱敏状态。
///
/// 名称用于标识公开子系统，例如 `axum`、`redis-session` 或 `postgres-primary`；
/// 状态只能取固定枚举。对象不接受任意详情文本，从类型层面减少把连接串、令牌
/// 或底层错误写入启动报告的机会。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SubsystemStatus {
    name: String,
    state: DiagnosticState,
}

impl SubsystemStatus {
    /// 创建一个脱敏子系统状态。
    #[must_use]
    pub fn new(name: impl Into<String>, state: DiagnosticState) -> Self {
        Self {
            name: name.into(),
            state,
        }
    }

    /// 返回子系统公开诊断名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回子系统可用性状态。
    #[must_use]
    pub const fn state(&self) -> DiagnosticState {
        self.state
    }
}
