//! ProxyFactory — 代理工厂。
use std::any::Any;
use std::sync::Arc;

/// 代理工厂。
#[derive(Clone, Debug)]
pub struct ProxyFactory {
    pub target: Option<Arc<dyn Any + Send + Sync>>,
    pub interfaces: Vec<String>,
}
impl ProxyFactory {
    pub fn new() -> Self { Self { target: None, interfaces: Vec::new() } }
    pub fn set_target(&mut self, target: Arc<dyn Any + Send + Sync>) { self.target = Some(target); }
    pub fn add_interface(&mut self, interface: impl Into<String>) { self.interfaces.push(interface.into()); }
    pub fn get_proxy(&self) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.target.clone().ok_or_else(|| "No target set".into())
    }
}
