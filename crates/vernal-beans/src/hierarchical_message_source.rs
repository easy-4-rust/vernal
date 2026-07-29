use crate::message_source::MessageSource;
use std::sync::Arc;
pub trait HierarchicalMessageSource: MessageSource {
    fn get_parent_message_source(&self) -> Option<Arc<dyn MessageSource>>;
    fn set_parent_message_source(&mut self, parent: Option<Arc<dyn MessageSource>>);
}
