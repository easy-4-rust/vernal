//! 目标源错误。
//!
//! 对应 spring-aop `TargetSource` 相关错误。

use std::fmt;

/// 目标源错误。
#[derive(Debug)]
pub enum TargetSourceError {
    /// 目标不存在。
    NoSuchTarget(String),
    /// 目标创建失败。
    CreationFailed(String),
    /// 目标释放失败。
    ReleaseFailed(String),
}

impl fmt::Display for TargetSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetSourceError::NoSuchTarget(msg) => write!(f, "NoSuchTarget: {}", msg),
            TargetSourceError::CreationFailed(msg) => write!(f, "CreationFailed: {}", msg),
            TargetSourceError::ReleaseFailed(msg) => write!(f, "ReleaseFailed: {}", msg),
        }
    }
}

impl std::error::Error for TargetSourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_source_error_display() {
        let err = TargetSourceError::NoSuchTarget("test".to_string());
        assert_eq!(format!("{}", err), "NoSuchTarget: test");
    }
}
