//! Tests for all new files to recover coverage.
use std::sync::Arc;

// ── ApplicationContext system ─────────────────────────────────────

#[test]
fn static_context_register_and_get() {
    use vernal_beans::application_context::ApplicationContext;
    let ctx = vernal_beans::static_application_context::StaticApplicationContext::new("Test");
    ctx.register_bean("bean1", Arc::new(42i32));
    assert!(ctx.get_bean("bean1").is_ok());
}

#[test]
fn generic_context_refresh() {
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = vernal_beans::generic_application_context::GenericApplicationContext::new();
    ctx.refresh();
    assert!(ctx.is_active());
}

#[test]
fn annotation_context_register() {
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = vernal_beans::annotation_config_application_context::AnnotationConfigApplicationContext::new();
    ctx.register("com.example.Config");
    assert_eq!(ctx.config_classes().len(), 1);
    ctx.refresh();
    assert!(ctx.is_active());
}

// ── Event system ──────────────────────────────────────────────────

#[test]
fn generic_event_new() {
    use vernal_beans::application_event::ApplicationEvent;
    let e = vernal_beans::application_event::GenericApplicationEvent::new();
    assert!(e.timestamp > 0);
}

#[test]
#[test]
fn lifecycle_event_started() {
    use vernal_beans::application_event::ApplicationEvent;
    use vernal_beans::lifecycle_event::LifecycleEvent;
    let e = LifecycleEvent::Started;
    assert!(e.get_timestamp() > 0);
}

#[test]
fn context_refresh_event() {
    use vernal_beans::application_event::ApplicationEvent;
    let e = vernal_beans::context_refresh_event::ContextRefreshEvent;
    assert!(e.get_timestamp() > 0);
}

// ── MessageSource ─────────────────────────────────────────────────

#[test]
fn hierarchical_msg_source() {
    use vernal_beans::message_source::MessageSource;
    let s = vernal_beans::hierarchical_message_source::HierarchicalMessageSource::new();
    assert_eq!(s.get_message("c", &[], Some("d")), "d");
}

// ── Autowire system ───────────────────────────────────────────────

#[test]
fn autowire_utils_candidates() {
    use vernal_beans::autowire_utils::AutowireUtils;
    let tid = std::any::TypeId::of::<String>();
    assert!(AutowireUtils::is_autowire_candidate(tid, tid));
    assert_eq!(AutowireUtils::determine_autowire_candidates(tid, &[tid]).len(), 1);
}

#[test]
fn autowire_factory_create_err() {
    let f = vernal_beans::abstract_autowire_capable_bean_factory::AbstractAutowireCapableBeanFactory::new();
    assert!(f.create_bean("x", "y").is_err());
}

#[test]
fn constructor_resolver_new() {
    let r = vernal_beans::constructor_resolver::ConstructorResolver::new();
    assert!(r.resolve_constructor_arguments(&[]).is_empty());
}

#[test]
fn instantiation_strategy_err() {
    use vernal_beans::instantiation_strategy::InstantiationStrategy;
    let s = vernal_beans::simple_instantiation_strategy::SimpleInstantiationStrategy::new();
    assert!(s.instantiate("x", &[]).is_err());
}

// ── XML system ────────────────────────────────────────────────────

#[test]
fn xml_reader_context_new() {
    let ctx = vernal_beans::xml_reader_context::XmlReaderContext::new("test.xml");
    assert_eq!(ctx.resource_description(), "test.xml");
}

#[test]
fn xml_reader_helpers_parse() {
    let attrs = vernal_beans::xml_reader_helpers::parse_attributes("a=\"1\" b=\"2\"");
    assert_eq!(attrs.len(), 2);
}

#[test]

// ── Metadata system ───────────────────────────────────────────────

#[test]
fn annotation_metadata_new() {
    let m = vernal_beans::annotation_metadata::AnnotationMetadata::new("Test");
    assert_eq!(m.type_name(), "Test");
}

