#![forbid(unsafe_code)]
#![doc = "Vernal 上下文支持层（对标 spring-context-support）。\n\n提供外部生态能力的 Rust 适配：\n- 缓存：事务感知装饰器、Caffeine（moka）适配、JCache 适配\n- 调度：Quartz（tokio-cron-scheduler）适配\n- 邮件：JavaMail（lettre）适配\n- 模板：FreeMarker（tera）适配"]

pub mod cache;
pub mod mail;
pub mod scheduling;
pub mod ui;
