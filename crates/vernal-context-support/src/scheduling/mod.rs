//! 调度支持模块 — 对标 `org.springframework.scheduling`。
//!
//! 包含：
//! - `quartz`：Quartz 调度适配（tokio-cron-scheduler 后端）

pub mod quartz;
