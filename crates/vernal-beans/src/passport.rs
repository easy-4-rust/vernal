use std::collections::{HashMap, HashSet};
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Passport {
    principal: String,
    authenticated: bool,
    authorities: HashSet<String>,
    attributes: HashMap<String, String>,
}
impl Passport {
    pub fn new(principal: impl Into<String>) -> Self {
        Self {
            principal: principal.into(),
            ..Self::default()
        }
    }
    pub fn principal(&self) -> &str {
        &self.principal
    }
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
    pub fn set_authenticated(&mut self, v: bool) {
        self.authenticated = v;
    }
    pub fn grant(&mut self, a: impl Into<String>) -> bool {
        self.authorities.insert(a.into())
    }
    pub fn revoke(&mut self, a: &str) -> bool {
        self.authorities.remove(a)
    }
    pub fn has_authority(&self, a: &str) -> bool {
        self.authorities.contains(a)
    }
    pub fn set_attribute(&mut self, k: impl Into<String>, v: impl Into<String>) -> Option<String> {
        self.attributes.insert(k.into(), v.into())
    }
    pub fn attribute(&self, k: &str) -> Option<&str> {
        self.attributes.get(k).map(String::as_str)
    }
}
