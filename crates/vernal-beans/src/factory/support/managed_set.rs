//! ManagedSet — Spring 风格管理集合。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedSet`。
//!
//! 在 Spring 的 Bean 定义解析过程中，`ManagedSet` 表示一个
//! `<set>` 元素。元素可以是未解析的 Bean 引用，
//! 在 Bean 实例化时被解析为真实对象。

use std::collections::HashSet;
use std::sync::Mutex;

/// Spring 风格管理集合。
///
/// 对应 Spring 的 `ManagedSet`。
///
/// 表示 Bean 定义中 `<set>` 元素的值。元素为字符串键。
pub struct ManagedSet {
    items: Mutex<HashSet<String>>,
}

impl ManagedSet {
    /// 创建空的管理集合。
    pub fn new() -> Self { Self { items: Mutex::new(HashSet::new()) } }

    /// 添加元素，返回是否为新元素。
    pub fn add(&self, key: &str) -> bool {
        self.items.lock().unwrap().insert(key.to_string())
    }

    /// 是否包含指定元素。
    pub fn contains(&self, key: &str) -> bool {
        self.items.lock().unwrap().contains(key)
    }

    /// 元素数量。
    pub fn len(&self) -> usize { self.items.lock().unwrap().len() }

    /// 是否为空。
    pub fn is_empty(&self) -> bool { self.items.lock().unwrap().is_empty() }

    /// 移除元素，返回是否成功。
    pub fn remove(&self, key: &str) -> bool {
        self.items.lock().unwrap().remove(key)
    }

    /// 所有元素收集为 `Vec`。
    pub fn to_vec(&self) -> Vec<String> {
        self.items.lock().unwrap().iter().cloned().collect()
    }

    /// 清空集合。
    pub fn clear(&self) {
        self.items.lock().unwrap().clear();
    }

    /// 批量添加多个元素。
    pub fn add_all(&self, keys: &[&str]) {
        let mut items = self.items.lock().unwrap();
        for key in keys {
            items.insert(key.to_string());
        }
    }

    /// 检查是否包含所有指定元素。
    pub fn contains_all(&self, keys: &[&str]) -> bool {
        let items = self.items.lock().unwrap();
        keys.iter().all(|key| items.contains(*key))
    }

    /// 与另一个 ManagedSet 求交集（就地修改）。
    pub fn retain_all(&self, other: &ManagedSet) {
        let other_items = other.items.lock().unwrap().clone();
        let mut items = self.items.lock().unwrap();
        items.retain(|item| other_items.contains(item));
    }

    /// 与另一个 ManagedSet 求并集。
    pub fn union_with(&self, other: &ManagedSet) {
        let other_items = other.items.lock().unwrap().clone();
        let mut items = self.items.lock().unwrap();
        for item in other_items {
            items.insert(item);
        }
    }
}

impl Default for ManagedSet { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_contains() {
        let set = ManagedSet::new();
        assert!(set.add("a"));
        assert!(!set.add("a")); // 重复添加返回 false
        assert!(set.contains("a"));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn remove_element() {
        let set = ManagedSet::new();
        set.add("x");
        assert!(set.remove("x"));
        assert!(!set.contains("x"));
        assert!(set.is_empty());
    }

    #[test]
    fn to_vec_returns_all_elements() {
        let set = ManagedSet::new();
        set.add("alpha");
        set.add("beta");
        let mut v = set.to_vec();
        v.sort();
        assert_eq!(v, vec!["alpha", "beta"]);
    }

    #[test]
    fn clear_removes_all() {
        let set = ManagedSet::new();
        set.add("1");
        set.add("2");
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn add_all_inserts_multiple() {
        let set = ManagedSet::new();
        set.add_all(&["a", "b", "c"]);
        assert_eq!(set.len(), 3);
        assert!(set.contains("a"));
        assert!(set.contains("b"));
        assert!(set.contains("c"));
    }

    #[test]
    fn contains_all_checks_subset() {
        let set = ManagedSet::new();
        set.add_all(&["x", "y", "z"]);

        assert!(set.contains_all(&["x", "y"]));
        assert!(!set.contains_all(&["x", "missing"]));
    }

    #[test]
    fn retain_all_keeps_intersection() {
        let set1 = ManagedSet::new();
        set1.add_all(&["a", "b", "c"]);

        let set2 = ManagedSet::new();
        set2.add_all(&["b", "c", "d"]);

        set1.retain_all(&set2);
        assert_eq!(set1.len(), 2);
        assert!(set1.contains("b"));
        assert!(set1.contains("c"));
        assert!(!set1.contains("a"));
    }

    #[test]
    fn union_with_combines_sets() {
        let set1 = ManagedSet::new();
        set1.add_all(&["a", "b"]);

        let set2 = ManagedSet::new();
        set2.add_all(&["b", "c"]);

        set1.union_with(&set2);
        assert_eq!(set1.len(), 3);
        assert!(set1.contains("a"));
        assert!(set1.contains("b"));
        assert!(set1.contains("c"));
    }
}
