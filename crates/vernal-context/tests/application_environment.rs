//! Context-local PropertySource、Profile 与占位符解析合同测试。

use std::sync::Arc;

use vernal_context::{ApplicationEnvironment, EnvironmentError, MapPropertySource, PropertySource};

/// 模拟远端配置中心读取失败，验证 Environment 不会静默降级到低优先级值。
struct FailingPropertySource;

impl PropertySource for FailingPropertySource {
    fn name(&self) -> &'static str {
        "remote"
    }

    fn get(&self, _key: &str) -> Result<Option<String>, EnvironmentError> {
        Err(EnvironmentError::PropertySource {
            source_name: String::from("remote"),
            source: Arc::new(std::io::Error::other("remote credential must stay private")),
        })
    }
}

/// 把具体 Map 来源转换为框架中立 `PropertySource`。
fn source(
    name: &str,
    values: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> Arc<dyn PropertySource> {
    Arc::new(MapPropertySource::new(name, values).expect("valid map property source"))
}

#[test]
fn explicit_source_order_controls_property_precedence() {
    let mut builder = ApplicationEnvironment::builder();
    builder
        .add_last(source(
            "application",
            [("service.port", "8080"), ("service.name", "vernal")],
        ))
        .expect("base source")
        .add_first(source("command-line", [("service.port", "9090")]))
        .expect("override source");
    let environment = builder.build();

    assert_eq!(
        environment.property("service.port").expect("property"),
        Some(String::from("9090"))
    );
    assert_eq!(
        environment.property("service.name").expect("property"),
        Some(String::from("vernal"))
    );
    assert_eq!(
        environment.property_source_names().collect::<Vec<_>>(),
        ["command-line", "application"]
    );
}

#[test]
fn typed_lookup_and_required_properties_are_structured_and_redacted() {
    let mut builder = ApplicationEnvironment::builder();
    builder
        .add_last(source(
            "application",
            [
                ("service.port", "8088"),
                ("service.enabled", "true"),
                ("database.url", "postgres://secret-user:secret-password@db"),
            ],
        ))
        .expect("source");
    let environment = builder.build();

    assert_eq!(
        environment
            .require::<u16>("service.port")
            .expect("typed port"),
        8088
    );
    assert_eq!(
        environment
            .get::<bool>("service.enabled")
            .expect("typed flag"),
        Some(true)
    );
    assert!(matches!(
        environment.require::<String>("missing.key"),
        Err(EnvironmentError::MissingProperty { key }) if key == "missing.key"
    ));

    let error = environment
        .require::<u64>("database.url")
        .expect_err("URL must not parse as an integer");
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("database.url"));
    assert!(diagnostic.contains("u64"));
    assert!(!diagnostic.contains("secret-user"));
    assert!(!diagnostic.contains("secret-password"));
    assert!(!format!("{environment:?}").contains("postgres://"));
}

#[test]
fn placeholders_resolve_across_sources_with_nested_defaults() {
    let mut builder = ApplicationEnvironment::builder();
    builder
        .add_last(source(
            "application",
            [
                ("service.host", "127.0.0.1"),
                ("service.port", "8080"),
                ("service.endpoint", "http://${service.host}:${service.port}"),
                (
                    "service.health",
                    "${health.endpoint:${service.endpoint}/health}",
                ),
                ("service.raw", "${service.host}"),
            ],
        ))
        .expect("source");
    let environment = builder.build();

    assert_eq!(
        environment.property("service.endpoint").expect("endpoint"),
        Some(String::from("http://127.0.0.1:8080"))
    );
    assert_eq!(
        environment.property("service.health").expect("health"),
        Some(String::from("http://127.0.0.1:8080/health"))
    );
    assert_eq!(
        environment.raw_property("service.raw").expect("raw"),
        Some(String::from("${service.host}"))
    );
}

#[test]
fn placeholder_failures_report_keys_without_values() {
    let mut builder = ApplicationEnvironment::builder();
    builder
        .add_last(source(
            "application",
            [
                ("cycle.a", "${cycle.b}"),
                ("cycle.b", "${cycle.a}"),
                ("missing.ref", "${does.not.exist}"),
                ("malformed", "${not.closed"),
            ],
        ))
        .expect("source");
    let environment = builder.build();

    assert!(matches!(
        environment.property("cycle.a"),
        Err(EnvironmentError::CircularPlaceholder { path })
            if path == ["cycle.a", "cycle.b", "cycle.a"]
    ));
    assert!(matches!(
        environment.property("missing.ref"),
        Err(EnvironmentError::UnresolvedPlaceholder {
            property,
            placeholder,
        }) if property == "missing.ref" && placeholder == "does.not.exist"
    ));
    assert!(matches!(
        environment.property("malformed"),
        Err(EnvironmentError::MalformedPlaceholder { property })
            if property == "malformed"
    ));
}

#[test]
fn active_profiles_replace_defaults_and_remain_context_local() {
    let default_environment = ApplicationEnvironment::empty();
    assert_eq!(
        default_environment.effective_profiles().collect::<Vec<_>>(),
        ["default"]
    );
    assert!(
        default_environment
            .is_profile_active("default")
            .expect("valid profile")
    );

    let mut builder = ApplicationEnvironment::builder();
    builder
        .default_profile("local")
        .expect("default profile")
        .active_profile("production")
        .expect("active profile")
        .active_profile("observability")
        .expect("second active profile");
    let environment = builder.build();

    assert_eq!(
        environment.effective_profiles().collect::<Vec<_>>(),
        ["observability", "production"]
    );
    assert!(!environment.is_profile_active("default").expect("profile"));
    assert!(
        environment
            .is_profile_active("production")
            .expect("profile")
    );
    assert!(matches!(
        environment.is_profile_active("invalid profile"),
        Err(EnvironmentError::InvalidProfile { .. })
    ));
}

#[test]
fn source_and_map_registration_reject_ambiguous_inputs_atomically() {
    let duplicate = MapPropertySource::new(
        "duplicate-map",
        [("service.port", "8080"), ("service.port", "9090")],
    );
    assert!(matches!(
        duplicate,
        Err(EnvironmentError::DuplicatePropertyKey {
            source_name,
            key,
        }) if source_name == "duplicate-map" && key == "service.port"
    ));

    let mut builder = ApplicationEnvironment::builder();
    builder
        .add_last(source("application", [("service.port", "8080")]))
        .expect("first source");
    assert!(matches!(
        builder.add_first(source("application", [("service.port", "9090")])),
        Err(EnvironmentError::DuplicatePropertySource { name })
            if name == "application"
    ));

    // 重复注册失败后，先前合法来源仍可被冻结并正常读取。
    let environment = builder.build();
    assert_eq!(
        environment.property("service.port").expect("property"),
        Some(String::from("8080"))
    );
}

#[test]
fn source_read_failure_is_structured_and_does_not_fall_through() {
    let mut builder = ApplicationEnvironment::builder();
    builder
        .add_first(Arc::new(FailingPropertySource))
        .expect("failing source has valid metadata")
        .add_last(source("fallback", [("service.port", "8080")]))
        .expect("fallback source");
    let environment = builder.build();

    let error = environment
        .property("service.port")
        .expect_err("source failure must remain observable");
    assert!(!error.to_string().contains("remote credential"));
    assert!(!format!("{error:?}").contains("remote credential"));
    assert!(matches!(
        error,
        EnvironmentError::PropertySource { source_name, .. }
            if source_name == "remote"
    ));
}
