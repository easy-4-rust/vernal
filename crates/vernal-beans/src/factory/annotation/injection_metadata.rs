//! InjectionMetadata — Spring 风格的注入元数据。
use std::any::Any;

/// Spring 风格的注入元数据。
#[derive(Debug, Default)]
pub struct InjectionMetadata {
    /// pub。
    pub elements: Vec<Box<dyn Any + Send + Sync>>,
}

impl InjectionMetadata {
    /// 创建一个新的实例。
    pub fn new() -> Self { Self::default() }
    /// 添加元素。
    pub fn add_element(&mut self, element: Box<dyn Any + Send + Sync>) { self.elements.push(element); }
    /// 获取元素数量。
    pub fn element_count(&self) -> usize { self.elements.len() }
    /// 判断是否empty。
    pub fn is_empty(&self) -> bool { self.elements.is_empty() }
    /// 移除。
    pub fn clear(&mut self) { self.elements.clear(); }
}
