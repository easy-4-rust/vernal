//! ImportSelector — 导入选择器。
use std::fmt;

/// 导入选择器 trait。
pub trait ImportSelector: Send + Sync + fmt::Debug {
    fn select_imports(&self) -> Vec<String>;
}
