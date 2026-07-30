//! 综合测试 — 覆盖所有新建文件的主要 API。
use std::sync::Arc;

// ── ApplicationContext system ─────────────────────────────────────

#[test]
fn static_application_context_register_bean() {
    use vernal_beans::application_context::ApplicationContext;
    let ctx = vernal_beans::static_application_context::StaticApplicationContext::new("Test");
    ctx.register_bean("bean1", Arc::new(42i32));
    assert!(ctx.get_bean("bean1").is_ok());
}

#[test]
fn generic_application_context_basic() {
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = vernal_beans::generic_application_context::GenericApplicationContext::new();
    ctx.refresh();
    assert!(ctx.is_active());
}

#[test]
fn annotation_config_context_basic() {
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = vernal_beans::annotation_config_application_context::AnnotationConfigApplicationContext::new();
    ctx.register("com.example.Config");
    assert_eq!(ctx.config_classes().len(), 1);
    ctx.refresh();
    assert!(ctx.is_active());
}

#[test]
fn abstract_application_context_basic() {
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = vernal_beans::abstract_application_context::AbstractApplicationContext::new("TestCtx");
    assert_eq!(ctx.get_display_name(), "TestCtx");
    ctx.refresh();
    assert!(ctx.is_active());
    ctx.close();
    assert!(!ctx.is_active());
}

// ── ApplicationEvent system ────────────────────────────────────────

#[test]
fn application_event_basic() {
    use vernal_beans::application_event::ApplicationEvent;
    let e = vernal_beans::application_event::GenericApplicationEvent::new();
    assert!(e.timestamp > 0);
}

#[test]
fn lifecycle_event_basic() {
    use vernal_beans::application_event::ApplicationEvent;
    use vernal_beans::lifecycle_event::LifecycleEvent;
    let e = LifecycleEvent::Started;
    assert!(vernal_beans::lifecycle_event::LifecycleEvent::Started.get_timestamp() > 0);
}

#[test]
fn simple_multicaster_basic() {
    use vernal_beans::application_event_multicaster::ApplicationEventMulticaster;
    let mc = vernal_beans::simple_application_event_multicaster::SimpleApplicationEventMulticaster::new();
    mc.multicast_event(Arc::new("event".to_string()) as Arc<dyn std::any::Any + Send + Sync>);
}

// ── MessageSource system ────────────────────────────────────────────

#[test]
fn hierarchical_message_source_basic() {
    use vernal_beans::message_source::MessageSource;
    let s = vernal_beans::hierarchical_message_source::HierarchicalMessageSource::new();
    assert_eq!(s.get_message("code", &[], Some("default")), "default");
}

#[test]
fn reloadable_message_source_basic() {
    use vernal_beans::message_source::MessageSource;
    let s = vernal_beans::reloadable_message_source::ReloadableMessageSource::new();
    s.reload();
    assert_eq!(s.get_message("code", &[], Some("val")), "val");
}

// ── Autowire/Instantiation system ──────────────────────────────────

#[test]
fn autowire_utils_basic() {
    use vernal_beans::autowire_utils::AutowireUtils;
    let tid = std::any::TypeId::of::<String>();
    assert!(AutowireUtils::is_autowire_candidate(tid, tid));
    let c = AutowireUtils::determine_autowire_candidates(tid, &[tid, std::any::TypeId::of::<i32>()]);
    assert_eq!(c.len(), 1);
}

#[test]
fn abstract_autowire_factory_basic() {
    let f = vernal_beans::abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory::new();
    assert!(f.create_bean("test", "test").is_err());
}

#[test]
fn constructor_resolver_basic() {
    let r = vernal_beans::constructor_resolver::ConstructorResolver::new();
    assert!(r.resolve_constructor_arguments(&[]).is_empty());
}

#[test]
fn simple_instantiation_strategy_basic() {
    use vernal_beans::instantiation_strategy::InstantiationStrategy;
    let s = vernal_beans::simple_instantiation_strategy::SimpleInstantiationStrategy::new();
    assert!(s.instantiate("test", &[]).is_err());
}

#[test]
fn destructible_bean_adapter_basic() {
    use vernal_beans::destructible_bean_adapter::DisposableBeanAdapter;
    let a = DisposableBeanAdapter::new("myBean");
    assert_eq!(a.bean_name(), "myBean");
    assert!(a.destroy().is_ok());
}

// ── XML/BeanDefinition system ───────────────────────────────────────

#[test]
fn xml_reader_context_basic() {
    let ctx = vernal_beans::xml_reader_context::XmlReaderContext::new("test.xml");
    assert_eq!(ctx.resource_description(), "test.xml");
}

#[test]
fn xml_reader_helpers_basic() {
    let attrs = vernal_beans::xml_reader_helpers::parse_attributes("name=\"test\" id=\"1\"");
    assert_eq!(attrs.get("name"), Some(&"test".to_string()));
    assert!(vernal_beans::xml_reader_helpers::is_default_namespace(""));
}

