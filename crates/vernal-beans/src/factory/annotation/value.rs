//! Value — Spring 风格 @Value 注解标记。

pub struct Value {
    value: String,
}

impl Value {
    pub fn new(value: String) -> Self { Self { value } }
    pub fn value(&self) -> &str { &self.value }
    pub fn len(&self) -> usize { self.value.len() }
    pub fn is_empty(&self) -> bool { self.value.is_empty() }
}
