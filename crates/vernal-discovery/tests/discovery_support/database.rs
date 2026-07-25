//! 测试数据库组件对象。

/// 由核心发现分组贡献的无依赖 Singleton。
#[derive(vernal_macros::Component)]
#[component(discover = "test.core")]
pub struct Database;
