#![forbid(unsafe_code)]
#![doc = "Vernal 健康检查与指标端点（对标 spring-actuator）。"]

mod health;

pub use health::{HealthIndicator, Health, HealthStatus};
