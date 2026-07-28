//! RequestScope — Spring 风格的请求作用域。
//!
//! 对应 Java 类：`org.springframework.web.context.request.RequestScope`。
//!
//! 在 HTTP 请求生命周期内缓存 Bean 实例。请求结束后销毁所有 Bean。
//! 与 `vernal-context` 的请求作用域集成。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::bean_scope::BeanScope;

/// Spring 风格的请求作用域。
///
/// 对应 Spring 的 `RequestScope`。
///
/// 在 HTTP 请求生命周期内缓存 Bean 实例：
/// - 每次 `get` 同一名称返回同一实例（缓存）
/// - 请求结束后调用所有 destruction callback
/// - 支持 `remove` 主动移除 Bean
///
/// ## 使用场景
///
/// - Web 应用中的 Request-scoped Bean
/// - 每个 HTTP 请求一个实例的组件（如 UserContext、RequestContext）
///
/// ## 与 vernal-context 的关系
///
/// `RequestScope` 可以与 `vernal-context` 的 `ScopeContext` 集成，
/// 通过 `Container::open_scope::<RequestScope>()` 进入请求作用域。
pub struct RequestScope {
    /// 已缓存的 Bean 实例（name → instance）。
    instances: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 销毁回调（name → callback）。
    destruction_callbacks: Mutex<Vec<Box<dyn FnOnce() + Send + Sync>>>,
    /// 请求标识。
    request_id: String,
}

impl RequestScope {
    /// 创建新的 RequestScope。
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
            destruction_callbacks: Mutex::new(Vec::new()),
            request_id: request_id.into(),
        }
    }

    /// 获取当前请求标识。
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// 获取已缓存的 Bean 数量。
    pub fn cached_count(&self) -> usize {
        self.instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// 执行所有销毁回调。
    ///
    /// 在请求结束后调用，清空缓存并执行所有注册的回调。
    pub fn destroy(&self) {
        // 执行所有销毁回调
        let callbacks: Vec<_> = {
            let mut cbs = self
                .destruction_callbacks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *cbs)
        };
        for callback in callbacks {
            callback();
        }
        // 清空缓存
        self.instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

impl BeanScope for RequestScope {
    /// 获取或创建作用域内的 Bean 实例。
    ///
    /// 对应 Spring 的 `RequestScope.get(String name, ObjectFactory objectFactory)`。
    ///
    /// 首次调用时通过 `object_factory` 创建实例并缓存；
    /// 后续调用直接返回缓存的实例。
    fn get(
        &self,
        name: &str,
        object_factory: &dyn Fn() -> Box<dyn Any + Send + Sync>,
    ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        let mut instances = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(existing) = instances.get(name) {
            // 已缓存：返回克隆的 Arc
            return Ok(Box::new(Arc::clone(existing)));
        }

        // 首次创建：调用工厂并缓存
        let instance = object_factory();
        let arc_instance: Arc<dyn Any + Send + Sync> = Arc::from(instance);
        instances.insert(name.to_string(), Arc::clone(&arc_instance));
        Ok(Box::new(arc_instance))
    }

    /// 移除并返回作用域内的 Bean 实例。
    ///
    /// 对应 Spring 的 `RequestScope.remove(String name)`。
    fn remove(
        &self,
        name: &str,
    ) -> Result<Option<Box<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let removed = self
            .instances
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name);
        Ok(removed.map(|arc| Box::new(arc) as Box<dyn Any + Send + Sync>))
    }

    /// 注册销毁回调。
    ///
    /// 对应 Spring 的 `RequestScope.registerDestructionCallback(String name, Runnable callback)`。
    fn register_destruction_callback(
        &self,
        _name: &str,
        callback: Box<dyn FnOnce() + Send + Sync>,
    ) {
        self.destruction_callbacks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(callback);
    }

    /// 解析上下文对象。
    ///
    /// 对应 Spring 的 `RequestScope.resolveContextualObject(String key)`。
    /// 返回请求标识作为上下文对象。
    fn resolve_contextual_object(&self, key: &str) -> Option<Box<dyn Any>> {
        if key == "request" {
            Some(Box::new(self.request_id.clone()))
        } else {
            None
        }
    }

    /// 返回会话标识（请求标识）。
    ///
    /// 对应 Spring 的 `RequestScope.getConversationId()`。
    fn conversation_id(&self) -> Option<&str> {
        Some(&self.request_id)
    }
}
