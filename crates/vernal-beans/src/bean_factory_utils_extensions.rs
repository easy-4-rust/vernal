//! BeanFactoryUtilsExtensions — BeanFactory 工具扩展。
use crate::bean_factory::BeanFactory;

/// BeanFactory 工具扩展。
pub struct BeanFactoryUtilsExtensions;
impl BeanFactoryUtilsExtensions {
    pub fn is_factory_bean_by_name(factory: &dyn BeanFactory, name: &str) -> bool {
        name.starts_with('&') || name.contains("FactoryBean")
    }
}
