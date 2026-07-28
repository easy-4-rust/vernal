//! 本地数据源任务存储 — 对标 `org.springframework.scheduling.quartz.LocalDataSourceJobStore`。
//!
//! 将 Quartz JDBC JobStore 的连接管理委托给 Spring 管理的 DataSource。
//! 在 Rust 中，这通过存储连接池配置来模拟。

use std::sync::Arc;

/// 连接提供者 trait。
pub trait ConnectionProvider: Send + Sync {
    /// 获取连接。
    fn get_connection(
        &self,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>;
    /// 释放连接。
    fn release_connection(&self, connection: Box<dyn std::any::Any + Send + Sync>);
}

/// 本地数据源任务存储。
///
/// 对标 Spring 的 `LocalDataSourceJobStore`，将 Quartz JDBC JobStore 的连接管理
/// 委托给 Spring 管理的 DataSource。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `TX_DATA_SOURCE_PREFIX` | `TX_DATA_SOURCE_PREFIX` | 事务数据源前缀常量 |
/// | `NON_TX_DATA_SOURCE_PREFIX` | `NON_TX_DATA_SOURCE_PREFIX` | 非事务数据源前缀常量 |
/// | `initialize()` | `initialize()` | 初始化并注册连接提供者 |
pub struct LocalDataSourceJobStore {
    /// 事务数据源前缀
    pub tx_data_source_prefix: String,
    /// 非事务数据源前缀
    pub non_tx_data_source_prefix: String,
    /// 事务连接提供者
    tx_provider: Option<Arc<dyn ConnectionProvider>>,
    /// 非事务连接提供者
    non_tx_provider: Option<Arc<dyn ConnectionProvider>>,
}

impl LocalDataSourceJobStore {
    /// 事务数据源前缀。
    pub const TX_DATA_SOURCE_PREFIX: &'static str = "springTxDataSource.";
    /// 非事务数据源前缀。
    pub const NON_TX_DATA_SOURCE_PREFIX: &'static str = "springNonTxDataSource.";

    /// 创建本地数据源任务存储。
    pub fn new() -> Self {
        Self {
            tx_data_source_prefix: Self::TX_DATA_SOURCE_PREFIX.to_string(),
            non_tx_data_source_prefix: Self::NON_TX_DATA_SOURCE_PREFIX.to_string(),
            tx_provider: None,
            non_tx_provider: None,
        }
    }

    /// 设置事务连接提供者。
    pub fn set_tx_provider(&mut self, provider: Arc<dyn ConnectionProvider>) {
        self.tx_provider = Some(provider);
    }

    /// 设置非事务连接提供者。
    pub fn set_non_tx_provider(&mut self, provider: Arc<dyn ConnectionProvider>) {
        self.non_tx_provider = Some(provider);
    }

    /// 初始化并注册连接提供者。
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 在实际项目中，这里会从 SchedulerFactoryBean 的配置中获取 DataSource
        // 并注册到 Quartz 的 DBConnectionManager
        Ok(())
    }

    /// 获取事务连接。
    pub fn get_connection(
        &self,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>>
    {
        self.tx_provider
            .as_ref()
            .ok_or_else(|| -> Box<dyn std::error::Error + Send + Sync> {
                "事务连接提供者未配置".into()
            })?
            .get_connection()
    }

    /// 释放事务连接。
    pub fn release_connection(&self, connection: Box<dyn std::any::Any + Send + Sync>) {
        if let Some(provider) = &self.tx_provider {
            provider.release_connection(connection);
        }
    }
}

impl Default for LocalDataSourceJobStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_data_source_job_store_constants() {
        assert_eq!(
            LocalDataSourceJobStore::TX_DATA_SOURCE_PREFIX,
            "springTxDataSource."
        );
        assert_eq!(
            LocalDataSourceJobStore::NON_TX_DATA_SOURCE_PREFIX,
            "springNonTxDataSource."
        );
    }

    #[test]
    fn test_local_data_source_job_store_new() {
        let store = LocalDataSourceJobStore::new();
        assert!(store.tx_provider.is_none());
        assert!(store.non_tx_provider.is_none());
    }
}
