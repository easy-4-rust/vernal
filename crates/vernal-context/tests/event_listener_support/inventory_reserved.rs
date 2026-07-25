//! 库存预留测试事件值对象。

/// 模拟 Ddd4r 或业务组件发布的强类型领域事件。
pub struct InventoryReserved {
    /// 被预留的业务序号。
    pub sequence: u64,
}
