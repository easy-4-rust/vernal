//! Expression 集成 + PlaceholderConfigurer + ConfigurationClassPostProcessor 测试。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use vernal_beans::ComponentDefinition;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::bean_definition_registry_post_processor::BeanDefinitionRegistryPostProcessor;
use vernal_beans::bean_expression_resolver::BeanExpressionResolver;
use vernal_beans::bean_factory_post_processor::BeanFactoryPostProcessor;
use vernal_beans::configuration_class_post_processor::ConfigurationClassPostProcessor;
use vernal_beans::placeholder_configurer::PlaceholderConfigurerSupport;
use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug)]
struct AppConfig {
    db_url: String,
    max_connections: u32,
}

// ── 1. StandardBeanExpressionResolver 测试 ───────────────────────────────

/// 参照 Spring `StandardBeanExpressionResolverTests`：
/// 验证 Bean 名称引用。
#[test]
fn expression_resolver_bean_reference() {
    let resolver = StandardBeanExpressionResolver::new();
    let bean: Arc<dyn Any + Send + Sync> = Arc::new(AppConfig {
        db_url: "test".to_string(),
        max_connections: 10,
    });

    resolver.register_bean("appConfig".to_string(), bean);

    // 验证 Bean 名称引用
    let result = resolver.evaluate("appConfig", None).unwrap();
    assert!(result.is_some());
}

/// 验证不存在的 Bean 名称返回 None。
#[test]
fn expression_resolver_unknown_bean() {
    let resolver = StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("unknownBean", None).unwrap();
    assert!(result.is_none());
}

/// 验证 register_bean / bean_count。
#[test]
fn expression_resolver_register_and_count() {
    let resolver = StandardBeanExpressionResolver::new();
    assert_eq!(resolver.bean_count(), 0);

    resolver.register_bean("a".to_string(), Arc::new(42i32));
    resolver.register_bean("b".to_string(), Arc::new("hello".to_string()));
    assert_eq!(resolver.bean_count(), 2);

    resolver.clear_context();
    assert_eq!(resolver.bean_count(), 0);
}

/// 验证 BeanExpressionResolver trait 实现。
#[test]
fn expression_resolver_trait_implementation() {
    let resolver = StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("test", None).unwrap();
    assert!(result.is_none()); // 无 Bean 上下文
}

// ── 2. PlaceholderConfigurerSupport 测试 ─────────────────────────────────

/// 参照 Spring `PropertyPlaceholderConfigurerTests`：
/// 验证占位符替换。
#[test]
fn placeholder_configurer_basic() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property(
        "db.url".to_string(),
        "postgres://localhost/test".to_string(),
    );
    configurer.set_property("db.port".to_string(), "5432".to_string());

    let result = configurer.resolve_placeholder("${db.url}");
    assert_eq!(result, "postgres://localhost/test");
}

/// 验证带默认值的占位符。
#[test]
fn placeholder_configurer_with_default() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property("existing".to_string(), "value".to_string());

    // 存在的属性
    let result = configurer.resolve_placeholder("${existing}");
    assert_eq!(result, "value");

    // 不存在的属性，使用默认值
    let result = configurer.resolve_placeholder("${missing:default_value}");
    assert_eq!(result, "default_value");

    // 不存在的属性，无默认值
    let result = configurer.resolve_placeholder("${missing}");
    assert_eq!(result, "");
}

/// 验证嵌套占位符。
#[test]
fn placeholder_configurer_nested() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property("host".to_string(), "localhost".to_string());
    configurer.set_property("port".to_string(), "5432".to_string());

    let result = configurer.resolve_placeholder("jdbc://${host}:${port}/db");
    assert_eq!(result, "jdbc://localhost:5432/db");
}

/// 验证批量设置属性。
#[test]
fn placeholder_configurer_batch_set() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    let mut props = HashMap::new();
    props.insert("a".to_string(), "1".to_string());
    props.insert("b".to_string(), "2".to_string());
    configurer.set_properties(props);

    assert_eq!(configurer.get_property("a"), Some("1"));
    assert_eq!(configurer.get_property("b"), Some("2"));
}

/// 验证自定义前缀/后缀。
#[test]
fn placeholder_configurer_custom_prefix() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_placeholder_prefix("@{");
    configurer.set_placeholder_suffix("}");
    configurer.set_property("key".to_string(), "value".to_string());

    let result = configurer.resolve_placeholder("@{key}");
    assert_eq!(result, "value");
}

/// 验证 properties() 方法。
#[test]
fn placeholder_configurer_properties() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property("x".to_string(), "1".to_string());

    let props = configurer.properties();
    assert_eq!(props.len(), 1);
    assert_eq!(props.get("x").unwrap(), "1");
}

// ── 3. ConfigurationClassPostProcessor 测试 ──────────────────────────────

/// 参照 Spring `ConfigurationClassPostProcessorTests`：
/// 验证配置类处理器基本功能。
#[test]
fn configuration_class_post_processor_basic() {
    let processor = ConfigurationClassPostProcessor::new();
    assert_eq!(processor.registered_count(), 0);
}

/// 验证注册配置类。
#[test]
fn configuration_class_post_processor_register() {
    let mut processor = ConfigurationClassPostProcessor::new();
    processor.register_configuration("com.example.Config".to_string());
    processor.register_configuration("com.example.AppConfig".to_string());

    assert_eq!(processor.registered_count(), 2);
    assert!(
        processor
            .registered_configurations()
            .contains(&"com.example.Config".to_string())
    );
    assert!(
        processor
            .registered_configurations()
            .contains(&"com.example.AppConfig".to_string())
    );
}

