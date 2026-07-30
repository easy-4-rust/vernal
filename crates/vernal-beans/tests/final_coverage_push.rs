//! Final coverage push — targets the remaining uncovered paths.
use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    bean_definition::BeanDefinition, bean_factory::BeanFactory,
    bean_definition_registry::BeanDefinitionRegistry,
    autowire_capable_bean_factory::AutowireCapableBeanFactory,
    ComponentDefinition, ComponentKey, Container, RegistryBuilder,
    ResolveError, Dependency, TraitKey, Qualifier,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    Container::new(b.build().unwrap())
}

// ── BeanFactory trait (all 9 methods) ──────────────────────────────

#[test]
fn bf_get_bean_by_key() {
    let c = make_container();
    let bean = c.get_bean_by_key(&ComponentKey::of::<String>()).unwrap();
    assert_eq!((*bean).downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn bf_get_bean_by_type_id() {
    let c = make_container();
    let bean = c.get_bean_by_type_id(std::any::TypeId::of::<String>()).unwrap();
    assert_eq!((*bean).downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn bf_contains_bean() {
    let c = make_container();
    assert!(c.contains_bean(&ComponentKey::of::<String>()));
    assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
}

#[test]
fn bf_is_singleton() {
    let c = make_container();
    assert!(c.is_singleton(&ComponentKey::of::<String>()).unwrap());
}

#[test]
fn bf_is_prototype() {
    let c = make_container();
    assert!(!c.is_prototype(&ComponentKey::of::<String>()).unwrap());
}

#[test]
fn bf_get_type_ok() {
    let c = make_container();
    assert_eq!(c.get_type(&ComponentKey::of::<String>()).unwrap(), Some("alloc::string::String"));
    assert!(c.get_type(&ComponentKey::of::<f64>()).is_err());
}

#[test]
fn bf_get_aliases_empty() {
    let c = make_container();
    assert!(c.get_aliases(&ComponentKey::of::<String>()).is_empty());
}

#[test]
fn bf_is_type_match() {
    let c = make_container();
    assert!(c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<String>()));
    assert!(!c.is_type_match(&ComponentKey::of::<String>(), std::any::TypeId::of::<i32>()));
}

#[test]
fn bf_get_bean_provider() {
    let c = make_container();
    let p = c.get_bean_provider_by_type_id(std::any::TypeId::of::<String>()).unwrap();
    let _ = p.get();
}

// ── AutowireCapableBeanFactory ────────────────────────────────────

#[test]
fn acbf_create_bean_err() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::create_bean(&c, "unknown").is_err());
}

#[test]
fn acbf_autowire_bean() {
    let c = make_container();
    let bean = Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::autowire_bean(&c, bean).is_ok());
}

#[test]
fn acbf_initialize_bean() {
    let c = make_container();
    let bean = Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::initialize_bean(&c, bean, "test").is_ok());
}

#[test]
fn acbf_configure_bean() {
    let c = make_container();
    let bean = Arc::new("x".to_string()) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::configure_bean(&c, bean, "test").is_ok());
}

#[test]
fn acbf_destroy_instance() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::destroy_bean_instance(&c, "test", &"x").is_ok());
}

#[test]
fn acbf_autowire_modes() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 0, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 1, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 2, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 3, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire(&c, "alloc::string::String", 99, false).is_err());
}

#[test]
fn acbf_autowire_bean_properties() {
    let c = make_container();
    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::autowire_bean_properties(&c, bean.clone(), 0, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire_bean_properties(&c, bean.clone(), 1, false).is_ok());
    assert!(AutowireCapableBeanFactory::autowire_bean_properties(&c, bean, 2, false).is_ok());
}

#[test]
fn acbf_apply_bean_property_values() {
    let c = make_container();
    let bean = Arc::new(42i32) as Arc<dyn Any + Send + Sync>;
    assert!(AutowireCapableBeanFactory::apply_bean_property_values(&c, bean, "test").is_ok());
}

#[test]
fn acbf_resolve_dependency_ok() {
    let c = make_container();
    let desc = vernal_beans::DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "String");
    assert!(AutowireCapableBeanFactory::resolve_dependency(&c, &desc, None).is_ok());
}

#[test]
fn acbf_resolve_dependency_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    let desc = vernal_beans::DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "String");
    assert!(AutowireCapableBeanFactory::resolve_dependency(&c, &desc, None).is_err());
}

