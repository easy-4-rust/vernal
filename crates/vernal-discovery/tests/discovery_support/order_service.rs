//! 测试订单服务组件对象。

use std::sync::Arc;

use super::Database;

/// 通过派生定义显式依赖数据库的核心发现组件。
#[derive(vernal_macros::Component)]
#[component(discover = "test.core")]
pub struct OrderService {
    database: Arc<Database>,
}

impl OrderService {
    /// 返回当前 Context 注入的数据库 Singleton。
    #[must_use]
    pub fn database(&self) -> &Arc<Database> {
        &self.database
    }
}
