//! Web 集成静态描述对象。

use crate::{IntegrationRole, TransportKind};

/// 用于启动诊断和兼容矩阵的不可变集成元数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegrationDescriptor {
    /// Vernal 适配器 crate。
    pub crate_name: &'static str,
    /// 上游框架或基础抽象。
    pub upstream: &'static str,
    /// 集成职责。
    pub role: IntegrationRole,
    /// 主要传输类型。
    pub transport: TransportKind,
    /// 当前实现成熟度。
    pub status: &'static str,
}

impl IntegrationDescriptor {
    /// 创建不可变集成描述。
    #[must_use]
    pub const fn new(
        crate_name: &'static str,
        upstream: &'static str,
        role: IntegrationRole,
        transport: TransportKind,
        status: &'static str,
    ) -> Self {
        Self {
            crate_name,
            upstream,
            role,
            transport,
            status,
        }
    }
}
