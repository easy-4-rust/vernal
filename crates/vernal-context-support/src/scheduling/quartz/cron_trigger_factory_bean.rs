//! Cron 触发器工厂 — 对标 `org.springframework.scheduling.quartz.CronTriggerFactoryBean`。

/// Cron 触发器。
#[derive(Debug, Clone)]
pub struct CronTrigger {
    /// Cron 表达式
    pub cron_expression: String,
    /// 时区
    pub time_zone: Option<String>,
}

/// Cron 触发器工厂 Bean。
///
/// 对标 Spring 的 `CronTriggerFactoryBean`，创建 `CronTrigger` 实例。
pub struct CronTriggerFactoryBean {
    cron_expression: String,
    time_zone: Option<String>,
}

impl CronTriggerFactoryBean {
    /// 创建 Cron 触发器工厂。
    pub fn new(cron_expression: String) -> Self {
        Self {
            cron_expression,
            time_zone: None,
        }
    }

    /// 设置时区。
    pub fn set_time_zone(&mut self, time_zone: String) {
        self.time_zone = Some(time_zone);
    }

    /// 创建 Cron 触发器。
    pub fn trigger(&self) -> CronTrigger {
        CronTrigger {
            cron_expression: self.cron_expression.clone(),
            time_zone: self.time_zone.clone(),
        }
    }
}
