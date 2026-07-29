//! ManagedMap — Spring 风格的托管映射。
#[derive(Clone, Debug, Default)]
pub struct ManagedMap<K, V> { items: std::collections::HashMap<K, V> }
impl<K: std::hash::Hash + Eq, V> ManagedMap<K, V> {
    pub fn new() -> Self { Self { items: std::collections::HashMap::new() } }
    pub fn from_hashmap(items: std::collections::HashMap<K, V>) -> Self { Self { items } }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> { self.items.insert(key, value) }
    pub fn get(&self, key: &K) -> Option<&V> { self.items.get(key) }
    pub fn contains_key(&self, key: &K) -> bool { self.items.contains_key(key) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}
