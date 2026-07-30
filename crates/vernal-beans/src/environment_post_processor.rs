//! EnvironmentPostProcessor — 环境后处理器。
use crate::environment::Environment;

/// 环境后处理器 trait。
pub trait EnvironmentPostProcessor: Send + Sync {
    fn post_process_environment(&self, environment: &mut dyn Environment);
}
