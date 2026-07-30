//! ParameterNameDiscoverer — 参数名发现器。
/// 参数名发现器 trait。
pub trait ParameterNameDiscoverer: Send + Sync {
    fn get_parameter_names(&self, method_name: &str) -> Vec<String>;
}