#[test]
fn acbf_resolve_dependency_multiple() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
    b.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
        .qualified(Qualifier::new("q2").unwrap()));
    let c = Container::new(b.build().unwrap());
    let desc = vernal_beans::DependencyDescriptor::for_field(std::any::TypeId::of::<String>(), "String");
    assert!(AutowireCapableBeanFactory::resolve_dependency(&c, &desc, None).is_err());
}

#[test]
fn acbf_resolve_named_bean_ok() {
    let c = make_container();
    assert!(AutowireCapableBeanFactory::resolve_named_bean(&c, std::any::TypeId::of::<String>()).is_ok());
}

#[test]
fn acbf_resolve_named_bean_not_found() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert!(AutowireCapableBeanFactory::resolve_named_bean(&c, std::any::TypeId::of::<String>()).is_err());
}

#[test]
fn acbf_type_converter() {
    let mut c = make_container();
    assert!(AutowireCapableBeanFactory::type_converter(&c).is_none());
    c.set_type_converter(None);
}

// ── BeanDefinitionRegistry ─────────────────────────────────────────

#[test]
fn bdr_register_and_remove() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let mut c = Container::new(RegistryBuilder::new().build().unwrap());
    let def = Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>;
    BeanDefinitionRegistry::register_bean_definition(&mut c, "dyn".to_string(), def).unwrap();
    assert!(BeanDefinitionRegistry::contains_bean_definition(&c, "dyn"));
    assert!(BeanDefinitionRegistry::remove_bean_definition(&mut c, "dyn").is_ok());
    assert!(!BeanDefinitionRegistry::contains_bean_definition(&c, "dyn"));
}

#[test]
fn bdr_get_bean_definition() {
    let c = make_container();
    let def = BeanDefinitionRegistry::get_bean_definition(&c, "alloc::string::String");
    assert!(def.is_some());
}

#[test]
fn bdr_count_and_names() {
    let c = make_container();
    assert_eq!(BeanDefinitionRegistry::bean_definition_count(&c), 2);
    assert!(BeanDefinitionRegistry::bean_definition_names(&c).len() >= 2);
}

// ── Container public methods ──────────────────────────────────────

#[test]
fn container_new() {
    let c = Container::new(RegistryBuilder::new().build().unwrap());
    assert_eq!(c.bean_post_processor_count(), 0);
}

#[test]
fn container_add_post_processor() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;
    let mut c = make_container();
    struct PP;
    impl BeanPostProcessor for PP {}
    c.add_bean_post_processor(Arc::new(PP));
    assert_eq!(c.bean_post_processor_count(), 1);
}

#[test]
fn container_registry() {
    let c = make_container();
    let _ = c.registry();
}

#[test]
fn container_open_scope() {
    let c = make_container();
    let _scope = c.open_scope::<String>();
}

#[test]
fn container_transient_tracker() {
    let c = make_container();
    let _ = c.transient_tracker();
}

#[test]
fn container_warm_up() {
    let c = make_container();
    assert!(c.warm_up().is_ok());
}

#[test]
fn container_unused_definitions() {
    let c = make_container();
    let _ = c.unused_definitions();
}

#[test]
fn container_resolve() {
    let c = make_container();
    let val: Result<Arc<String>, _> = c.resolve();
    assert!(val.is_ok());
    assert_eq!(*val.unwrap(), "hello");
}

#[test]
fn container_resolve_qualified() {
    let mut b = RegistryBuilder::new();
    let q = Qualifier::new("primary").unwrap();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "a".to_string()));
    b.register(ComponentDefinition::singleton::<String, _>(|_| "b".to_string()).qualified(q.clone()));
    let c = Container::new(b.build().unwrap());
    let val: Result<Arc<String>, _> = c.resolve_qualified(&q);
    assert!(val.is_ok());
    assert_eq!(*val.unwrap(), "b");
}

#[test]
fn container_resolve_in_scope() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let val: Result<Arc<String>, _> = c.resolve_in(&scope);
    assert!(val.is_ok());
}

#[test]
fn container_resolve_qualified_in_scope() {
    let c = make_container();
    let scope = c.open_scope::<String>();
    let q = Qualifier::new("primary").unwrap();
    let val: Result<Arc<String>, _> = c.resolve_qualified_in(&q, &scope);
    assert!(val.is_err());
}

