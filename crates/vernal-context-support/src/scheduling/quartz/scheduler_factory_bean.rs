//! 调度器工厂 Bean — 对标 `org.springframework.scheduling.quartz.SchedulerFactoryBean`。
//!
//! 创建并管理 Quartz Scheduler 的生命周期。
//! 在 Rust 中使用 tokio 任务模拟调度器。

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot};

/// 调度器状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerState {
    /// 未启动。
    Stopped,
    /// 运行中。
    Running,
    /// 已暂停。
    Paused,
}

/// 调度器工厂 Bean。
///
/// 对标 Spring 的 `SchedulerFactoryBean`，创建并管理调度器的生命周期。
/// 支持 start / pause / resume / shutdown 四态。
///
/// # Spring 方法映射
///
/// | Spring 方法 | Rust 方法 | 说明 |
/// |---|---|---|
/// | `afterPropertiesSet()` | `start()` | 启动调度器 |
/// | `start()` | `start()` | 启动调度器 |
/// | `stop()` | `stop()` | 停止调度器 |
/// | `shutdown()` | `shutdown()` | 关闭调度器 |
/// | `pause()` | `pause()` | 暂停调度器 |
/// | `resume()` | `resume()` | 恢复调度器 |
/// | `isRunning()` | `is_running()` | 是否运行中 |
pub struct SchedulerFactoryBean {
    state: Arc<Mutex<SchedulerState>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl SchedulerFactoryBean {
    /// 创建调度器工厂 Bean。
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SchedulerState::Stopped)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动调度器。
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Running {
            return Ok(());
        }

        let (tx, rx) = oneshot::channel();
        *self.shutdown_tx.lock().await = Some(tx);
        *state = SchedulerState::Running;

        // 启动后台任务
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            let mut rx = rx;
            loop {
                tokio::select! {
                    _ = &mut rx => {
                        // 收到关闭信号
                        let mut state = state_clone.lock().await;
                        *state = SchedulerState::Stopped;
                        break;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                        // 模拟调度器心跳
                    }
                }
            }
        });

        Ok(())
    }

    /// 停止调度器。
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.lock().await;
        if *state == SchedulerState::Stopped {
            return Ok(());
        }

        // 发送关闭信号
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }

        *state = SchedulerState::Stopped;
        Ok(())
    }

    /// 关闭调度器（与 stop 等价）。
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stop().await
    }

    /// 暂停调度器。
    pub async fn pause(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.lock().await;
        if *state != SchedulerState::Running {
            return Err("调度器未运行，无法暂停".into());
        }
        *state = SchedulerState::Paused;
        Ok(())
    }

    /// 恢复调度器。
    pub async fn resume(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.state.lock().await;
        if *state != SchedulerState::Paused {
            return Err("调度器未暂停，无法恢复".into());
        }
        *state = SchedulerState::Running;
        Ok(())
    }

    /// 获取调度器状态。
    pub async fn state(&self) -> SchedulerState {
        *self.state.lock().await
    }

    /// 是否运行中。
    pub async fn is_running(&self) -> bool {
        *self.state.lock().await == SchedulerState::Running
    }
}

impl Default for SchedulerFactoryBean {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SchedulerFactoryBean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchedulerFactoryBean").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_lifecycle() {
        let bean = SchedulerFactoryBean::new();
        assert_eq!(bean.state().await, SchedulerState::Stopped);
        assert!(!bean.is_running().await);

        // 启动
        bean.start().await.unwrap();
        assert!(bean.is_running().await);
        assert_eq!(bean.state().await, SchedulerState::Running);

        // 暂停
        bean.pause().await.unwrap();
        assert_eq!(bean.state().await, SchedulerState::Paused);

        // 恢复
        bean.resume().await.unwrap();
        assert!(bean.is_running().await);

        // 停止
        bean.stop().await.unwrap();
        assert_eq!(bean.state().await, SchedulerState::Stopped);

        // 关闭
        bean.start().await.unwrap();
        bean.shutdown().await.unwrap();
        assert_eq!(bean.state().await, SchedulerState::Stopped);
    }

    #[tokio::test]
    async fn test_pause_not_running() {
        let bean = SchedulerFactoryBean::new();
        assert!(bean.pause().await.is_err());
    }

    #[tokio::test]
    async fn test_resume_not_paused() {
        let bean = SchedulerFactoryBean::new();
        bean.start().await.unwrap();
        assert!(bean.resume().await.is_err());
    }

    #[tokio::test]
    async fn test_double_start() {
        let bean = SchedulerFactoryBean::new();
        bean.start().await.unwrap();
        // 重复启动应该无错误
        bean.start().await.unwrap();
        assert!(bean.is_running().await);
        bean.stop().await.unwrap();
    }
}
