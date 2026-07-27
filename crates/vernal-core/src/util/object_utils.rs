//! 对象工具。
//!
//! 对标 Spring `org.springframework.util.ObjectUtils`。
//!
//! 只实现 Rust std 中没有但 Spring 中常用的方法。

/// 对象工具。
///
/// 对标 Spring `ObjectUtils`。
pub struct ObjectUtils;

impl ObjectUtils {
    /// 检查对象是否为 null/空(对标 Spring `ObjectUtils.isEmpty(Object)`)。
    ///
    /// Rust 中对应规则:
    /// - `None` → true
    /// - `Some("")` → true
    /// - `Some("non-empty")` → false
    /// - 空集合 → true(通过 `is_empty` trait)
    #[must_use]
    pub fn is_empty_option_str(s: Option<&str>) -> bool {
        s.is_none_or(str::is_empty)
    }

    /// 检查元素是否包含在数组中。
    ///
    /// 对标 Spring `ObjectUtils.containsElement(Object[], Object)`。
    #[must_use]
    pub fn contains_element<T: PartialEq>(array: &[T], element: &T) -> bool {
        array.iter().any(|x| x == element)
    }

    /// 检查两个对象是否相等(包括都是 None 的情况)。
    ///
    /// 对标 Spring `ObjectUtils.nullSafeEquals(Object, Object)`。
    #[must_use]
    pub fn null_safe_equals<T: PartialEq>(a: Option<&T>, b: Option<&T>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    /// 计算对象的 hashCode(通过 std Hash trait)。
    ///
    /// 对标 Spring `ObjectUtils.nullSafeHashCode(Object)`。
    #[must_use]
    pub fn null_safe_hash<T: std::hash::Hash>(obj: Option<&T>) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        match obj {
            Some(o) => {
                1u8.hash(&mut hasher); // null 标志位
                o.hash(&mut hasher);
            }
            None => {
                0u8.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// 获取对象 IdentityHashCode(用指针地址)。
    ///
    /// 对标 Spring `ObjectUtils.getIdentityHexString(Object)`(用 `System.identityHashCode`)。
    #[must_use]
    pub fn identity_address<T>(obj: &T) -> usize {
        std::ptr::from_ref::<T>(obj) as usize
    }

    /// 把切片用指定分隔符拼接为字符串。
    ///
    /// 对标 Spring `ObjectUtils.nullSafeToString(Object[])` + `arrayToDelimitedString`。
    #[must_use]
    pub fn array_to_delimited_string<T: std::fmt::Debug>(array: &[T], delimiter: &str) -> String {
        array
            .iter()
            .map(|x| format!("{x:?}"))
            .collect::<Vec<_>>()
            .join(delimiter)
    }

    /// 比较两个可空对象(对标 Spring `ObjectUtils.nullSafeComparator`)。
    ///
    /// 返回值:None 比 Some 小;Some 之间用 `Ord` 比较。
    #[must_use]
    pub fn null_safe_compare<T: Ord>(a: Option<&T>, b: Option<&T>) -> std::cmp::Ordering {
        match (a, b) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_empty_option_str_basic() {
        assert!(ObjectUtils::is_empty_option_str(None));
        assert!(ObjectUtils::is_empty_option_str(Some("")));
        assert!(!ObjectUtils::is_empty_option_str(Some("hello")));
    }

    #[test]
    fn contains_element_basic() {
        let v = vec!["a", "b", "c"];
        assert!(ObjectUtils::contains_element(&v, &"a"));
        assert!(!ObjectUtils::contains_element(&v, &"x"));
    }

    #[test]
    fn null_safe_equals_both_none() {
        let a: Option<&i32> = None;
        let b: Option<&i32> = None;
        assert!(ObjectUtils::null_safe_equals(a, b));
    }

    #[test]
    fn null_safe_equals_one_none() {
        let a: Option<&i32> = Some(&1);
        let b: Option<&i32> = None;
        assert!(!ObjectUtils::null_safe_equals(a, b));
    }

    #[test]
    fn null_safe_equals_both_some_equal() {
        assert!(ObjectUtils::null_safe_equals(Some(&1), Some(&1)));
    }

    #[test]
    fn null_safe_equals_both_some_different() {
        assert!(!ObjectUtils::null_safe_equals(Some(&1), Some(&2)));
    }

    #[test]
    fn null_safe_hash_none_vs_some_different() {
        let none: Option<&i32> = None;
        let some: Option<&i32> = Some(&1);
        assert_ne!(
            ObjectUtils::null_safe_hash(none),
            ObjectUtils::null_safe_hash(some)
        );
    }

    #[test]
    fn null_safe_hash_same_value_equal() {
        let a: Option<&i32> = Some(&42);
        let b: Option<&i32> = Some(&42);
        assert_eq!(
            ObjectUtils::null_safe_hash(a),
            ObjectUtils::null_safe_hash(b)
        );
    }

    #[test]
    fn identity_address_is_stable_for_same_object() {
        let x = 42;
        let addr1 = ObjectUtils::identity_address(&x);
        let addr2 = ObjectUtils::identity_address(&x);
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn array_to_delimited_string_basic() {
        let arr = [1, 2, 3];
        assert_eq!(
            ObjectUtils::array_to_delimited_string(&arr, ", "),
            "1, 2, 3"
        );
    }

    #[test]
    fn null_safe_compare_basic() {
        use std::cmp::Ordering;
        assert_eq!(
            ObjectUtils::null_safe_compare::<i32>(None, None),
            Ordering::Equal
        );
        assert_eq!(
            ObjectUtils::null_safe_compare(Some(&1), Some(&2)),
            Ordering::Less
        );
        assert_eq!(
            ObjectUtils::null_safe_compare(Some(&2), Some(&1)),
            Ordering::Greater
        );
        assert_eq!(
            ObjectUtils::null_safe_compare::<i32>(None, Some(&1)),
            Ordering::Less
        );
        assert_eq!(
            ObjectUtils::null_safe_compare::<i32>(Some(&1), None),
            Ordering::Greater
        );
    }
}
