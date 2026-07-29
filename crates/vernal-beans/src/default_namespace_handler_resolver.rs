//! DefaultNamespaceHandlerResolver — 默认命名空间处理器解析器。
//!
//! 对应 Java 类：
//! `org.springframework.beans.factory.xml.DefaultNamespaceHandlerResolver`。
//!
//! 维护一个命名空间 URI → [`NamespaceHandler`] 的注册表，并支持以工厂
//! 闭包延迟创建处理器。对应 Spring 从 `META-INF/spring.handlers`
//! 加载映射的行为，这里改为以注册 API 提供。

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::namespace_handler::NamespaceHandler;
use crate::namespace_handler_resolver::NamespaceHandlerResolver;

/// 默认命名空间处理器解析器。
///
/// 对应 Spring 的 `DefaultNamespaceHandlerResolver`。
///
/// 持有两张表：
/// - 已实例化的处理器（`Arc<dyn NamespaceHandler>`）；
/// - 处理器工厂（命名空间 URI → 构造闭包），首次解析时惰性实例化。
pub struct DefaultNamespaceHandlerResolver {
    /// 已实例化的处理器。
    handlers: Mutex<HashMap<String, Arc<dyn NamespaceHandler>>>,
    /// 命名空间 URI → 工厂闭包。
    factories: Mutex<HashMap<String, Arc<dyn Fn() -> Box<dyn NamespaceHandler> + Send + Sync>>>,
}

impl DefaultNamespaceHandlerResolver {
    /// 创建空的解析器。
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(HashMap::new()),
            factories: Mutex::new(HashMap::new()),
        }
    }

    /// 直接注册一个已实例化的处理器。
    pub fn register(&self, namespace_uri: impl Into<String>, handler: Arc<dyn NamespaceHandler>) {
        self.handlers
            .lock()
            .expect("handlers lock poisoned")
            .insert(namespace_uri.into(), handler);
    }

    /// 注册一个处理器工厂（惰性实例化）。
    pub fn register_factory<F>(&self, namespace_uri: impl Into<String>, factory: F)
    where
        F: Fn() -> Box<dyn NamespaceHandler> + Send + Sync + 'static,
    {
        self.factories
            .lock()
            .expect("factories lock poisoned")
            .insert(namespace_uri.into(), Arc::new(factory));
    }

    /// 返回已知的所有命名空间 URI（已实例化 + 已注册工厂）。
    fn all_uris(&self) -> Vec<String> {
        let mut uris: Vec<String> = self
            .handlers
            .lock()
            .expect("handlers lock poisoned")
            .keys()
            .cloned()
            .collect();
        uris.extend(
            self.factories
                .lock()
                .expect("factories lock poisoned")
                .keys()
                .cloned(),
        );
        uris.sort();
        uris.dedup();
        uris
    }
}

impl Default for DefaultNamespaceHandlerResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DefaultNamespaceHandlerResolver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultNamespaceHandlerResolver")
            .field("namespace_uris", &self.all_uris())
            .finish()
    }
}

impl NamespaceHandlerResolver for DefaultNamespaceHandlerResolver {
    fn resolve(&self, namespace_uri: &str) -> Option<Arc<dyn NamespaceHandler>> {
        // 1. 已实例化缓存。
        if let Some(handler) = self
            .handlers
            .lock()
            .expect("handlers lock poisoned")
            .get(namespace_uri)
        {
            return Some(Arc::clone(handler));
        }
        // 2. 工厂惰性实例化。
        let factory = self
            .factories
            .lock()
            .expect("factories lock poisoned")
            .get(namespace_uri)
            .map(Arc::clone);
        if let Some(factory) = factory {
            let mut handler_box = factory();
            handler_box.init();
            let arc: Arc<dyn NamespaceHandler> = Arc::from(handler_box);
            self.handlers
                .lock()
                .expect("handlers lock poisoned")
                .insert(namespace_uri.to_string(), Arc::clone(&arc));
            return Some(arc);
        }
        None
    }

    fn namespace_uris(&self) -> Vec<String> {
        self.all_uris()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace_handler::NamespaceHandlerSupport;

    #[test]
    fn resolves_registered_handler() {
        let resolver = DefaultNamespaceHandlerResolver::new();
        let mut handler = NamespaceHandlerSupport::new();
        handler.init();
        resolver.register("urn:test", Arc::new(handler));
        assert!(resolver.resolve("urn:test").is_some());
        assert!(resolver.resolve("urn:unknown").is_none());
        assert!(resolver.namespace_uris().contains(&"urn:test".to_string()));
    }
}
