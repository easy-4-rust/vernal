//! ManagedSet — Spring 风格的托管集合。
#[derive(Clone, Debug, Default)]
pub struct ManagedSet<T> { items: std::collections::HashSet<T> }
impl<T: std::hash::Hash + Eq> ManagedSet<T> {
    pub fn new() -> Self { Self { items: std::collections::HashSet::new() } }
    pub fn from_hashset(items: std::collections::HashSet<T>) -> Self { Self { items } }
    pub fn insert(&mut self, item: T) -> bool { self.items.insert(item) }
    pub fn contains(&self, item: &T) -> bool { self.items.contains(item) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}
