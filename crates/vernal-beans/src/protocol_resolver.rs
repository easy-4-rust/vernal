//! ProtocolResolver — 协议解析器 trait。
//!
//! 对应 Java 类：`org.springframework.core.io.ProtocolResolver`。
//!
//! 允许第三方为自定义协议前缀（如 `classpath:`、`myapp:`）注册解析逻辑，
//! 由 `ResourceLoader` 在解析资源位置时优先询问已注册的协议解析器。

use std::sync::Arc;

use crate::resource::Resource;

/// Spring 风格的协议解析器 trait。
///
/// 对应 Spring 的 `ProtocolResolver`。
///
/// 实现 `resolve` 以处理特定协议前缀的资源位置字符串，返回一个
/// [`Resource`] 实例。返回 `None` 表示当前解析器不处理该位置。
pub trait ProtocolResolver: Send + Sync {
    /// 尝试解析给定位置为资源。
    ///
    /// 对应 Spring 的 `Resource resolve(String location, ResourceLoader loader)`。
    ///
    /// 返回 `Some(resource)` 表示已处理；`None` 表示交由后续解析器/默认逻辑处理。
    fn resolve(&self, location: &str) -> Option<Arc<dyn Resource>>;
}

/// 简单的协议解析器包装：用闭包实现 [`ProtocolResolver`]。
///
/// 便于在不定义新类型的情况下注册一次性协议解析逻辑。
pub struct ClosureProtocolResolver {
    /// 协议前缀（如 `"myapp:"`）。
    prefix: String,
    /// 解析闭包：位置字符串 → 可选资源。
    resolver: Box<dyn Fn(&str) -> Option<Arc<dyn Resource>> + Send + Sync>,
}

impl ClosureProtocolResolver {
    /// 创建一个新的闭包协议解析器。
    ///
    /// `prefix` 为触发前缀；当 `location` 以该前缀开头时调用 `resolver`。
    pub fn new(
        prefix: impl Into<String>,
        resolver: Box<dyn Fn(&str) -> Option<Arc<dyn Resource>> + Send + Sync>,
    ) -> Self {
        Self {
            prefix: prefix.into(),
            resolver,
        }
    }

    /// 返回协议前缀。
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

impl ProtocolResolver for ClosureProtocolResolver {
    fn resolve(&self, location: &str) -> Option<Arc<dyn Resource>> {
        if location.starts_with(self.prefix.as_str()) {
            (self.resolver)(location)
        } else {
            None
        }
    }
}

impl std::fmt::Debug for ClosureProtocolResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureProtocolResolver")
            .field("prefix", &self.prefix)
            .finish()
    }
}