#[test]
fn config_class_new() {
    let c = vernal_beans::configuration_class::ConfigurationClass::new("Config");
    assert_eq!(c.class_name(), "Config");
}

#[test]
fn config_class_parser_new() {
    let p = vernal_beans::configuration_class_parser::ConfigurationClassParser::new();
    let c = p.parse("Config");
    assert_eq!(c.class_name(), "Config");
}

// ── Transaction ───────────────────────────────────────────────────

#[test]
fn transaction_template_new() {
    let t = vernal_beans::transaction_template::TransactionTemplate::new();
    assert!(t.execute(|| Ok(42)).is_ok());
}

// ── Cache system ──────────────────────────────────────────────────

#[test]
fn simple_cache_basic() {
    use vernal_beans::cache::{Cache, SimpleCache};
    let c = SimpleCache::new();
    c.put("k", Arc::new(42i32));
    assert!(c.get("k").is_some());
}

#[test]
fn cache_manager_basic() {
    use vernal_beans::cache_manager::CacheManager;
    let m = CacheManager::new();
    m.create_cache("c");
    assert!(m.get_cache("c").is_some());
}

// ── Validation ────────────────────────────────────────────────────

#[test]
fn validation_result_basic() {
    use vernal_beans::validation::ValidationResult;
    let mut r = ValidationResult::new();
    assert!(r.is_valid());
    r.add_error("err");
    assert!(!r.is_valid());
}

// ── HTTP/Web ──────────────────────────────────────────────────────

#[test]
fn http_method_eq() {
    use vernal_beans::http_method::HttpMethod;
    assert_eq!(HttpMethod::Get, HttpMethod::Get);
    assert_ne!(HttpMethod::Get, HttpMethod::Post);
}

#[test]
fn http_status_ok() {
    use vernal_beans::http_status::HttpStatus;
    assert!(HttpStatus::ok().is_success());
    assert!(!HttpStatus::not_found().is_success());
}

#[test]
fn http_headers_set_get() {
    use vernal_beans::http_headers::HttpHeaders;
    let mut h = HttpHeaders::new();
    h.set("Content-Type", "text/plain");
    assert_eq!(h.get_first("Content-Type"), Some("text/plain"));
}

#[test]
fn media_type_eq() {
    use vernal_beans::media_type::MediaType;
    let json = MediaType::application_json();
    assert_eq!(json.to_string(), "application/json");
}

// ── Attribute accessor ────────────────────────────────────────────

#[test]
fn attribute_accessor_set_get() {
    let mut a = vernal_beans::attribute_accessor_support::AttributeAccessorSupport::new();
    a.set_attribute("k", Box::new(42i32));
    assert!(a.has_attribute("k"));
    assert_eq!(a.len(), 1);
    a.remove_attribute("k");
    assert!(a.is_empty());
}

// ── Passport ──────────────────────────────────────────────────────

#[test]
fn passport_new() {
    let p = vernal_beans::passport::Passport::new("user", "pass");
    assert_eq!(p.principal(), "user");
    assert_eq!(p.credentials(), "pass");
}

// ── DisposableBeanAdapter ────────────────────────────────────────

#[test]
fn disposable_adapter_new() {
    let a = vernal_beans::destructible_bean_adapter::DisposableBeanAdapter::new("bean");
    assert_eq!(a.bean_name(), "bean");
    assert!(a.destroy().is_ok());
}

// ── Scoped context events ─────────────────────────────────────────

#[test]
fn context_events_new() {
    use vernal_beans::application_event::ApplicationEvent;
    assert!(vernal_beans::context_started_event::ContextStartedEvent.get_timestamp() > 0);
    assert!(vernal_beans::context_stopped_event::ContextStoppedEvent.get_timestamp() > 0);
    assert!(vernal_beans::context_closed_event::ContextClosedEvent.get_timestamp() > 0);
}