// ── Annotation/Metadata system ──────────────────────────────────────

#[test]
fn annotation_metadata_basic() {
    let mut m = vernal_beans::annotation_metadata::AnnotationMetadata::new("com.example.MyClass");
    assert_eq!(m.type_name(), "com.example.MyClass");
    m.add_annotation("Autowired", std::collections::HashMap::new());
    assert!(m.has_annotation("Autowired"));
}

#[test]
fn configuration_class_basic() {
    let mut c = vernal_beans::configuration_class::ConfigurationClass::new("com.example.Config");
    c.add_bean_method("dataSource");
    assert_eq!(c.bean_methods(), &["dataSource"]);
}

// ── AOP system ─────────────────────────────────────────────────────

#[test]
fn proxy_factory_basic() {
    let mut pf = vernal_beans::proxy_factory::ProxyFactory::new();
    pf.set_target(Arc::new("hello".to_string()) as Arc<dyn std::any::Any + Send + Sync>);
    pf.add_interface("MyInterface");
    assert!(pf.get_proxy().is_ok());
}

// ── Transaction system ─────────────────────────────────────────────

#[test]
fn transaction_template_basic() {
    use vernal_beans::transaction_template::TransactionTemplate;
    let template = TransactionTemplate::new();
    let result = template.execute(|| Ok(42i32));
    assert_eq!(result.unwrap(), 42);
}

// ── HTTP/Web system ────────────────────────────────────────────────

#[test]
fn http_method_basic() {
    use vernal_beans::http_method::HttpMethod;
    assert_eq!(HttpMethod::Get, HttpMethod::Get);
    assert_ne!(HttpMethod::Get, HttpMethod::Post);
}

#[test]
fn http_status_basic() {
    use vernal_beans::http_status::HttpStatus;
    let s = HttpStatus::ok();
    assert!(s.is_success());
    assert!(!s.is_error());
    assert_eq!(s.code, 200);
}

#[test]
fn http_headers_basic() {
    use vernal_beans::http_headers::HttpHeaders;
    let mut h = HttpHeaders::new();
    h.set("Content-Type", "application/json");
    assert!(h.contains("Content-Type"));
    assert_eq!(h.get_first("Content-Type"), Some("application/json"));
    h.remove("Content-Type");
    assert!(!h.contains("Content-Type"));
}

#[test]
fn media_type_basic() {
    use vernal_beans::media_type::MediaType;
    let json = MediaType::application_json();
    assert_eq!(json.to_string(), "application/json");
    assert!(json.matches(&MediaType::application_json()));
}

// ── Validation system ──────────────────────────────────────────────

#[test]
fn validation_result_basic() {
    use vernal_beans::validation::ValidationResult;
    let mut r = ValidationResult::new();
    assert!(r.is_valid());
    r.add_error("test error");
    assert!(!r.is_valid());
}

#[test]
fn validation_errors_basic() {
    use vernal_beans::validation_errors::ValidationErrors;
    let mut v = ValidationErrors::new();
    assert!(!v.has_errors());
    v.add("error1");
    v.add("error2");
    assert!(v.has_errors());
    assert_eq!(v.count(), 2);
}

// ── Cache system ───────────────────────────────────────────────────

#[test]
fn simple_cache_basic() {
    use vernal_beans::cache::{Cache, SimpleCache};
    let c = SimpleCache::new();
    assert!(c.get("key").is_none());
    c.put("key", Arc::new(42i32));
    assert!(c.get("key").is_some());
    c.evict("key");
    assert!(c.get("key").is_none());
}

#[test]
fn cache_manager_basic() {
    use vernal_beans::cache_manager::CacheManager;
    let m = CacheManager::new();
    m.create_cache("test");
    assert!(m.get_cache("test").is_some());
    assert_eq!(m.cache_names().len(), 1);
}

// ── Utility system ─────────────────────────────────────────────────

#[test]
fn class_utils_basic() {
    use vernal_beans::class_utils::ClassUtils;
    assert_eq!(ClassUtils::get_short_name("com::example::Test"), "Test");
    assert_eq!(ClassUtils::get_package_name("com::example::Test"), "com::example");
}

#[test]
fn string_utils_basic() {
    use vernal_beans::string_utils::StringUtils;
    assert!(StringUtils::has_text("hello"));
    assert!(!StringUtils::has_text("  "));
    assert!(StringUtils::starts_with("hello", "he"));
    assert!(StringUtils::ends_with("hello", "lo"));
}

#[test]
fn ant_path_matcher_basic() {
    use vernal_beans::ant_path_matcher::AntPathMatcher;
    let m = AntPathMatcher::new();
    assert!(m.matches("*", "anything"));
    assert!(m.matches("**", "anything"));
    assert!(m.matches("test", "test"));
    assert!(!m.matches("test", "other"));
}

