//! 环境抽象模块。
//!
//! 对标 Spring `org.springframework.core.env` 包：PropertySource 抽象、Environment 抽象、
//! Profile/Properties 语义。本模块仅定义 trait/struct，不实现细节策略。

mod abstract_environment;
mod abstract_property_resolver;
mod command_line_args;
mod command_line_property_source;
mod composite_property_source;
mod configurable_environment;
mod configurable_property_resolver;
mod environment;
mod environment_capable;
mod enumerable_property_source;
mod map_property_source;
mod missing_required_properties_exception;
mod mutable_property_sources;
mod profiles;
mod profiles_parser;
mod properties_property_source;
mod property_resolver;
mod property_source;
mod property_sources;
mod property_sources_property_resolver;
mod simple_command_line_args_parser;
mod simple_command_line_property_source;
mod standard_environment;
mod system_environment_property_source;

pub use abstract_environment::AbstractEnvironment;
pub use abstract_property_resolver::AbstractPropertyResolver;
pub use command_line_args::CommandLineArgs;
pub use command_line_property_source::CommandLinePropertySource;
pub use composite_property_source::CompositePropertySource;
pub use configurable_environment::ConfigurableEnvironment;
pub use configurable_property_resolver::ConfigurablePropertyResolver;
pub use environment::Environment;
pub use environment_capable::EnvironmentCapable;
pub use enumerable_property_source::EnumerablePropertySource;
pub use map_property_source::MapPropertySource;
pub use missing_required_properties_exception::MissingRequiredPropertiesException;
pub use mutable_property_sources::MutablePropertySources;
pub use profiles::Profiles;
pub use profiles_parser::ProfilesParser;
pub use properties_property_source::PropertiesPropertySource;
pub use property_resolver::PropertyResolver;
pub use property_source::PropertySource;
pub use property_sources::PropertySources;
pub use property_sources_property_resolver::PropertySourcesPropertyResolver;
pub use simple_command_line_args_parser::SimpleCommandLineArgsParser;
pub use simple_command_line_property_source::SimpleCommandLinePropertySource;
pub use standard_environment::StandardEnvironment;
pub use system_environment_property_source::SystemEnvironmentPropertySource;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn map_property_source_basic() {
        let mut properties = HashMap::new();
        properties.insert("key1".to_string(), "value1".to_string());
        properties.insert("key2".to_string(), "value2".to_string());

        let source = MapPropertySource::new("test", properties);
        assert_eq!(source.name(), "test");
        assert_eq!(source.get_property("key1"), Some("value1".to_string()));
        assert_eq!(source.get_property("key2"), Some("value2".to_string()));
        assert_eq!(source.get_property("key3"), None);
    }

    #[test]
    fn system_environment_property_source() {
        let source = SystemEnvironmentPropertySource::new("env");
        // PATH 环境变量应该存在
        assert!(source.get_property("PATH").is_some());
    }

    #[test]
    fn property_sources_priority() {
        let mut sources = MutablePropertySources::new();

        let mut low = HashMap::new();
        low.insert("key".to_string(), "low".to_string());
        sources.add_last(Box::new(MapPropertySource::new("low", low)));

        let mut high = HashMap::new();
        high.insert("key".to_string(), "high".to_string());
        sources.add_first(Box::new(MapPropertySource::new("high", high)));

        assert_eq!(sources.get_property("key"), Some("high".to_string()));
    }

    #[test]
    fn standard_environment_has_system_properties() {
        let env = StandardEnvironment::new();
        // 系统属性应该存在
        assert!(env.get_property("user.home").is_some() || env.get_property("user.name").is_some());
    }

    #[test]
    fn standard_environment_default_profiles() {
        let env = StandardEnvironment::new();
        assert_eq!(env.get_default_profiles(), vec!["default".to_string()]);
    }

    #[test]
    fn standard_environment_active_profiles() {
        let mut env = StandardEnvironment::new();
        env.set_active_profiles(vec!["dev".to_string(), "test".to_string()]);
        assert_eq!(env.get_active_profiles(), vec!["dev".to_string(), "test".to_string()]);
        assert!(env.accepts_profiles(&["dev"]));
        assert!(!env.accepts_profiles(&["prod"]));
    }

    #[test]
    fn property_sources_add_first() {
        let mut sources = MutablePropertySources::new();
        let mut low = HashMap::new();
        low.insert("key".to_string(), "low".to_string());
        sources.add_last(Box::new(MapPropertySource::new("low", low)));

        let mut high = HashMap::new();
        high.insert("key".to_string(), "high".to_string());
        sources.add_first(Box::new(MapPropertySource::new("high", high)));

        assert_eq!(sources.get_property("key"), Some("high".to_string()));
    }

    #[test]
    fn property_sources_add_last() {
        let mut sources = MutablePropertySources::new();
        let mut first = HashMap::new();
        first.insert("key".to_string(), "first".to_string());
        sources.add_last(Box::new(MapPropertySource::new("first", first)));

        let mut second = HashMap::new();
        second.insert("key".to_string(), "second".to_string());
        sources.add_last(Box::new(MapPropertySource::new("second", second)));

        // 先添加的优先级更高
        assert_eq!(sources.get_property("key"), Some("first".to_string()));
    }

    #[test]
    fn property_sources_len() {
        let mut sources = MutablePropertySources::new();
        assert_eq!(sources.len(), 0);
        assert!(sources.is_empty());

        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        sources.add_last(Box::new(MapPropertySource::new("test", props)));
        assert_eq!(sources.len(), 1);
        assert!(!sources.is_empty());
    }

    #[test]
    fn property_sources_contains_property() {
        let mut sources = MutablePropertySources::new();
        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        sources.add_last(Box::new(MapPropertySource::new("test", props)));

        assert!(sources.contains_property("key"));
        assert!(!sources.contains_property("missing"));
    }

    #[test]
    fn property_sources_get_property_missing() {
        let sources = MutablePropertySources::new();
        assert_eq!(sources.get_property("missing"), None);
    }

    #[test]
    fn map_property_source_set_property() {
        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        let mut source = MapPropertySource::new("test", props);
        assert_eq!(source.get_property("key"), Some("value".to_string()));

        source.set_property("key", "new_value");
        assert_eq!(source.get_property("key"), Some("new_value".to_string()));
    }

    #[test]
    fn map_property_source_remove_property() {
        let mut props = HashMap::new();
        props.insert("key".to_string(), "value".to_string());
        let mut source = MapPropertySource::new("test", props);
        assert!(source.contains_property("key"));

        source.remove_property("key");
        assert!(!source.contains_property("key"));
    }

    #[test]
    fn system_environment_property_source_contains_path() {
        let source = SystemEnvironmentPropertySource::new("env");
        assert!(source.get_property("PATH").is_some());
    }

    #[test]
    fn standard_environment_get_property() {
        let env = StandardEnvironment::new();
        // 系统属性应该存在
        assert!(env.get_property("user.home").is_some() || env.get_property("user.name").is_some());
    }

    #[test]
    fn standard_environment_get_property_with_default() {
        let env = StandardEnvironment::new();
        let value = env.get_property_with_default("nonexistent_key", "default_value");
        assert_eq!(value, "default_value");
    }

    #[test]
    fn standard_environment_contains_property() {
        let env = StandardEnvironment::new();
        assert!(env.contains_property("PATH"));
        assert!(!env.contains_property("nonexistent_key_12345"));
    }
}
