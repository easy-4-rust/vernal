//! 应用 Runner Tokio 任务执行对象。

use std::{sync::Arc, time::Duration};

use tokio::task::{JoinError, JoinHandle};
use vernal_core::SharedError;

use crate::{ApplicationRunnerFailure, ContextError};

/// 在隔离 Tokio task 中收口一次性 Runner 的错误、panic、取消与超时。
///
/// 本对象没有运行期状态。它与生命周期执行器遵循同一“两段超时”原则：先等待
/// 启动预算，超时后请求 abort，再在有限收口时间内消费 `JoinHandle`。专用对象
/// 保留 `ApplicationRunner` 错误语义，避免把一次性工作伪装成生命周期组件。
pub(crate) struct ApplicationRunnerTaskExecutor;

impl ApplicationRunnerTaskExecutor {
    /// 等待 Runner 任务并映射为稳定 Context 错误。
    pub(crate) async fn execute(
        mut task: JoinHandle<Result<(), SharedError>>,
        runner: &'static str,
        execution_timeout: Duration,
        abort_timeout: Duration,
    ) -> Result<(), ContextError> {
        if let Ok(result) = tokio::time::timeout(execution_timeout, &mut task).await {
            return Self::map_join_result(result, runner);
        }

        // abort 只终止能够继续轮询的异步任务。收口等待仍必须有界，否则一个违反
        // Tokio 协作约束的 Runner 会永久占有 Context 的 Starting 状态。
        task.abort();
        let abort_settled = tokio::time::timeout(abort_timeout, &mut task).await.is_ok();
        Err(ContextError::ApplicationRunnerTimeout {
            runner,
            timeout: execution_timeout,
            abort_settled,
        })
    }

    /// 把正常结果、业务错误和 Tokio `JoinError` 转成同一 Runner 错误合同。
    fn map_join_result(
        result: Result<Result<(), SharedError>, JoinError>,
        runner: &'static str,
    ) -> Result<(), ContextError> {
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(source)) => Err(ContextError::ApplicationRunnerFailed {
                source: ApplicationRunnerFailure::new(runner, source),
            }),
            Err(source) => Err(ContextError::ApplicationRunnerFailed {
                source: ApplicationRunnerFailure::new(runner, Arc::new(source)),
            }),
        }
    }
}
