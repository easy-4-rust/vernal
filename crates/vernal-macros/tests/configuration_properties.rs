//! 类型安全配置对象派生、嵌套绑定与 `IoC` 注入合同测试。

use std::{error::Error, sync::Arc};

use vernal_context::{
    ApplicationEnvironment, ConfigurationProperties, ContextError, ContextState, MapPropertySource,
    VernalApplicationBuilder,
};
use vernal_beans::ComponentDefinition;

/// 缺少显式线程数时使用的应用默认值。
const fn default_worker_threads() -> usize {
    4
}

/// TLS 子配置；作为嵌套字段时根前缀会被父对象覆盖。
#[derive(Debug, vernal_macros::ConfigurationProperties)]
#[configuration(prefix = "tls")]
struct TlsProperties {
    #[configuration(default)]
    enabled: bool,
    certificate: Option<String>,
}

/// 模拟一个同时使用必填、可选、默认、改名、kebab-case 和嵌套字段的服务配置。
#[derive(Debug, vernal_macros::ConfigurationProperties)]
#[configuration(prefix = "service", rename_all = "kebab-case")]
struct ServiceProperties {
    port: u16,
    #[configuration(rename = "application-name")]
    name: String,
    token: Option<String>,
    #[configuration(default)]
    graceful_shutdown: bool,
    #[configuration(default = "default_worker_threads()")]
    worker_threads: usize,
    #[configuration(nested)]
    tls: TlsProperties,
}

/// 证明配置对象与普通业务依赖使用相同的 Resolver 和 Singleton 图。
struct ConfiguredService {
    properties: Arc<ServiceProperties>,
}

/// 创建显式依赖配置对象的业务组件定义。
fn configured_service_definition() -> ComponentDefinition {
    ComponentDefinition::try_singleton::<ConfiguredService, _>(
        |resolver| -> Result<ConfiguredService, Box<dyn Error + Send + Sync>> {
            Ok(ConfiguredService {
                properties: resolver.resolve::<ServiceProperties>()?,
            })
        },
    )
    .depends_on::<ServiceProperties>()
}

/// 创建测试使用的不可变属性来源。
fn property_source(
    values: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> Arc<MapPropertySource> {
    Arc::new(MapPropertySource::new("application", values).expect("valid property source"))
}

#[tokio::test]
async fn derived_configuration_is_a_context_local_native_component() {
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application
        .environment()
        .add_last(property_source([
            ("service.port", "8088"),
            ("service.application-name", "Vernal"),
            ("service.token", "${deployment.token:local-token}"),
            ("service.tls.enabled", "true"),
            ("service.tls.certificate", "/run/secrets/server.pem"),
        ]))
        .expect("property source should register");
    application
        .configuration_properties::<ServiceProperties>()
        .expect("configuration definition should register");
    application
        .register(configured_service_definition())
        .expect("dependent service should register");

    let context = application.build().expect("application should build");
    let service = context
        .container()
        .resolve::<ConfiguredService>()
        .expect("configured service should resolve");
    let same_properties = context
        .container()
        .resolve::<ServiceProperties>()
        .expect("configuration should resolve directly");

    assert!(Arc::ptr_eq(&service.properties, &same_properties));
    assert_eq!(service.properties.port, 8088);
    assert_eq!(service.properties.name, "Vernal");
    assert_eq!(service.properties.token.as_deref(), Some("local-token"));
    assert!(!service.properties.graceful_shutdown);
    assert_eq!(service.properties.worker_threads, 4);
    assert!(service.properties.tls.enabled);
    assert_eq!(
        service.properties.tls.certificate.as_deref(),
        Some("/run/secrets/server.pem")
    );
}

#[test]
fn binding_error_identifies_field_and_key_without_exposing_value() {
    let mut builder = ApplicationEnvironment::builder();
    builder
        .add_last(property_source([
            ("service.port", "postgres://admin:secret@database"),
            ("service.application-name", "Vernal"),
        ]))
        .expect("property source should register");
    let error = ServiceProperties::bind(&builder.build())
        .expect_err("invalid port must fail configuration binding");

    assert_eq!(error.field(), "port");
    assert_eq!(error.property_key(), "service.port");
    assert!(error.source().is_some());
    let display = error.to_string();
    let debug = format!("{error:?}");
    assert!(display.contains("service.port"));
    assert!(!display.contains("admin"));
    assert!(!display.contains("secret"));
    assert!(!debug.contains("postgres"));
}

#[tokio::test]
async fn refresh_warms_configuration_singletons_and_fails_before_ready() {
    let mut application =
        VernalApplicationBuilder::current().expect("Tokio runtime should be available");
    application
        .environment()
        .add_last(property_source([
            ("service.port", "postgres://admin:secret@database"),
            ("service.application-name", "Vernal"),
        ]))
        .expect("property source should register");
    application
        .configuration_properties::<ServiceProperties>()
        .expect("configuration definition should register");
    let context = application.build().expect("component graph should build");

    let error = context
        .refresh()
        .await
        .expect_err("refresh must eagerly bind singleton configuration");
    assert!(matches!(error, ContextError::ContainerWarmUp { .. }));
    assert_eq!(context.state().await, ContextState::Failed);
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("service.port"));
    assert!(!diagnostic.contains("admin"));
    assert!(!diagnostic.contains("secret"));
    assert!(!format!("{error:?}").contains("postgres://"));

    context
        .close()
        .await
        .expect("failed context should still close");
    assert_eq!(context.state().await, ContextState::Closed);
}
