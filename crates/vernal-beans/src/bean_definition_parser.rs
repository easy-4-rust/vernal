//! BeanDefinitionParser — Bean 定义解析器。
use std::fmt;

/// Bean 定义解析器 trait。
pub trait BeanDefinitionParser: Send + Sync + fmt::Debug {
    fn parse(&self, element: &str) -> Option<String>;
}
