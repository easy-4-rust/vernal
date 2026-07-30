//! Configurable — Spring 风格 @Configurable 注解标记。

#[derive(Debug, Clone)]
pub struct Configurable {
    enabled: bool,
    autowire: bool,
}

impl Configurable {
    pub fn new() -> Self { Self { enabled: true, autowire: true } }
    pub fn enabled(&self) -> bool { self.enabled }
    pub fn set_enabled(&mut self, v: bool) { self.enabled = v; }
    pub fn is_autowire(&self) -> bool { self.autowire }
    pub fn set_autowire(&mut self, v: bool) { self.autowire = v; }
}
impl Default for Configurable { fn default() -> Self { Self::new() } }
