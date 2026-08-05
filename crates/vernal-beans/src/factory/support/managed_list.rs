//! ManagedList — Spring 风格管理列表。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ManagedList`。
//!
//! 在 Spring 的 Bean 定义解析过程中，`ManagedList` 表示一个
//! `<list>` 元素。在解析阶段元素可以是未解析的运行时 Bean 引用，
//! 最终在 Bean 实例化时被解析为真实对象。

use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

/// Spring 风格管理列表。
///
/// 对应 Spring 的 `ManagedList`。
///
/// 表示 Bean 定义中 `<list>` 元素的值。元素类型可在构建时约束。
pub struct ManagedList {
    items: Mutex<Vec<Arc<dyn Any + Send + Sync>>>,
    element_type: Option<TypeId>,
}

impl ManagedList {
    /// 创建空的管理列表。
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            element_type: None,
        }
    }

    /// 设置元素类型约束。
    pub fn with_element_type(mut self, type_id: TypeId) -> Self {
        self.element_type = Some(type_id);
        self
    }

    /// 添加元素。
    pub fn add(&self, item: Arc<dyn Any + Send + Sync>) {
        self.items.lock().unwrap().push(item);
    }

    /// 按索引获取元素。
    pub fn get(&self, index: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.items.lock().unwrap().get(index).map(Arc::clone)
    }

    /// 元素数量。
    pub fn len(&self) -> usize {
        self.items.lock().unwrap().len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.items.lock().unwrap().is_empty()
    }

    /// 获取元素类型约束。
    pub fn element_type(&self) -> Option<TypeId> {
        self.element_type
    }

    /// 将所有元素收集为 `Vec`。
    pub fn to_vec(&self) -> Vec<Arc<dyn Any + Send + Sync>> {
        self.items.lock().unwrap().iter().map(Arc::clone).collect()
    }

    /// 清空列表。
    pub fn clear(&self) {
        self.items.lock().unwrap().clear();
    }

    /// 在指定位置插入元素。
    pub fn insert(&self, index: usize, item: Arc<dyn Any + Send + Sync>) {
        self.items.lock().unwrap().insert(index, item);
    }

    /// 移除指定位置的元素。
    pub fn remove(&self, index: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut items = self.items.lock().unwrap();
        if index < items.len() {
            Some(items.remove(index))
        } else {
            None
        }
    }

    /// 检查列表是否包含满足谓词的元素。
    pub fn contains_any<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Arc<dyn Any + Send + Sync>) -> bool,
    {
        self.items
            .lock()
            .unwrap()
            .iter()
            .any(|item| predicate(item))
    }

    /// 对每个元素执行操作。
    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&Arc<dyn Any + Send + Sync>),
    {
        for item in self.items.lock().unwrap().iter() {
            f(item);
        }
    }
}

impl Default for ManagedList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_get_elements() {
        let list = ManagedList::new();
        list.add(Arc::new(1_i32));
        list.add(Arc::new("hello".to_string()));
        assert_eq!(list.len(), 2);
        assert_eq!(*list.get(0).unwrap().downcast_ref::<i32>().unwrap(), 1);
        assert_eq!(
            *list.get(1).unwrap().downcast_ref::<String>().unwrap(),
            "hello"
        );
    }

    #[test]
    fn empty_list_is_empty() {
        let list = ManagedList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.get(0).is_none());
    }

    #[test]
    fn element_type_constraint() {
        let list = ManagedList::new().with_element_type(TypeId::of::<String>());
        assert_eq!(list.element_type(), Some(TypeId::of::<String>()));
    }

    #[test]
    fn to_vec_returns_clone() {
        let list = ManagedList::new();
        list.add(Arc::new(42_u64));
        let v = list.to_vec();
        assert_eq!(v.len(), 1);
        assert_eq!(*v[0].downcast_ref::<u64>().unwrap(), 42);
    }

    #[test]
    fn clear_removes_all() {
        let list = ManagedList::new();
        list.add(Arc::new(1));
        list.add(Arc::new(2));
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn insert_at_position() {
        let list = ManagedList::new();
        list.add(Arc::new(1_i32));
        list.add(Arc::new(3_i32));
        list.insert(1, Arc::new(2_i32));

        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0).unwrap().downcast_ref::<i32>().unwrap(), 1);
        assert_eq!(*list.get(1).unwrap().downcast_ref::<i32>().unwrap(), 2);
        assert_eq!(*list.get(2).unwrap().downcast_ref::<i32>().unwrap(), 3);
    }

    #[test]
    fn remove_at_position() {
        let list = ManagedList::new();
        list.add(Arc::new(1_i32));
        list.add(Arc::new(2_i32));
        list.add(Arc::new(3_i32));

        let removed = list.remove(1).unwrap();
        assert_eq!(*removed.downcast_ref::<i32>().unwrap(), 2);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn remove_out_of_bounds_returns_none() {
        let list = ManagedList::new();
        list.add(Arc::new(1_i32));
        assert!(list.remove(5).is_none());
    }

    #[test]
    fn contains_any_finds_matching() {
        let list = ManagedList::new();
        list.add(Arc::new(1_i32));
        list.add(Arc::new(2_i32));
        list.add(Arc::new(3_i32));

        assert!(list.contains_any(|item| { item.downcast_ref::<i32>().map_or(false, |&v| v > 2) }));
        assert!(
            !list.contains_any(|item| { item.downcast_ref::<i32>().map_or(false, |&v| v > 10) })
        );
    }
}