// ── Scheduled executor ────────────────────────────────────────────

#[test]
fn scheduled_executor_new() {
    let _s = vernal_beans::scheduled_executor::ScheduledExecutor::new();
}

// ── Condition system ──────────────────────────────────────────────

#[test]
fn condition_context_new() {
    let _c = vernal_beans::condition::ConditionContext::new();
}

// ── Resource system ───────────────────────────────────────────────

#[test]
fn resource_abstract_basic() {
    use vernal_beans::resource::Resource;
    let r = vernal_beans::abstract_resource::AbstractResource::new("test.txt");
    assert!(r.exists());
    assert_eq!(r.description(), "test.txt");
}

#[test]
fn filesystem_resource_basic() {
    use vernal_beans::resource::Resource;
    let r = vernal_beans::filesystem_resource::FileSystemResource::new("/tmp/test.txt");
    assert_eq!(r.filename(), Some("test.txt"));
    assert!(!r.exists());
}

#[test]
fn classpath_resource_basic() {
    use vernal_beans::resource::Resource;
    let r = vernal_beans::classpath_resource::ClassPathResource::new("com/test.xml");
    assert_eq!(r.get_path(), "com/test.xml");
}

#[test]
fn url_resource_basic() {
    use vernal_beans::resource::Resource;
    let r = vernal_beans::url_resource::UrlResource::new("https://example.com/f.txt");
    assert_eq!(r.scheme(), Some("https"));
    assert_eq!(r.filename(), Some("f.txt"));
}

// ── Resource loader ───────────────────────────────────────────────

#[test]
fn default_resource_loader_basic() {
    use vernal_beans::default_resource_loader::{DefaultResourceLoader, ResourceLoader};
    let loader = DefaultResourceLoader::new();
    let r = loader.get_resource("classpath:test.xml").unwrap();
    assert!(r.description().contains("test.xml"));
}

// ── Placeholder resolver ─────────────────────────────────────────

#[test]
fn placeholder_resolver_basic() {
    use vernal_beans::placeholder_resolver::PlaceholderResolver;
    use vernal_beans::property_source::PropertySource;
    let mut ps = PropertySource::new("p");
    ps.set("name", "world");
    let mut r = PlaceholderResolver::new();
    r.add_source(ps);
    assert_eq!(r.resolve("${name}").unwrap(), "world");
}

// ── Profile system ────────────────────────────────────────────────

#[test]
fn profile_new() {
    use vernal_beans::profile::Profile;
    let p = Profile::new("dev");
    assert_eq!(p.name(), "dev");
    assert!(!p.is_default());
}

#[test]
fn profiles_add_remove() {
    use vernal_beans::profiles::Profiles;
    use vernal_beans::profile::Profile;
    let mut ps = Profiles::new();
    ps.add(Profile::new("dev"));
    assert!(ps.is_active("dev"));
    ps.remove("dev");
    assert!(!ps.is_active("dev"));
}

// ── String utils ──────────────────────────────────────────────────

#[test]
fn string_utils_has_text() {
    use vernal_beans::string_utils::StringUtils;
    assert!(StringUtils::has_text("hello"));
    assert!(!StringUtils::has_text("  "));
}

#[test]
fn ant_path_matches() {
    use vernal_beans::ant_path_matcher::AntPathMatcher;
    let m = AntPathMatcher::new();
    assert!(m.matches("*", "anything"));
    assert!(!m.matches("a", "b"));
}

#[test]
fn class_utils_short_name() {
    use vernal_beans::class_utils::ClassUtils;
    assert_eq!(ClassUtils::get_short_name("com::example::Test"), "Test");
}

// ── SerializableTypeWrapper ───────────────────────────────────────

#[test]
fn serializable_wrapper_new() {
    let w = vernal_beans::serializable_type_wrapper::SerializableTypeWrapper::new("S");
    assert_eq!(w.type_name(), "S");
}
