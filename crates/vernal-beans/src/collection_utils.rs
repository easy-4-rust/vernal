//! CollectionUtils — 集合工具。
/// 集合工具。
pub struct CollectionUtils;
impl CollectionUtils {
    pub fn is_empty<T>(slice: &[T]) -> bool { slice.is_empty() }
    pub fn size<T>(slice: &[T]) -> usize { slice.len() }
    pub fn contains<T: PartialEq>(slice: &[T], item: &T) -> bool { slice.contains(item) }
}
