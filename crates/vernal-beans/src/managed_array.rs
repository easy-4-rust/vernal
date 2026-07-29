//! ManagedArray — Spring 风格的托管数组。
#[derive(Clone, Debug, Default)]
pub struct ManagedArray<T> { items: Vec<T> }
impl<T> ManagedArray<T> {
    pub fn new() -> Self { Self { items: Vec::new() } }
    pub fn from_vec(items: Vec<T>) -> Self { Self { items } }
    pub fn into_vec(self) -> Vec<T> { self.items }
    pub fn push(&mut self, item: T) { self.items.push(item); }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}
