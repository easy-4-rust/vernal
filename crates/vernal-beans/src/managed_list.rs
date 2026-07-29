//! ManagedList — Spring 风格的托管列表。
#[derive(Clone, Debug, Default)]
pub struct ManagedList<T> { items: Vec<T> }
impl<T> ManagedList<T> {
    pub fn new() -> Self { Self { items: Vec::new() } }
    pub fn from_vec(items: Vec<T>) -> Self { Self { items } }
    pub fn into_vec(self) -> Vec<T> { self.items }
    pub fn push(&mut self, item: T) { self.items.push(item); }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = &T> { self.items.iter() }
}
