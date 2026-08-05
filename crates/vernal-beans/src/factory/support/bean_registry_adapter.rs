//! BeanRegistryAdapter — Spring 风格 Bean 注册表适配器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.BeanRegistryAdapter`。
//!
//! 提供一个轻量级的 Bean 注册表，将 Bean 名称映射到 `TypeId`。
//! 在 Spring 中，`BeanRegistryAdapter` 用于将底层注册表暴露给外部消费者。
//! 在 vernal 中，此适配器用于类型查找和注册管理。

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

/// Spring 风格 Bean 注册表适配器。
///
/// 对应 Spring 的 `BeanRegistryAdapter`。
///
/// 将 Bean 名称映射到 `TypeId`，用于类型查找和注册管理。
pub struct BeanRegistryAdapter {
    beans: Mutex<HashMap<String, TypeId>>,
}

impl BeanRegistryAdapter {
    /// 创建空的注册表适配器。
    pub fn new() -> Self {
        Self {
            beans: Mutex::new(HashMap::new()),
        }
    }

    /// 注册一个 Bean 名称到类型的映射。
    pub fn register(&self, name: String, type_id: TypeId) {
        self.beans.lock().unwrap().insert(name, type_id);
    }

    /// 取消注册一个 Bean。
    pub fn unregister(&self, name: &str) {
        self.beans.lock().unwrap().remove(name);
    }

    /// 获取指定名称的 Bean 的 `TypeId`。
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.beans.lock().unwrap().get(name).copied()
    }

    /// 已注册 Bean 数量。
    pub fn count(&self) -> usize {
        self.beans.lock().unwrap().len()
    }

    /// 是否包含指定名称的 Bean。
    pub fn contains(&self, name: &str) -> bool {
        self.beans.lock().unwrap().contains_key(name)
    }

    /// 列出所有已注册的 Bean 名称。
    pub fn registered_names(&self) -> Vec<String> {
        self.beans.lock().unwrap().keys().cloned().collect()
    }

    /// 判断注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.beans.lock().unwrap().is_empty()
    }

    /// 清空注册表。
    pub fn clear(&self) {
        self.beans.lock().unwrap().clear();
    }

    /// 按 TypeId 查找所有匹配的 Bean 名称。
    pub fn find_names_by_type(&self, type_id: TypeId) -> Vec<String> {
        self.beans
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, tid)| **tid == type_id)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// 批量注册多个 Bean。
    pub fn register_all(&self, entries: Vec<(String, TypeId)>) {
        let mut beans = self.beans.lock().unwrap();
        for (name, type_id) in entries {
            beans.insert(name, type_id);
        }
    }

    /// 检查指定 TypeId 是否有注册的 Bean。
    pub fn has_type(&self, type_id: TypeId) -> bool {
        self.beans
            .lock()
            .unwrap()
            .values()
            .any(|&tid| tid == type_id)
    }

    /// 获取指定 TypeId 的 Bean 数量。
    pub fn count_by_type(&self, type_id: TypeId) -> usize {
        self.beans
            .lock()
            .unwrap()
            .values()
            .filter(|&&tid| tid == type_id)
            .count()
    }
}

impl Default for BeanRegistryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register("foo".to_string(), TypeId::of::<String>());
        assert!(adapter.contains("foo"));
        assert_eq!(adapter.get_type_id("foo"), Some(TypeId::of::<String>()));
        assert_eq!(adapter.count(), 1);
    }

    #[test]
    fn unregister_removes_entry() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register("bar".to_string(), TypeId::of::<i32>());
        assert!(adapter.contains("bar"));
        adapter.unregister("bar");
        assert!(!adapter.contains("bar"));
        assert_eq!(adapter.count(), 0);
    }

    #[test]
    fn registered_names_returns_all_keys() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register("a".to_string(), TypeId::of::<i32>());
        adapter.register("b".to_string(), TypeId::of::<String>());
        let mut names = adapter.registered_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn clear_removes_all() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register("x".to_string(), TypeId::of::<i32>());
        adapter.clear();
        assert!(adapter.is_empty());
    }

    #[test]
    fn find_names_by_type_returns_matching() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register("a".to_string(), TypeId::of::<i32>());
        adapter.register("b".to_string(), TypeId::of::<String>());
        adapter.register("c".to_string(), TypeId::of::<i32>());

        let mut names = adapter.find_names_by_type(TypeId::of::<i32>());
        names.sort();
        assert_eq!(names, vec!["a", "c"]);
    }

    #[test]
    fn register_all_registers_multiple() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register_all(vec![
            ("x".to_string(), TypeId::of::<i32>()),
            ("y".to_string(), TypeId::of::<String>()),
        ]);
        assert_eq!(adapter.count(), 2);
        assert!(adapter.contains("x"));
        assert!(adapter.contains("y"));
    }

    #[test]
    fn has_type_checks_existence() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register("a".to_string(), TypeId::of::<i32>());

        assert!(adapter.has_type(TypeId::of::<i32>()));
        assert!(!adapter.has_type(TypeId::of::<String>()));
    }

    #[test]
    fn count_by_type_returns_correct_count() {
        let adapter = BeanRegistryAdapter::new();
        adapter.register("a".to_string(), TypeId::of::<i32>());
        adapter.register("b".to_string(), TypeId::of::<i32>());
        adapter.register("c".to_string(), TypeId::of::<String>());

        assert_eq!(adapter.count_by_type(TypeId::of::<i32>()), 2);
        assert_eq!(adapter.count_by_type(TypeId::of::<String>()), 1);
        assert_eq!(adapter.count_by_type(TypeId::of::<bool>()), 0);
    }
}
