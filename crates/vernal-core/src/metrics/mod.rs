//! 指标与启动观测包。
//!
//! 对标 Spring `org.springframework.core.metrics` 包：应用启动步骤观测。

mod application_startup;
mod default_application_startup;
mod startup_step;

pub use application_startup::ApplicationStartup;
pub use default_application_startup::DefaultApplicationStartup;
pub use startup_step::{SimpleStartupStep, StartupStep};