// ── ResolveError formatting ───────────────────────────────────────

#[test]
fn resolve_error_not_found() {
    let e = ResolveError::NotFound { component: "t".into(), path: vec!["r".into()] };
    let s = format!("{}", e);
    assert!(s.contains("t"));
}

#[test]
fn resolve_error_ambiguous() {
    let e = ResolveError::Ambiguous { component: "t".into(), candidates: vec!["a".into()], path: vec!["r".into()] };
    let s = format!("{}", e);
    assert!(s.contains("t"));
}

#[test]
fn resolve_error_undeclared() {
    let e = ResolveError::UndeclaredDependency { component: ComponentKey::of::<String>(), dependency: "d".into() };
    assert!(!format!("{}", e).is_empty());
}

#[test]
fn resolve_error_type_mismatch() {
    let e = ResolveError::TypeMismatch { component: ComponentKey::of::<String>() };
    assert!(!format!("{}", e).is_empty());
}

#[test]
fn resolve_error_trait_binding_mismatch() {
    let e = ResolveError::TraitBindingTypeMismatch { binding: TraitKey::of::<dyn std::fmt::Debug>(), target: ComponentKey::of::<i32>() };
    assert!(!format!("{}", e).is_empty());
}

#[test]
fn resolve_error_construction() {
    let e = ResolveError::Construction { component: ComponentKey::of::<String>(), source: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "e")) };
    assert!(!format!("{}", e).is_empty());
}

#[test]
fn resolve_error_circular() {
    let e = ResolveError::CircularRuntime { path: vec!["a".into(), "b".into()] };
    assert!(!format!("{}", e).is_empty());
}

#[test]
fn resolve_error_provider_during_construction() {
    let e = ResolveError::ProviderUsedDuringConstruction { component: ComponentKey::of::<String>(), dependency: "d".into() };
    assert!(!format!("{}", e).is_empty());
}

#[test]
fn resolve_error_scope_not_active() {
    let e = ResolveError::ScopeNotActive { component: ComponentKey::of::<String>(), scope: vernal_beans::ScopeKey::of::<String>() };
    assert!(!format!("{}", e).is_empty());
}

#[test]
fn resolve_error_scope_owner_mismatch() {
    let e = ResolveError::ScopeOwnerMismatch { scope: vernal_beans::ScopeKey::of::<String>() };
    assert!(!format!("{}", e).is_empty());
}

// ── Dependency types ───────────────────────────────────────────────

#[test]
fn dependency_of() {
    let d = Dependency::of::<String>();
    assert!(!format!("{}", d).is_empty());
}

#[test]
fn dependency_qualified() {
    let q = Qualifier::new("q").unwrap();
    let d = Dependency::qualified::<String>(q);
    assert!(!format!("{}", d).is_empty());
}

#[test]
fn dependency_optional() {
    let d = Dependency::optional_of::<String>();
    assert!(!format!("{}", d).is_empty());
}

#[test]
fn dependency_provider() {
    let d = Dependency::provider_of::<String>();
    assert!(!format!("{}", d).is_empty());
}

#[test]
fn dependency_trait() {
    let d = Dependency::trait_of::<dyn std::fmt::Debug + Send + Sync>();
    assert!(!format!("{}", d).is_empty());
}

// ── PropertySource ─────────────────────────────────────────────────

#[test]
fn property_source_basic() {
    use vernal_beans::property_source::PropertySource;
    let mut ps = PropertySource::new("env");
    ps.set("KEY", "VALUE");
    assert_eq!(ps.get_property("KEY"), Some("VALUE"));
    assert!(ps.contains_property("KEY"));
    assert!(!ps.contains_property("X"));
    assert_eq!(ps.len(), 1);
    assert!(!ps.is_empty());
    assert_eq!(ps.name(), "env");
    assert_eq!(ps.property_names().len(), 1);
    assert!(ps.remove("KEY").is_some());
    assert_eq!(ps.len(), 0);
}

// ── PropertySources ────────────────────────────────────────────────

