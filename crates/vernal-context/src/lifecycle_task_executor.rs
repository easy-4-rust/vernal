//! 生命周期钩子 Tokio 任务执行对象。

use std::{sync::Arc, time::Duration};

use tokio::task::{JoinError, JoinHandle};
use vernal_core::BoxError;

use crate::{ContextError, LifecyclePhase};

/// 在隔离 Tokio task 中执行用户生命周期钩子并统一处理失败、panic 与超时。
///
/// 本对象没有运行期状态，只集中维护三个阶段必须一致的边界语义：普通业务错误
/// 保留为 `source`，task panic/异常取消转成共享错误源，超时则先 abort 并在受限
/// 时间内消费 `JoinHandle`。调用方据此继续执行逆序回滚，而不会把超时钩子的
/// `JoinHandle` 静默遗弃。
pub(crate) struct LifecycleTaskExecutor;

impl LifecycleTaskExecutor {
    /// 等待一个已提交的生命周期任务，并将所有终态归一为 Context 错误合同。
    pub(crate) async fn execute(
        mut task: JoinHandle<Result<(), BoxError>>,
        component: &'static str,
        phase: LifecyclePhase,
        execution_timeout: Duration,
        abort_timeout: Duration,
    ) -> Result<(), ContextError> {
        if let Ok(result) = tokio::time::timeout(execution_timeout, &mut task).await {
            return Self::map_join_result(result, component, phase);
        }

        // Tokio abort 是协作式终止：先发出请求，再有界等待 JoinHandle 观察到
        // 终态。即使用户 Future 忽略协作约束，Context 也不会在这里无限等待；
        // 错误中的 settled 位让上层诊断可以区分风险等级。
        task.abort();
        let abort_settled = tokio::time::timeout(abort_timeout, &mut task).await.is_ok();
        Err(ContextError::LifecycleTimeout {
            component,
            phase,
            timeout: execution_timeout,
            abort_settled,
        })
    }

    /// 把正常任务结果、组件业务错误和 Tokio `JoinError` 映射到统一生命周期错误。
    fn map_join_result(
        result: Result<Result<(), BoxError>, JoinError>,
        component: &'static str,
        phase: LifecyclePhase,
    ) -> Result<(), ContextError> {
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(source)) => Err(ContextError::Lifecycle {
                component,
                phase,
                source: source.into(),
            }),
            Err(source) => Err(ContextError::Lifecycle {
                component,
                phase,
                source: Arc::new(source),
            }),
        }
    }
}
