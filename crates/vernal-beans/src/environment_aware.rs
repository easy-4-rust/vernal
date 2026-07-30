//! EnvironmentAware — 环境感知 trait。
use crate::environment::Environment;

/// 环境感知 trait。
pub trait EnvironmentAware: Send + Sync {
    fn set_environment(&mut self, environment: Box<dyn Environment>);
}
