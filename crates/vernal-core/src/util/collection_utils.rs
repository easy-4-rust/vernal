//! 集合工具。
//!
//! 对标 Spring `org.springframework.util.CollectionUtils`。
//!
//! 只实现 Rust std 中没有但 Spring 中常用的方法。
//! Spring 的方法签名多数依赖 Java `Collection` / `Map` 抽象,Rust std 已有等价物:
//!
//! - `isEmpty(Collection)` → Rust `is_empty()`
//! - `containsInstance(Collection, T)` → Rust `iter().any(|x| x == &t)`
//! - `mergeArrayIntoCollection` → Rust `extend()`
//! - `mergePropertiesIntoMap` → Rust `extend()`

use std::collections::HashMap;
use std::hash::Hash;

/// 集合工具。
///
/// 对标 Spring `CollectionUtils`。
pub struct CollectionUtils;

impl CollectionUtils {
    /// 检查切片是否为空或 `None`。
    ///
    /// 对标 Spring `CollectionUtils.isEmpty(Collection)`(支持 null)。
    #[must_use]
    pub fn is_empty<T>(collection: Option<&[T]>) -> bool {
        collection.is_none_or(<[T]>::is_empty)
    }

    /// 检查 `HashMap` 是否为空或 `None`。
    ///
    /// 对标 Spring `CollectionUtils.isEmpty(Map)`(支持 null)。
    #[must_use]
    pub fn is_empty_map<K, V>(map: Option<&HashMap<K, V>>) -> bool {
        map.is_none_or(std::collections::HashMap::is_empty)
    }

    /// 检查切片是否包含指定元素(基于引用相等)。
    ///
    /// 对标 Spring `CollectionUtils.containsInstance(Collection, Object)`。
    ///
    /// Rust 等价:`iter().any(|x| x == &element)`,本方法提供 Spring 兼容 API。
    #[must_use]
    pub fn contains_instance<T: PartialEq>(collection: &[T], element: &T) -> bool {
        collection.iter().any(|x| x == element)
    }

    /// 查找切片中第一个匹配的元素引用。
    ///
    /// 对标 Spring `CollectionUtils.findValueMatch(Collection, Predicate)`(简化版)。
    pub fn find_first_match<T, F>(collection: &[T], predicate: F) -> Option<&T>
    where
        F: Fn(&T) -> bool,
    {
        collection.iter().find(|x| predicate(*x))
    }

    /// 检查切片中是否存在**任意**元素匹配谓词。
    ///
    /// 对标 Spring `CollectionUtils.contains(Collection, Predicate)`。
    pub fn contains_match<T, F>(collection: &[T], predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        collection.iter().any(predicate)
    }

    /// 把切片的所有元素追加到目标 Vec。
    ///
    /// 对标 Spring `CollectionUtils.mergeArrayIntoCollection(Object[], Collection)`。
    pub fn merge_array_into_vec<S>(array: &[S], target: &mut Vec<S>)
    where
        S: Clone,
    {
        target.extend(array.iter().cloned());
    }

    /// 把源 `HashMap` 合并到目标 `HashMap`。
    ///
    /// 对标 Spring `CollectionUtils.mergePropertiesIntoMap(Properties, Map)`。
    pub fn merge_map_into_map<K, V>(source: &HashMap<K, V>, target: &mut HashMap<K, V>)
    where
        K: Clone + Eq + Hash,
        V: Clone,
    {
        for (k, v) in source {
            target.insert(k.clone(), v.clone());
        }
    }

    /// 检查两个切片是否包含相同元素(忽略顺序,允许重复)。
    ///
    /// 对标 Spring `CollectionUtils.containsAny(Collection, Collection)`。
    ///
    /// **注意**:本方法实现的是 `contains_any`(任意元素相交),
    /// 不是 Spring `containsAll`(全部包含)。Spring 这两个方法语义容易混淆。
    #[must_use]
    pub fn contains_any<T: PartialEq>(a: &[T], b: &[T]) -> bool {
        b.iter().any(|item| a.contains(item))
    }

