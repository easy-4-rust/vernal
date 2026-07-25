//! Context 托管周期任务契约。

use std::{error::Error, future::Future};

use tokio_util::sync::CancellationToken;

use crate::TaskSchedule;

/// 由 `IoC` Singleton 实现、在应用启动后周期执行的 Tokio 任务契约。
///
/// Vernal 在全部 Lifecycle 与一次性 Runner 成功后激活任务，并在提交 `Ready`
/// 前把任务交给 [`crate::ManagedTaskSupervisor`]。同一个任务不会重叠执行；
/// 不同任务可以并发。任一次执行返回错误、panic 或异常取消都会触发应用取消，
/// 服务主循环随后沿统一 Context 关闭路径排空任务并逆序停止组件。
///
/// 如果某项工作必须成功完成后应用才能对外宣称就绪，应实现
/// [`crate::ApplicationRunner`]；本契约只表达应用存活期间的持续周期工作。
pub trait ScheduledTask: Send + Sync + 'static {
    /// 单次执行返回的结构化错误类型。
    type Error: Error + Send + Sync + 'static;

    /// 返回构建期已经校验的不可变调度计划。
    fn schedule(&self) -> TaskSchedule;

    /// 执行一个周期。
    ///
    /// 实现方必须在外部调用和长循环中检查取消令牌，并保持 Future 能向 Tokio
    /// 让出执行权。Context 关闭会先取消令牌，再按 `TaskShutdownPolicy` 等待；
    /// 不合作的任务最终由监督器请求 abort。
    fn run(
        &self,
        cancellation: CancellationToken,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// 返回低基数、无敏感信息的静态任务名称。
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
