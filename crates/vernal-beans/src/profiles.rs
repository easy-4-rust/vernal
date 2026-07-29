//! Profiles — Spring 风格的 Profile 管理。
use crate::profile::Profile;
use std::collections::HashSet;

/// Profile 管理。
#[derive(Clone, Debug, Default)]
pub struct Profiles {
    active: HashSet<Profile>,
    default_profile_name: String,
}
impl Profiles {
    pub fn new() -> Self { Self { active: HashSet::new(), default_profile_name: "default".to_string() } }
    pub fn contains(&self, name: &str) -> bool { self.active.iter().any(|p| p.name() == name) }
    pub fn is_active(&self, name: &str) -> bool { self.contains(name) }
    pub fn add(&mut self, profile: Profile) { self.active.insert(profile); }
    pub fn remove(&mut self, name: &str) { self.active.retain(|p| p.name() != name); }
    pub fn get_active_profiles(&self) -> Vec<&str> { self.active.iter().map(|p| p.name()).collect() }
}
