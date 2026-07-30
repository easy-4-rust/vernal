//! Lookup — Spring 风格 @Lookup 注解标记。

pub struct Lookup {
    value: String,
}

impl Lookup {
    pub fn new(value: String) -> Self { Self { value } }
    pub fn value(&self) -> &str { &self.value }
}
