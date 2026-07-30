//! NamespaceHandler — Spring 风格的命名空间处理器。
use std::fmt;

/// 命名空间处理器 trait。
pub trait NamespaceHandler: Send + Sync + fmt::Debug {
    fn init(&self);
    fn parse(&self, element: &str) -> Option<String>;
}
