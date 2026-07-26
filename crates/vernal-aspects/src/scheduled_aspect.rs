//! 定时调度切面（对标 Spring 的 @Scheduled）。
//!
//! 通过 AOP 拦截器实现声明式定时调度。
//! 与 `vernal-context` 的 `ScheduledTask` 配合使用。

use std::sync::Arc;

use vernal_aop::{Interceptor, Invocation, InvocationError, InvocationFuture, Next};

/// 调度配置。
#[derive(Debug, Clone)]
pub struct ScheduleConfig {
    /// cron 表达式（与 fixed_delay/fixed_rate 互斥）
    pub cron: String,
    /// 固定延迟（毫秒），0 表示不使用
    pub fixed_delay_ms: u64,
    /// 固定频率（毫秒），0 表示不使用
    pub fixed_rate_ms: u64,
    /// 初始延迟（毫秒）
    pub initial_delay_ms: u64,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            cron: String::new(),
            fixed_delay_ms: 0,
            fixed_rate_ms: 0,
            initial_delay_ms: 0,
        }
    }
}

/// 定时调度切面（对标 Spring 的 ScheduledAnnotationBeanPostProcessor）。
///
/// 通过 AOP 拦截器实现声明式定时调度。
///
/// # 使用方式
///
/// ```rust,ignore
/// use vernal_aspects::{ScheduledAspect, ScheduleConfig};
///
/// let config = ScheduleConfig {
///     cron: "0 */5 * * * *".to_string(),
///     ..Default::default()
/// };
///
/// let aspect = ScheduledAspect::new(config);
/// ```
pub struct ScheduledAspect {
    config: ScheduleConfig,
}

impl ScheduledAspect {
    /// 创建定时调度切面。
    #[must_use]
    pub fn new(config: ScheduleConfig) -> Self {
        Self { config }
    }

    /// 获取调度配置。
    #[must_use]
    pub fn config(&self) -> &ScheduleConfig {
        &self.config
    }
}

impl Interceptor for ScheduledAspect {
    fn intercept<'a>(
        &'a self,
        invocation: Arc<Invocation>,
        next: Next<'a>,
    ) -> InvocationFuture<'a> {
        Box::pin(async move {
            // TODO: 集成 vernal-context 的 ScheduledTask
            // 当前实现：直接执行目标方法，不做调度
            // 完整实现需要：
            // 1. 根据 config 解析调度策略
            // 2. 注册到 ApplicationContext 的调度器
            // 3. 按调度策略周期执行目标方法

            next.run(invocation).await
        })
    }
}
