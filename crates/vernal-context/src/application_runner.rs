//! 应用一次性启动任务契约。

use std::{error::Error, future::Future};

use tokio_util::sync::CancellationToken;

/// 在组件全部启动后、应用提交 `Ready` 前执行一次的 `IoC` 组件契约。
///
/// Runner 适合安全规则缓存预热、投影检查点恢复、工具索引加载等一次性工作。
/// 长期 Worker、定时任务和消息消费循环不属于本契约：它们应由生命周期组件提交给
/// [`crate::ManagedTaskSupervisor`]，让 Context 持有任务句柄并统一处理失败与停机。
///
/// 实现类型必须作为 Singleton 注册，并通过
/// [`crate::VernalApplicationBuilder::application_runner`]、应用模块或条件模块声明。
/// Vernal 按 `IoC` 依赖计划串行调用 Runner；任一错误、panic、超时或应用取消都会
/// 阻止后续 Runner，取消应用并逆序回滚已经初始化的生命周期组件。
pub trait ApplicationRunner: Send + Sync + 'static {
    /// Runner 返回的结构化错误类型。
    type Error: Error + Send + Sync + 'static;

    /// 执行一次启动工作。
    ///
    /// 实现方应在外部调用和较长循环之间检查取消令牌，并保持 Future 能向 Tokio
    /// 让出执行权。Vernal 会使用生命周期启动预算约束本方法；超时后请求 abort，
    /// 但 Tokio 无法强制终止永久阻塞或没有让出点的代码。
    fn run(
        &self,
        cancellation: CancellationToken,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// 返回低基数、无敏感数据的静态诊断名称。
    ///
    /// 默认使用实现类型全名。覆盖值会进入启动观察和结构化错误，不得包含配置值、
    /// Token、租户或请求标识。
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
