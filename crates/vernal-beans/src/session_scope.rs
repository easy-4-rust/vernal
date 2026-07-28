//! SessionScope — Spring 风格的会话作用域。
//!
//! 对应 Java 类：`org.springframework.web.context.request.SessionScope`。
//!
//! 在 HTTP 会话生命周期内缓存 Bean 实例。会话结束后销毁所有 Bean。
//! 与 `vernal-context` 的会话作用域集成。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::bean_scope::BeanScope;

/// Spring 风格的会话作用域。
///
/// 对应 Spring 的 `SessionScope`。
///
/// 在 HTTP 会话生命周期内缓存 Bean 实例：
/// - 每次 `get` 同一名称返回同一实例（缓存）
/// - 会话结束后调用所有 destruction callback
/// - 支持 `remove` 主动移除 Bean
///
/// ## 使用场景
///
/// - Web 应用中的 Session-scoped Bean
/// - 用户会话状态（如 ShoppingCart、UserPreferences）
///
/// ## 与 vernal-context 的关系
///
/// `SessionScope` 可以与 `vernal-context` 的 `ScopeContext` 集成，
/// 通过 `Container::open_scope::<SessionScope>()` 进入会话作用域。
pub struct SessionScope {
    /// 已缓存的 Bean 实例（name → instance）。
    instances: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 销毁回调。
    destruction_callbacks: Mutex<Vec<Box<dyn FnOnce() + Send + Sync>>>,
    /// 会话标识。
    session_id: String,
}

impl SessionScope {
    /// 创建新的 SessionScope。
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            instances: Mutex::new(HashMap::new()),
            destruction_callbacks: Mutex::new(Vec::new()),
            session_id: session_id.into(),
        }
    }

    /// 获取当前会话标识。
    pub fn session_id(&self) -> &str {
        &self.session_id
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
    /// 在会话结束后调用，清空缓存并执行所有注册的回调。
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

impl BeanScope for SessionScope {
    /// 获取或创建作用域内的 Bean 实例。
    ///
    /// 对应 Spring 的 `SessionScope.get(String name, ObjectFactory objectFactory)`。
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
            return Ok(Box::new(Arc::clone(existing)));
        }

        // 首次创建
        let instance = object_factory();
        let arc_instance: Arc<dyn Any + Send + Sync> = Arc::from(instance);
        instances.insert(name.to_string(), Arc::clone(&arc_instance));
        Ok(Box::new(arc_instance))
    }

    /// 移除并返回作用域内的 Bean 实例。
    ///
    /// 对应 Spring 的 `SessionScope.remove(String name)`。
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
    /// 对应 Spring 的 `SessionScope.registerDestructionCallback(String name, Runnable callback)`。
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
    /// 对应 Spring 的 `SessionScope.resolveContextualObject(String key)`。
    /// 返回会话标识作为上下文对象。
    fn resolve_contextual_object(&self, key: &str) -> Option<Box<dyn Any>> {
        if key == "session" {
            Some(Box::new(self.session_id.clone()))
        } else {
            None
        }
    }

    /// 返回会话标识。
    ///
    /// 对应 Spring 的 `SessionScope.getConversationId()`。
    fn conversation_id(&self) -> Option<&str> {
        Some(&self.session_id)
    }
}
