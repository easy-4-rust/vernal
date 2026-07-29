//! BeanWiringInfo — Spring 风格的 Bean 配置信息。
use std::fmt;

/// Spring 风格的 Bean 配置信息。
#[derive(Clone, Debug)]
pub struct BeanWiringInfo {
    pub bean_name: String,
    pub type_name: String,
    pub is_default_dependency: bool,
}

impl BeanWiringInfo {
    pub fn new(bean_name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self { bean_name: bean_name.into(), type_name: type_name.into(), is_default_dependency: true }
    }
    pub fn with_type_name(type_name: impl Into<String>) -> Self {
        Self { bean_name: String::new(), type_name: type_name.into(), is_default_dependency: true }
    }
    pub fn get_bean_name(&self) -> &str { &self.bean_name }
    pub fn get_type_name(&self) -> &str { &self.type_name }
    pub fn is_default_dependency(&self) -> bool { self.is_default_dependency }
}

impl fmt::Display for BeanWiringInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[bean={}; type={}]", self.bean_name, self.type_name)
    }
}
