//! AutowireCandidateQualifier — Spring 风格自动装配候选限定符。

use std::collections::HashMap;
use std::sync::Mutex;

pub struct AutowireCandidateQualifier {
    qualifier_type: String,
    attributes: Mutex<HashMap<String, String>>,
}

impl AutowireCandidateQualifier {
    pub fn new(qualifier_type: String) -> Self {
        Self {
            qualifier_type,
            attributes: Mutex::new(HashMap::new()),
        }
    }
    pub fn qualifier_type(&self) -> &str { &self.qualifier_type }
    pub fn set_attribute(&self, name: String, value: String) {
        self.attributes.lock().unwrap().insert(name, value);
    }
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        self.attributes.lock().unwrap().get(name).cloned()
    }
    pub fn attribute_count(&self) -> usize {
        self.attributes.lock().unwrap().len()
    }
}
