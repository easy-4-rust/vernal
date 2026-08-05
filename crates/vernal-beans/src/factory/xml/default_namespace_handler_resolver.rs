//! DefaultNamespaceHandlerResolver — Spring 风格的默认命名空间处理器解析器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.xml.DefaultNamespaceHandlerResolver`。
//!
//! 在 Spring 中，`DefaultNamespaceHandlerResolver` 通过 `META-INF/spring.handlers`
//! 文件查找命名空间 URI 对应的 `NamespaceHandler` 类。
//! 文件格式：`http\://www.springframework.org/schema/context=org.springframework.context.config.ContextNamespaceHandler`
//!
//! ## 设计说明
//!
//! 在 vernal 中，此解析器维护一个 URI -> 处理器名称的映射表。

use std::collections::HashMap;

use crate::factory::xml::namespace_handler::NamespaceHandler;
use crate::factory::xml::namespace_handler_resolver::NamespaceHandlerResolver;

/// 默认命名空间处理器解析器。
///
/// 对应 Spring 的 `DefaultNamespaceHandlerResolver`。
///
/// 通过注册表查找命名空间 URI 对应的处理器。
#[derive(Debug, Default)]
pub struct DefaultNamespaceHandlerResolver {
    /// 命名空间 URI -> 处理器描述映射。
    handler_mappings: HashMap<String, String>,
    /// 解析计数。
    resolve_count: std::sync::atomic::AtomicU32,
}

impl DefaultNamespaceHandlerResolver {
    /// 创建默认命名空间处理器解析器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册命名空间处理器描述。
    ///
    /// # 参数
    /// - `namespace_uri` — 命名空间 URI
    /// - `handler_description` — 处理器描述（类名或标识）
    pub fn register_handler(
        &mut self,
        namespace_uri: impl Into<String>,
        handler_description: impl Into<String>,
    ) {
        self.handler_mappings
            .insert(namespace_uri.into(), handler_description.into());
    }

    /// 获取已注册的处理器数量。
    pub fn handler_count(&self) -> usize {
        self.handler_mappings.len()
    }

    /// 获取解析次数。
    pub fn resolve_count(&self) -> u32 {
        self.resolve_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// 获取所有注册的命名空间 URI。
    pub fn registered_uris(&self) -> Vec<String> {
        self.handler_mappings.keys().cloned().collect()
    }
}

impl NamespaceHandlerResolver for DefaultNamespaceHandlerResolver {
    fn resolve(
        &self,
        namespace_uri: &str,
    ) -> Result<Box<dyn NamespaceHandler>, Box<dyn std::error::Error + Send + Sync>> {
        self.resolve_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if let Some(desc) = self.handler_mappings.get(namespace_uri) {
            // 返回一个描述性的错误，说明需要具体的 NamespaceHandler 实现
            Err(format!(
                "DefaultNamespaceHandlerResolver: handler '{}' registered for '{}', \
                 but concrete implementation not yet available",
                desc, namespace_uri
            )
            .into())
        } else {
            Err(format!(
                "DefaultNamespaceHandlerResolver: no handler registered for namespace '{}'",
                namespace_uri
            )
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_resolve_handler() {
        let mut resolver = DefaultNamespaceHandlerResolver::new();
        resolver.register_handler(
            "http://www.springframework.org/schema/context",
            "ContextNamespaceHandler",
        );
        assert_eq!(resolver.handler_count(), 1);
        assert!(
            resolver
                .registered_uris()
                .contains(&"http://www.springframework.org/schema/context".to_string())
        );
    }

    #[test]
    fn resolve_unknown_namespace_returns_error() {
        let resolver = DefaultNamespaceHandlerResolver::new();
        let result = resolver.resolve("http://unknown.ns/schema");
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("no handler registered"));
    }

    #[test]
    fn resolve_tracks_count() {
        let mut resolver = DefaultNamespaceHandlerResolver::new();
        resolver.register_handler("http://test.ns", "TestHandler");
        assert_eq!(resolver.resolve_count(), 0);

        let _ = resolver.resolve("http://test.ns");
        assert_eq!(resolver.resolve_count(), 1);

        let _ = resolver.resolve("http://unknown.ns");
        assert_eq!(resolver.resolve_count(), 2);
    }

    #[test]
    fn registered_handler_returns_error_with_description() {
        let mut resolver = DefaultNamespaceHandlerResolver::new();
        resolver.register_handler("http://test.ns", "MyHandler");
        let result = resolver.resolve("http://test.ns");
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string();
        assert!(err_msg.contains("MyHandler"));
    }
}
