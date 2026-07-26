//! 类型比较器 trait。
//!
//! 对标 Spring 的 `TypeComparator`。

use super::typed_value::TypedValue;

/// 类型比较器 trait。
///
/// 比较两个值的大小。
/// 对标 Spring 的 `org.springframework.expression.TypeComparator`。
pub trait TypeComparator: Send + Sync {
    /// 是否可以比较两个值。
    fn can_compare(&self, left: &TypedValue, right: &TypedValue) -> bool;

    /// 比较两个值。返回 std::cmp::Ordering。
    fn compare(&self, left: &TypedValue, right: &TypedValue) -> Result<std::cmp::Ordering, String>;
}
