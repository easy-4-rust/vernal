//! BeanDefinitionResource — Spring 风格 Bean 定义资源。

#[derive(Debug, Clone)]
pub struct BeanDefinitionResource {
    description: String,
    content: String,
}

impl BeanDefinitionResource {
    pub fn new(description: String, content: String) -> Self {
        Self { description, content }
    }
    pub fn description(&self) -> &str { &self.description }
    pub fn content(&self) -> &str { &self.content }
    pub fn content_len(&self) -> usize { self.content.len() }
    pub fn is_empty(&self) -> bool { self.content.is_empty() }
}
