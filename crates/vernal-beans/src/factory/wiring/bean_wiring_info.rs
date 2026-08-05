//! BeanWiringInfo — Spring 风格的 Bean 配置信息。
use std::fmt;

/// Spring 风格的 Bean 配置信息。
#[derive(Clone, Debug)]
pub struct BeanWiringInfo {
    /// pub。
    pub bean_name: String,
    /// pub。
    pub type_name: String,
    /// pub。
    pub is_default_dependency: bool,
}

impl BeanWiringInfo {
    /// 创建一个新的实例。
    pub fn new(bean_name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            bean_name: bean_name.into(),
            type_name: type_name.into(),
            is_default_dependency: true,
        }
    }
    /// 执行with_type_name操作。
    pub fn with_type_name(type_name: impl Into<String>) -> Self {
        Self {
            bean_name: String::new(),
            type_name: type_name.into(),
            is_default_dependency: true,
        }
    }
    /// 获取Bean名称。
    pub fn get_bean_name(&self) -> &str {
        &self.bean_name
    }
    /// 获取类型名称。
    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }
    /// 判断是否默认依赖。
    pub fn is_default_dependency(&self) -> bool {
        self.is_default_dependency
    }
}

impl fmt::Display for BeanWiringInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[bean={}; type={}]", self.bean_name, self.type_name)
    }
}
