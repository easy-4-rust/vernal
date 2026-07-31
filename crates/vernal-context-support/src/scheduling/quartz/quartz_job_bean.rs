//! Quartz 任务 Bean 基类 — 对标 `org.springframework.scheduling.quartz.QuartzJobBean`。

/// Quartz 任务 trait。
///
/// 对标 Spring 的 `QuartzJobBean`，所有通过 Spring 管理的 Quartz 任务都实现此 trait。
pub trait QuartzJob: Send + Sync + 'static {
    /// 执行任务。
    fn execute(
        &self,
        context: &JobExecutionContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 任务执行上下文。
#[derive(Debug)]
pub struct JobExecutionContext {
    /// 任务名称
    pub job_name: String,
    /// 任务组
    pub job_group: String,
    /// 触发器名称
    pub trigger_name: Option<String>,
}

/// 简单任务实现。
pub struct SimpleQuartzJob {
    handler: Box<
        dyn Fn(&JobExecutionContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync,
    >,
}

impl SimpleQuartzJob {
    /// 创建简单任务。
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(&JobExecutionContext) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
            + Send
            + Sync
            + 'static,
    {
        Self {
            handler: Box::new(handler),
        }
    }
}

impl QuartzJob for SimpleQuartzJob {
    fn execute(
        &self,
        context: &JobExecutionContext,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        (self.handler)(context)
    }
}
