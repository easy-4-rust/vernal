//! DefaultParameterNameDiscoverer — 默认参数名发现器。
use crate::parameter_name_discoverer::ParameterNameDiscoverer;

/// 默认参数名发现器。
#[derive(Clone, Debug, Default)]
pub struct DefaultParameterNameDiscoverer;
impl DefaultParameterNameDiscoverer {
    pub fn new() -> Self { Self }
}
impl ParameterNameDiscoverer for DefaultParameterNameDiscoverer {
    fn get_parameter_names(&self, _method_name: &str) -> Vec<String> { Vec::new() }
}
