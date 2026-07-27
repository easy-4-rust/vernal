//! 启动诊断阶段对象。

use serde::Serialize;

/// 一条启动观察记录所属的稳定执行阶段。
///
/// 该枚举比生命周期接口多出容器预热与组件解析两个阶段，使构造失败和
/// `Lifecycle` 失败可以在报告中明确区分，而不需要序列化底层错误正文。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticPhase {
    /// 按构建计划预热全部 Singleton。
    ContainerWarmUp,
    /// 从容器解析生命周期组件。
    ComponentResolution,
    /// 执行组件初始化钩子。
    Initialize,
    /// 执行组件启动钩子。
    Start,
    /// 执行一次性应用 Runner。
    ApplicationRunner,
    /// 解析并向 Tokio 任务监督器提交周期任务。
    ScheduledTaskActivation,
    /// 正常关闭或失败回滚时执行组件停止钩子。
    Stop,
    /// `pause()` 期间执行可暂停组件的 pause 钩子（对标 Spring 7.0 `onPause()`）。
    Pause,
}

impl DiagnosticPhase {
    /// 返回适合稳定日志、指标标签和测试断言的阶段名称。
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContainerWarmUp => "container_warm_up",
            Self::ComponentResolution => "component_resolution",
            Self::Initialize => "initialize",
            Self::Start => "start",
            Self::ApplicationRunner => "application_runner",
            Self::ScheduledTaskActivation => "scheduled_task_activation",
            Self::Stop => "stop",
            Self::Pause => "pause",
        }
    }
}
