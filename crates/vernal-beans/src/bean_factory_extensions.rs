//! BeanFactoryExtensions — BeanFactory 扩展方法。
use crate::bean_factory::BeanFactory;

/// BeanFactory 扩展方法。
pub trait BeanFactoryExtensions: BeanFactory {
    fn find_annotation_on_bean(&self, _bean_name: &str, _annotation_type: &str) -> bool { false }
    fn get_bean_names_for_annotation(&self, _annotation_type: &str) -> Vec<String> { Vec::new() }
}