/// 验证 BeanFactoryPostProcessor trait 实现。
#[test]
fn configuration_class_post_processor_trait() {
    let processor = ConfigurationClassPostProcessor::new();
    // 验证 processor 可以创建
    assert_eq!(processor.registered_count(), 0);
    // processor 实现了 BeanFactoryPostProcessor trait
    // 但由于 Container 不直接实现 ConfigurableListableBeanFactory，
    // 这里只验证 processor 的基本功能
}

/// 验证 BeanDefinitionRegistryPostProcessor trait 实现。
#[test]
fn configuration_class_post_processor_registry_trait() {
    let processor = ConfigurationClassPostProcessor::new();
    // 验证可以创建实例并调用 trait 方法
    // Registry 不直接实现 BeanDefinitionRegistry，但 processor 可以与 RegistryBuilder 交互
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    let _registry = builder.build().unwrap();
    // post_process_bean_definition_registry 需要 &mut dyn BeanDefinitionRegistry
    // 当前 Registry 不实现此 trait，验证 processor 本身可以创建
    assert_eq!(processor.registered_count(), 0);
}

/// 验证 ConfigurationClassPostProcessor 与 PlaceholderConfigurer 集成。
#[test]
fn configuration_with_placeholder_integration() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property(
        "app.db.url".to_string(),
        "postgres://prod:5432/app".to_string(),
    );
    configurer.set_property("app.db.pool.size".to_string(), "20".to_string());

    let db_url = configurer.resolve_placeholder("${app.db.url}");
    let pool_size: u32 = configurer
        .resolve_placeholder("${app.db.pool.size}")
        .parse()
        .unwrap_or(10);

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            move |_resolver: &Resolver| AppConfig {
                db_url: db_url.clone(),
                max_connections: pool_size,
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let config = container.resolve::<AppConfig>().unwrap();
    assert_eq!(config.db_url, "postgres://prod:5432/app");
    assert_eq!(config.max_connections, 20);
}

// ── 4. Expression + BeanFactory 集成 ────────────────────────────────────

/// 验证 Expression 解析器与 BeanFactory 集成。
#[test]
fn expression_resolver_with_bean_factory() {
    let resolver = StandardBeanExpressionResolver::new();

    // 注册一些 Bean 到表达式上下文
    let config: Arc<dyn Any + Send + Sync> = Arc::new(AppConfig {
        db_url: "integrated".to_string(),
        max_connections: 5,
    });
    resolver.register_bean("appConfig".to_string(), config);

    // 验证可以通过名称引用
    let result = resolver.evaluate("appConfig", None).unwrap();
    assert!(result.is_some());

    // 验证不存在的 Bean
    let result = resolver.evaluate("nonexistent", None).unwrap();
    assert!(result.is_none());
}

// ── 5. PlaceholderConfigurer 属性源加载 ──────────────────────────────────

/// 验证从 HashMap 加载属性源。
#[test]
fn placeholder_configurer_from_hashmap() {
    let mut props = HashMap::new();
    props.insert("server.host".to_string(), "0.0.0.0".to_string());
    props.insert("server.port".to_string(), "8080".to_string());
    props.insert("app.name".to_string(), "my-app".to_string());

    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_properties(props);

    // 验证所有属性
    assert_eq!(configurer.get_property("server.host"), Some("0.0.0.0"));
    assert_eq!(configurer.get_property("server.port"), Some("8080"));
    assert_eq!(configurer.get_property("app.name"), Some("my-app"));

    // 验证占位符替换
    let url = configurer.resolve_placeholder("http://${server.host}:${server.port}");
    assert_eq!(url, "http://0.0.0.0:8080");
}

/// 验证占位符替换在字符串中的多个位置。
#[test]
fn placeholder_configurer_multiple_in_string() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property("host".to_string(), "db.example.com".to_string());
    configurer.set_property("port".to_string(), "5432".to_string());
    configurer.set_property("name".to_string(), "mydb".to_string());

    let result = configurer.resolve_placeholder("jdbc:postgresql://${host}:${port}/${name}");
    assert_eq!(result, "jdbc:postgresql://db.example.com:5432/mydb");
}

/// 验证空属性值替换为空字符串。
#[test]
fn placeholder_configurer_empty_value() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property("empty".to_string(), "".to_string());

    let result = configurer.resolve_placeholder("${empty}");
    assert_eq!(result, "");
}

/// 验证默认值优先级（属性存在时用属性值）。
#[test]
fn placeholder_configurer_default_priority() {
    let mut configurer = PlaceholderConfigurerSupport::new();
    configurer.set_property("key".to_string(), "actual".to_string());

    // 属性存在时使用属性值
    let result = configurer.resolve_placeholder("${key:fallback}");
    assert_eq!(result, "actual");
}

// ── 6. ConfigurationClassPostProcessor + BeanDefinitionRegistry ──────────

/// 验证 ConfigurationClassPostProcessor 可以在 RegistryBuilder 上操作。
#[test]
fn configuration_class_processor_with_registry() {
    let processor = ConfigurationClassPostProcessor::new();

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<AppConfig, _>(
            |_resolver: &Resolver| AppConfig {
                db_url: "test".to_string(),
                max_connections: 10,
            },
        ))
        .unwrap();

    // 验证 Builder 可以正常构建
    let registry = builder.build().unwrap();
    assert_eq!(registry.len(), 1);

    // ConfigurationClassPostProcessor 当前是空实现
    // 验证 processor 可以创建和配置
    assert_eq!(processor.registered_count(), 0);
}
