use crate::{
    hierarchical_message_source::HierarchicalMessageSource, message_source::MessageSource,
};
use std::sync::Arc;
#[derive(Default)]
pub struct DelegatingMessageSource {
    parent: Option<Arc<dyn MessageSource>>,
}
impl DelegatingMessageSource {
    pub fn new(parent: Option<Arc<dyn MessageSource>>) -> Self {
        Self { parent }
    }
}
impl MessageSource for DelegatingMessageSource {
    fn get_message(&self, code: &str, args: &[&str], locale: &str) -> Option<String> {
        self.parent
            .as_ref()
            .and_then(|p| p.get_message(code, args, locale))
    }
}
impl HierarchicalMessageSource for DelegatingMessageSource {
    fn get_parent_message_source(&self) -> Option<Arc<dyn MessageSource>> {
        self.parent.clone()
    }
    fn set_parent_message_source(&mut self, parent: Option<Arc<dyn MessageSource>>) {
        self.parent = parent;
    }
}
