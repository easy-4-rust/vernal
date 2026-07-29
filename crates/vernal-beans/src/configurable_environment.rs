//! ConfigurableEnvironment trait — 可配置环境接口。
use crate::environment::Environment;
use crate::property_source::PropertySource;

/// 可配置环境 trait。
pub trait ConfigurableEnvironment: Environment {
    fn get_property_sources(&self) -> &crate::property_sources::PropertySources;
    fn get_system_properties(&self) -> std::collections::HashMap<String, String>;
    fn get_system_environment(&self) -> std::collections::HashMap<String, String>;
    fn add_active_profile(&mut self, profile: &str);
}
