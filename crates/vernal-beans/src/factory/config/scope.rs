//! Scope — 对应 Spring `org.springframework.beans.factory.config.Scope`。
//!
//! Bean 作用域接口，定义 Bean 的生命周期范围。

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Bean 作用域接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.config.Scope`。
///
/// 定义 Bean 的生命周期范围，如 singleton、prototype、request、session 等。
///
/// ## 内置作用域
///
/// - `singleton` — 单例作用域（默认）
/// - `prototype` — 原型作用域（每次获取创建新实例）
/// - `request` — HTTP 请求作用域
/// - `session` — HTTP 会话作用域
/// - `application` — ServletContext 作用域
pub trait Scope: Send + Sync {
    /// 从作用域中获取 Bean。
    ///
    /// 对应 Spring 的 `Scope.get(String name, ObjectFactory<?> objectFactory)`。
    fn get(
        &self,
        name: &str,
        object_factory: &dyn Fn() -> Arc<dyn Any + Send + Sync>,
    ) -> Arc<dyn Any + Send + Sync>;

    /// 从作用域中移除 Bean。
    ///
    /// 对应 Spring 的 `Scope.remove(String name)`。
    fn remove(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>>;

    /// 注册销毁回调。
    ///
    /// 对应 Spring 的 `Scope.registerDestructionCallback(String name, Runnable callback)`。
    fn register_destruction_callback(&self, name: &str, callback: Box<dyn FnOnce() + Send>);

    /// 获取作用域会话 ID（如适用）。
    ///
    /// 对应 Spring 的 `Scope.getConversationId()`。
    fn get_conversation_id(&self) -> Option<String> {
        None
    }
}

/// 简单的线程安全作用域实现。
///
/// 使用 `Mutex<HashMap>` 存储作用域内的 Bean 实例。
pub struct SimpleScope {
    objects: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    callbacks: Mutex<HashMap<String, Box<dyn FnOnce() + Send>>>,
}

impl SimpleScope {
    /// 创建新的简单作用域。
    pub fn new() -> Self {
        Self {
            objects: Mutex::new(HashMap::new()),
            callbacks: Mutex::new(HashMap::new()),
        }
    }

    /// 获取作用域内 Bean 数量。
    pub fn size(&self) -> usize {
        self.objects.lock().unwrap().len()
    }
}

impl Default for SimpleScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope for SimpleScope {
    fn get(
        &self,
        name: &str,
        object_factory: &dyn Fn() -> Arc<dyn Any + Send + Sync>,
    ) -> Arc<dyn Any + Send + Sync> {
        let mut objects = self.objects.lock().unwrap();
        if let Some(obj) = objects.get(name) {
            return obj.clone();
        }
        let obj = object_factory();
        objects.insert(name.to_string(), obj.clone());
        obj
    }

    fn remove(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut objects = self.objects.lock().unwrap();
        objects.remove(name)
    }

    fn register_destruction_callback(&self, name: &str, callback: Box<dyn FnOnce() + Send>) {
        let mut callbacks = self.callbacks.lock().unwrap();
        callbacks.insert(name.to_string(), callback);
    }

    fn get_conversation_id(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_get_creates_and_caches() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(String::from("hello")) as Arc<dyn Any + Send + Sync>;

        let obj1 = scope.get("myBean", &factory);
        let obj2 = scope.get("myBean", &factory);

        // 同一个实例（单例作用域）
        assert!(Arc::ptr_eq(&obj1, &obj2));
        assert_eq!(scope.size(), 1);
    }

    #[test]
    fn test_scope_remove() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(42i32) as Arc<dyn Any + Send + Sync>;

        scope.get("bean1", &factory);
        assert_eq!(scope.size(), 1);

        let removed = scope.remove("bean1");
        assert!(removed.is_some());
        assert_eq!(scope.size(), 0);

        // 再次移除应返回 None
        assert!(scope.remove("bean1").is_none());
    }

    #[test]
    fn test_scope_register_destruction_callback() {
        let scope = SimpleScope::new();
        let called = Arc::new(Mutex::new(false));
        let called_clone = called.clone();

        scope.register_destruction_callback(
            "myBean",
            Box::new(move || {
                *called_clone.lock().unwrap() = true;
            }),
        );

        // 回调已注册但未执行
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_scope_default_trait() {
        let scope = SimpleScope::default();
        assert_eq!(scope.size(), 0);
    }

    #[test]
    fn test_scope_get_conversation_id() {
        let scope = SimpleScope::new();
        assert!(scope.get_conversation_id().is_none());
    }

    #[test]
    fn test_scope_get_different_beans() {
        let scope = SimpleScope::new();
        let factory1 = || Arc::new(String::from("bean1")) as Arc<dyn Any + Send + Sync>;
        let factory2 = || Arc::new(42i32) as Arc<dyn Any + Send + Sync>;

        let obj1 = scope.get("bean1", &factory1);
        let obj2 = scope.get("bean2", &factory2);

        assert_eq!(scope.size(), 2);
        assert!(!Arc::ptr_eq(&obj1, &obj2));
    }

    #[test]
    fn test_scope_remove_nonexistent() {
        let scope = SimpleScope::new();
        assert!(scope.remove("nonexistent").is_none());
    }

    #[test]
    fn test_scope_remove_and_readd() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(String::from("value")) as Arc<dyn Any + Send + Sync>;

        scope.get("bean1", &factory);
        assert_eq!(scope.size(), 1);

        scope.remove("bean1");
        assert_eq!(scope.size(), 0);

        // Re-add after removal
        scope.get("bean1", &factory);
        assert_eq!(scope.size(), 1);
    }

    #[test]
    fn test_scope_get_returns_cached_instance() {
        let scope = SimpleScope::new();
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();

        let factory = move || {
            *counter_clone.lock().unwrap() += 1;
            Arc::new(String::from("value")) as Arc<dyn Any + Send + Sync>
        };

        let _obj1 = scope.get("bean1", &factory);
        let _obj2 = scope.get("bean1", &factory);
        let _obj3 = scope.get("bean1", &factory);

        // Factory should only be called once
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_scope_multiple_destruction_callbacks() {
        let scope = SimpleScope::new();
        let called1 = Arc::new(Mutex::new(false));
        let called2 = Arc::new(Mutex::new(false));
        let c1 = called1.clone();
        let c2 = called2.clone();

        scope.register_destruction_callback("bean1", Box::new(move || {
            *c1.lock().unwrap() = true;
        }));
        scope.register_destruction_callback("bean2", Box::new(move || {
            *c2.lock().unwrap() = true;
        }));

        // Callbacks registered but not executed
        assert!(!*called1.lock().unwrap());
        assert!(!*called2.lock().unwrap());
    }

    #[test]
    fn test_scope_size_increments() {
        let scope = SimpleScope::new();
        assert_eq!(scope.size(), 0);

        let factory = || Arc::new(1) as Arc<dyn Any + Send + Sync>;
        scope.get("a", &factory);
        assert_eq!(scope.size(), 1);

        scope.get("b", &factory);
        assert_eq!(scope.size(), 2);

        scope.get("c", &factory);
        assert_eq!(scope.size(), 3);
    }

    #[test]
    fn test_scope_size_decrements_on_remove() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(1) as Arc<dyn Any + Send + Sync>;

        scope.get("a", &factory);
        scope.get("b", &factory);
        assert_eq!(scope.size(), 2);

        scope.remove("a");
        assert_eq!(scope.size(), 1);

        scope.remove("b");
        assert_eq!(scope.size(), 0);
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn test_scope_get_different_types() {
        let scope = SimpleScope::new();
        let factory1 = || Arc::new(String::from("hello")) as Arc<dyn Any + Send + Sync>;
        let factory2 = || Arc::new(42i32) as Arc<dyn Any + Send + Sync>;

        let s = scope.get("string_bean", &factory1);
        let i = scope.get("int_bean", &factory2);

        assert_eq!(scope.size(), 2);
        assert_eq!(*s.downcast_ref::<String>().unwrap(), "hello");
        assert_eq!(*i.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_scope_get_same_bean_twice() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(String::from("cached")) as Arc<dyn Any + Send + Sync>;

        let first = scope.get("bean", &factory);
        let second = scope.get("bean", &factory);

        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(scope.size(), 1);
    }

    #[test]
    fn test_scope_remove_returns_value() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(String::from("removed")) as Arc<dyn Any + Send + Sync>;

        scope.get("bean", &factory);
        let removed = scope.remove("bean").unwrap();
        assert_eq!(*removed.downcast_ref::<String>().unwrap(), "removed");
    }

    #[test]
    fn test_scope_get_conversation_id_none() {
        let scope = SimpleScope::new();
        assert!(scope.get_conversation_id().is_none());
    }

    #[test]
    fn test_scope_register_destruction_callback_multiple() {
        let scope = SimpleScope::new();
        scope.register_destruction_callback("bean1", Box::new(|| {}));
        scope.register_destruction_callback("bean2", Box::new(|| {}));
        // Both callbacks registered without panic
    }

    #[test]
    fn test_scope_get_after_remove_creates_new() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(String::from("value")) as Arc<dyn Any + Send + Sync>;

        scope.get("bean", &factory);
        scope.remove("bean");
        assert_eq!(scope.size(), 0);

        let new_instance = scope.get("bean", &factory);
        assert_eq!(scope.size(), 1);
        assert_eq!(*new_instance.downcast_ref::<String>().unwrap(), "value");
    }

    // ── Additional coverage for uncovered paths ─────────────────────────────

    #[test]
    fn test_scope_get_creates_and_caches_v2() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
        let obj1 = scope.get("bean", &factory);
        let obj2 = scope.get("bean", &factory);
        assert!(Arc::ptr_eq(&obj1, &obj2));
    }

    #[test]
    fn test_scope_remove_returns_cached_value() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(String::from("cached")) as Arc<dyn Any + Send + Sync>;
        scope.get("bean", &factory);
        let removed = scope.remove("bean").unwrap();
        assert_eq!(*removed.downcast_ref::<String>().unwrap(), "cached");
    }

    #[test]
    fn test_scope_register_multiple_callbacks() {
        let scope = SimpleScope::new();
        scope.register_destruction_callback("bean1", Box::new(|| {}));
        scope.register_destruction_callback("bean2", Box::new(|| {}));
        scope.register_destruction_callback("bean3", Box::new(|| {}));
    }

    #[test]
    fn test_scope_get_different_beans_different_types() {
        let scope = SimpleScope::new();
        let factory1 = || Arc::new(String::from("str")) as Arc<dyn Any + Send + Sync>;
        let factory2 = || Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
        let factory3 = || Arc::new(true) as Arc<dyn Any + Send + Sync>;
        scope.get("str_bean", &factory1);
        scope.get("int_bean", &factory2);
        scope.get("bool_bean", &factory3);
        assert_eq!(scope.size(), 3);
    }

    #[test]
    fn test_scope_remove_all_and_readdd() {
        let scope = SimpleScope::new();
        let factory = || Arc::new(1) as Arc<dyn Any + Send + Sync>;
        scope.get("a", &factory);
        scope.get("b", &factory);
        scope.get("c", &factory);
        assert_eq!(scope.size(), 3);
        scope.remove("a");
        scope.remove("b");
        scope.remove("c");
        assert_eq!(scope.size(), 0);
        scope.get("a", &factory);
        assert_eq!(scope.size(), 1);
    }

    #[test]
    fn test_scope_get_conversation_id_always_none() {
        let scope = SimpleScope::new();
        assert!(scope.get_conversation_id().is_none());
        // After getting a bean, still none
        let factory = || Arc::new(1) as Arc<dyn Any + Send + Sync>;
        scope.get("bean", &factory);
        assert!(scope.get_conversation_id().is_none());
    }

    #[test]
    fn test_scope_default_trait_v2() {
        let scope = SimpleScope::default();
        assert_eq!(scope.size(), 0);
        assert!(scope.get_conversation_id().is_none());
    }
}