    /// 检查 a 是否包含 b 的所有元素(允许 a 有额外元素)。
    ///
    /// 对标 Spring `CollectionUtils.containsAll(Collection, Collection)`。
    #[must_use]
    pub fn contains_all<T: PartialEq>(a: &[T], b: &[T]) -> bool {
        b.iter().all(|item| a.contains(item))
    }

    /// 返回切片中第一个重复出现的元素引用(基于 `PartialEq`)。
    ///
    /// 对标 Spring `CollectionUtils.findFirstMatch`(变体)。
    #[must_use]
    pub fn first_duplicate<T: PartialEq>(collection: &[T]) -> Option<&T> {
        for (i, item) in collection.iter().enumerate() {
            if collection[..i].iter().any(|x| x == item) {
                return Some(item);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_empty_none_returns_true() {
        let v: Option<&[i32]> = None;
        assert!(CollectionUtils::is_empty(v));
    }

    #[test]
    fn is_empty_empty_returns_true() {
        let v: Vec<i32> = vec![];
        assert!(CollectionUtils::is_empty(Some(&v)));
    }

    #[test]
    fn is_empty_nonempty_returns_false() {
        let v = vec![1, 2, 3];
        assert!(!CollectionUtils::is_empty(Some(&v)));
    }

    #[test]
    fn is_empty_map_basic() {
        let empty: HashMap<String, String> = HashMap::new();
        assert!(CollectionUtils::is_empty_map(Some(&empty)));
        assert!(CollectionUtils::is_empty_map::<String, String>(None));

        let mut m = HashMap::new();
        m.insert("k".to_string(), "v".to_string());
        assert!(!CollectionUtils::is_empty_map(Some(&m)));
    }

    #[test]
    fn contains_instance_basic() {
        let v = vec!["a", "b", "c"];
        assert!(CollectionUtils::contains_instance(&v, &"a"));
        assert!(CollectionUtils::contains_instance(&v, &"c"));
        assert!(!CollectionUtils::contains_instance(&v, &"x"));
    }

    #[test]
    fn find_first_match_basic() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(CollectionUtils::find_first_match(&v, |x| *x > 3), Some(&4));
        assert_eq!(CollectionUtils::find_first_match(&v, |x| *x > 10), None);
    }

    #[test]
    fn contains_match_basic() {
        let v = vec![1, 2, 3];
        assert!(CollectionUtils::contains_match(&v, |x| *x == 2));
        assert!(!CollectionUtils::contains_match(&v, |x| *x == 5));
    }

    #[test]
    fn merge_array_into_vec_basic() {
        let array = [1, 2, 3];
        let mut target = vec![4, 5];
        CollectionUtils::merge_array_into_vec(&array, &mut target);
        assert_eq!(target, vec![4, 5, 1, 2, 3]);
    }

    #[test]
    fn merge_map_into_map_basic() {
        let mut source = HashMap::new();
        source.insert("a".to_string(), 1);
        source.insert("b".to_string(), 2);

        let mut target = HashMap::new();
        target.insert("c".to_string(), 3);

        CollectionUtils::merge_map_into_map(&source, &mut target);
        assert_eq!(target.len(), 3);
        assert_eq!(target.get("a"), Some(&1));
        assert_eq!(target.get("c"), Some(&3));
    }

    #[test]
    fn contains_any_basic() {
        let a = vec!["a", "b", "c"];
        let b = vec!["x", "c"];
        assert!(CollectionUtils::contains_any(&a, &b));

        let c = vec!["x", "y"];
        assert!(!CollectionUtils::contains_any(&a, &c));
    }

    #[test]
    fn contains_all_basic() {
        let a = vec!["a", "b", "c", "d"];
        let b = vec!["a", "c"];
        assert!(CollectionUtils::contains_all(&a, &b));

        let c = vec!["a", "x"];
        assert!(!CollectionUtils::contains_all(&a, &c));
    }

    #[test]
    fn first_duplicate_basic() {
        let v = vec!["a", "b", "c", "a", "b"];
        assert_eq!(CollectionUtils::first_duplicate(&v), Some(&"a"));
    }

    #[test]
    fn first_duplicate_none_when_all_unique() {
        let v = vec!["a", "b", "c"];
        assert_eq!(CollectionUtils::first_duplicate(&v), None);
    }
}
