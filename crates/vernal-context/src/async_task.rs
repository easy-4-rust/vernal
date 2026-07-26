//! 通用后台任务 trait。
//!
//! 对标 tx_di 的 `Component::async_run`：长期运行的后台任务，如消息消费、
//! 连接监听、定时清理等。由 `ManagedTaskSupervisor` 在 `ApplicationContext::start()`
//! 后自动派生到 Tokio runtime。
//!
//! # 与 ScheduledTask 的区别
//!
//! | 特性 | AsyncTask | ScheduledTask |
//! |------|-----------|---------------|
//! | 执行模式 | 长期运行，不退出 | 周期执行，每次运行后等待 |
//! | 退出条件 | 取消令牌被取消 | 单次执行完成 |
//! | 典型用途 | 消息消费、连接监听 | 定时清理、数据同步 |
//! | 失败处理 | 取消整个应用 | 取消整个应用 |
//!
//! # 使用方式
//!
/// ```rust,ignore
/// use vernal_context::AsyncTask;
/// use tokio_util::sync::CancellationToken;
///
/// pub struct MessageConsumer;
///
/// impl AsyncTask for MessageConsumer {
///     fn name(&self) -> &'static str { "message-consumer" }
///
///     async fn run(&self, token: CancellationToken) -> Result<(), BoxError> {
///         loop {
///             tokio::select! {
///                 _ = token.cancelled() => break,
///                 msg = receive_message() => {
///                     process_message(msg).await?;
///                 }
///             }
///         }
///         Ok(())
///     }
/// }
/// ```

use std::any::type_name;

use tokio_util::sync::CancellationToken;
use vernal_core::BoxError;

use crate::LifecycleFuture;

/// 通用后台任务 trait。
///
/// 实现此 trait 的组件会在 `ApplicationContext::start()` 后自动派生到
/// Tokio runtime。任务必须监听取消令牌，在令牌被取消时优雅退出。
///
/// # 与 tx_di 的对应关系
///
/// tx_di 的 `Component::async_run(app, token)` 在 vernal 中被拆分为：
/// - `AsyncTask`（独立 trait）— 长期后台任务
/// - `Lifecycle::start(token)` — 一次性启动钩子
///
/// 这样分离更清晰：`Lifecycle::start` 是启动时的一次性工作，
/// `AsyncTask::run` 是持续运行的后台工作。
pub trait AsyncTask: Send + Sync + 'static {
    /// 返回任务名称（用于诊断和日志）。
    ///
    /// 默认值：类型全名。
    fn name(&self) -> &'static str {
        type_name::<Self>()
    }

    /// 执行后台任务。
    ///
    /// 任务必须监听 `token`，在令牌被取消时优雅退出。
    /// 返回 `Ok(())` 表示正常退出，`Err` 表示异常（会取消整个应用）。
    fn run(&self, token: CancellationToken) -> LifecycleFuture<'_>;
}
