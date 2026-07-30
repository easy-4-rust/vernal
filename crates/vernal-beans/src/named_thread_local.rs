//! NamedThreadLocal — 命名线程本地变量。
use std::cell::RefCell;
use std::marker::PhantomData;

/// 命名线程本地变量。
pub struct NamedThreadLocal<T: 'static> {
    name: &'static str,
    _marker: PhantomData<T>,
}
impl<T: 'static> NamedThreadLocal<T> {
    pub fn new(name: &'static str) -> Self { Self { name, _marker: PhantomData } }
    pub fn name(&self) -> &'static str { self.name }
}
