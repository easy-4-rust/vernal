//! 简单触发器工厂 — 对标 `org.springframework.scheduling.quartz.SimpleTriggerFactoryBean`。

use std::time::Duration;

/// 简单触发器。
#[derive(Debug, Clone)]
pub struct SimpleTrigger {
    /// 重复次数（-1 表示无限）
    pub repeat_count: i32,
    /// 重复间隔
    pub repeat_interval: Duration,
    /// 开始时间偏移
    pub start_delay: Duration,
}

/// 简单触发器工厂 Bean。
///
/// 对标 Spring 的 `SimpleTriggerFactoryBean`，创建 `SimpleTrigger` 实例。
pub struct SimpleTriggerFactoryBean {
    repeat_count: i32,
    repeat_interval: Duration,
    start_delay: Duration,
}

impl SimpleTriggerFactoryBean {
    /// 创建简单触发器工厂。
    pub fn new() -> Self {
        Self {
            repeat_count: -1,
            repeat_interval: Duration::from_secs(1),
            start_delay: Duration::ZERO,
        }
    }

    /// 设置重复次数。
    pub fn set_repeat_count(&mut self, count: i32) {
        self.repeat_count = count;
    }

    /// 设置重复间隔。
    pub fn set_repeat_interval(&mut self, interval: Duration) {
        self.repeat_interval = interval;
    }

    /// 设置开始延迟。
    pub fn set_start_delay(&mut self, delay: Duration) {
        self.start_delay = delay;
    }

    /// 创建简单触发器。
    pub fn trigger(&self) -> SimpleTrigger {
        SimpleTrigger {
            repeat_count: self.repeat_count,
            repeat_interval: self.repeat_interval,
            start_delay: self.start_delay,
        }
    }
}

impl Default for SimpleTriggerFactoryBean {
    fn default() -> Self {
        Self::new()
    }
}
