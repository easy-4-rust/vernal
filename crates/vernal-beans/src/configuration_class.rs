//! ConfigurationClass — 配置类。
#[derive(Clone, Debug)]
pub struct ConfigurationClass {
    pub class_name: String,
    pub bean_methods: Vec<String>,
}
impl ConfigurationClass {
    pub fn new(class_name: impl Into<String>) -> Self {
        Self { class_name: class_name.into(), bean_methods: Vec::new() }
    }
    pub fn class_name(&self) -> &str { &self.class_name }
    pub fn add_bean_method(&mut self, method: impl Into<String>) {
        self.bean_methods.push(method.into());
    }
    pub fn bean_methods(&self) -> &[String] { &self.bean_methods }
}