#[test]
fn property_sources_basic() {
    use vernal_beans::property_source::PropertySource;
    use vernal_beans::property_sources::PropertySources;
    let mut sources = PropertySources::new();
    assert!(sources.is_empty());
    sources.add(PropertySource::new("env"));
    assert_eq!(sources.len(), 1);
    assert!(sources.contains("env"));
    assert!(!sources.contains("x"));
    assert_eq!(sources.names(), vec!["env"]);
    let s = sources.get("env");
    assert!(s.is_some());
    let s = sources.get_mut("env");
    assert!(s.is_some());
    sources.clear();
    assert!(sources.is_empty());
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

#[test]
fn validation_errors_basic() {
    use vernal_beans::validation_errors::ValidationErrors;
    let mut v = ValidationErrors::new();
    assert!(!v.has_errors());
    v.add("e1");
    assert!(v.has_errors());
    assert_eq!(v.count(), 1);
}

// ── HTTP system ───────────────────────────────────────────────────

#[test]
fn http_status_basic() {
    use vernal_beans::http_status::HttpStatus;
    assert!(HttpStatus::ok().is_success());
    assert!(!HttpStatus::not_found().is_success());
    assert!(HttpStatus::internal_server_error().is_error());
}

#[test]
fn http_headers_basic() {
    use vernal_beans::http_headers::HttpHeaders;
    let mut h = HttpHeaders::new();
    h.set("Content-Type", "text/plain");
    assert!(h.contains("Content-Type"));
    assert_eq!(h.get_first("Content-Type"), Some("text/plain"));
    assert_eq!(h.get("Content-Type").unwrap().len(), 1);
    h.remove("Content-Type");
    assert!(!h.contains("Content-Type"));
}

#[test]
fn media_type_basic() {
    use vernal_beans::media_type::MediaType;
    let json = MediaType::application_json();
    let html = MediaType::text_html();
    let plain = MediaType::text_plain();
    assert!(json.matches(&json));
    assert!(!json.matches(&html));
    assert_eq!(json.to_string(), "application/json");
    assert_eq!(html.to_string(), "text/html");
    assert_eq!(plain.to_string(), "text/plain");
}

// ── Cache system ──────────────────────────────────────────────────

#[test]
fn simple_cache_basic() {
    use vernal_beans::cache::{Cache, SimpleCache};
    let c = SimpleCache::new();
    c.put("k", Arc::new(42i32));
    assert!(c.get("k").is_some());
    c.evict("k");
    assert!(c.get("k").is_none());
    c.put("k2", Arc::new("v"));
    c.clear();
    assert!(c.get("k2").is_none());
}

#[test]
fn cache_manager_basic() {
    use vernal_beans::cache_manager::CacheManager;
    let m = CacheManager::new();
    m.create_cache("c");
    assert!(m.get_cache("c").is_some());
    assert_eq!(m.cache_names().len(), 1);
    assert!(m.get_cache("nonexistent").is_none());
}

// ── ApplicationContext system ─────────────────────────────────────

#[test]
fn static_context_basic() {
    use vernal_beans::application_context::ApplicationContext;
    let ctx = vernal_beans::static_application_context::StaticApplicationContext::new("Test");
    ctx.register_bean("bean1", Arc::new(42i32));
    assert!(ctx.get_bean("bean1").is_ok());
    assert_eq!(ctx.get_display_name(), "Test");
    assert_eq!(ctx.get_startup_date(), 0);
    assert!(ctx.is_active());
}

#[test]
fn generic_context_basic() {
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = vernal_beans::generic_application_context::GenericApplicationContext::new();
    ctx.refresh();
    assert!(ctx.is_active());
    assert_eq!(ctx.get_display_name(), "GenericApplicationContext");
}

#[test]
fn annotation_context_basic() {
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = vernal_beans::annotation_config_application_context::AnnotationConfigApplicationContext::new();
    ctx.register("com.example.Config");
    assert_eq!(ctx.config_classes().len(), 1);
    ctx.refresh();
    assert!(ctx.is_active());
}

// ── Event system ──────────────────────────────────────────────────

#[test]
fn generic_event_basic() {
    use vernal_beans::application_event::ApplicationEvent;
    let e = vernal_beans::application_event::GenericApplicationEvent::new();
    assert!(e.timestamp > 0);
}

#[test]
fn lifecycle_event_basic() {
    use vernal_beans::application_event::ApplicationEvent;
    use vernal_beans::lifecycle_event::LifecycleEvent;
    assert!(LifecycleEvent::Started.get_timestamp() > 0);
    assert!(LifecycleEvent::Stopped.get_timestamp() > 0);
    assert!(LifecycleEvent::Refreshed.get_timestamp() > 0);
    assert!(LifecycleEvent::Closed.get_timestamp() > 0);
}

#[test]
fn simple_multicaster_basic() {
    use vernal_beans::application_event_multicaster::ApplicationEventMulticaster;
    let mc = vernal_beans::simple_application_event_multicaster::SimpleApplicationEventMulticaster::new();
    mc.multicast_event(Arc::new("event".to_string()) as Arc<dyn Any + Send + Sync>);
}

// ── Utility system ────────────────────────────────────────────────

#[test]
fn string_utils_has_text() {
    use vernal_beans::string_utils::StringUtils;
    assert!(StringUtils::has_text("hello"));
    assert!(!StringUtils::has_text("  "));
    assert!(StringUtils::has_length("a"));
    assert!(!StringUtils::has_length(""));
    assert!(StringUtils::starts_with("hello", "hel"));
    assert!(StringUtils::ends_with("hello", "llo"));
    assert_eq!(StringUtils::trim_whitespace("  hi  "), "hi");
}

#[test]
fn ant_path_matcher_basic() {
    use vernal_beans::ant_path_matcher::AntPathMatcher;
    let m = AntPathMatcher::new();
    assert!(m.matches("*", "anything"));
    assert!(!m.matches("a", "b"));
    let t = m.extract_path_template("/users/{id}");
    assert!(t.is_some());
}

#[test]
fn class_utils_basic() {
    use vernal_beans::class_utils::ClassUtils;
    assert_eq!(ClassUtils::get_short_name("com::example::Test"), "Test");
    assert_eq!(ClassUtils::get_package_name("com::example::Test"), "com::example");
}

#[test]
fn passport_basic() {
    let p = vernal_beans::passport::Passport::new("user", "pass");
    assert_eq!(p.principal(), "user");
    assert_eq!(p.credentials(), "pass");
}

#[test]
fn serializable_wrapper_basic() {
    let w = vernal_beans::serializable_type_wrapper::SerializableTypeWrapper::new("S");
    assert_eq!(w.type_name(), "S");
}

// ── Deep Container API coverage ───────────────────────────────────

#[test]
fn container_resolve_trait_err() {
    let c = make_container();
    let val: Result<Arc<dyn std::fmt::Debug + Send + Sync>, _> = c.resolve_trait();
    assert!(val.is_err());
}

#[test]
fn container_resolve_all_traits_empty() {
    let c = make_container();
    let val: Result<Vec<Arc<dyn std::fmt::Debug + Send + Sync>>, _> = c.resolve_all_traits();
    // No trait bindings registered, so returns empty vec (not error)
    assert!(val.unwrap().is_empty());
}

#[test]
fn container_resolve_in_scope_int() {
    let c = make_container();
    let scope = c.open_scope::<i32>();
    let val: Result<Arc<i32>, _> = c.resolve_in(&scope);
    assert!(val.is_ok());
    assert_eq!(*val.unwrap(), 42);
}

#[test]
fn container_open_scope_with_cancellation() {
    use tokio_util::sync::CancellationToken;
    let c = make_container();
    let ct = CancellationToken::new();
    let _scope = c.open_scope_with_cancellation::<String>(ct);
}

#[test]
fn container_bean_factory_contains_bean() {
    let c = make_container();
    assert!(c.contains_bean(&ComponentKey::of::<String>()));
    assert!(!c.contains_bean(&ComponentKey::of::<f64>()));
}

#[test]
fn container_bean_factory_get_type_none() {
    let c = make_container();
    let t = c.get_type(&ComponentKey::of::<f64>());
    assert!(t.is_err());
}

#[test]
fn container_get_bean_by_key_err() {
    let c = make_container();
    let result = c.get_bean_by_key(&ComponentKey::of::<f64>());
    assert!(result.is_err());
}

#[test]
fn container_is_singleton_true() {
    let c = make_container();
    assert!(c.is_singleton(&ComponentKey::of::<String>()).unwrap());
}

#[test]
fn container_is_prototype_false() {
    let c = make_container();
    assert!(!c.is_prototype(&ComponentKey::of::<String>()).unwrap());
}
