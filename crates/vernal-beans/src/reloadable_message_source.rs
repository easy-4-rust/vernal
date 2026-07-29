use crate::{
    hierarchical_message_source::HierarchicalMessageSource,
    message_source::{MessageMap, MessageSource, interpolate_message},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct ReloadableMessageSource {
    messages: RwLock<MessageMap>,
    parent: Option<Arc<dyn MessageSource>>,
}
impl ReloadableMessageSource {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn reload<I, K, V>(&self, locale: impl Into<String>, messages: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.messages
            .write()
            .expect("message lock poisoned")
            .insert(
                locale.into(),
                messages
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into()))
                    .collect::<HashMap<_, _>>(),
            );
    }
    pub fn clear(&self) {
        self.messages
            .write()
            .expect("message lock poisoned")
            .clear();
    }
}
impl MessageSource for ReloadableMessageSource {
    fn get_message(&self, code: &str, args: &[&str], locale: &str) -> Option<String> {
        self.messages
            .read()
            .expect("message lock poisoned")
            .get(locale)
            .and_then(|m| m.get(code))
            .map(|m| interpolate_message(m, args))
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.get_message(code, args, locale))
            })
    }
}
impl HierarchicalMessageSource for ReloadableMessageSource {
    fn get_parent_message_source(&self) -> Option<Arc<dyn MessageSource>> {
        self.parent.clone()
    }
    fn set_parent_message_source(&mut self, parent: Option<Arc<dyn MessageSource>>) {
        self.parent = parent;
    }
}
