//! Tests targeting 0%-covered files to maximize coverage.
use std::any::Any;
use std::sync::Arc;

// ── ManagedProperties ─────────────────────────────────────────────

#[test]
fn managed_properties_full() {
    use vernal_beans::managed_properties::ManagedProperties;
    let mut mp = ManagedProperties::new();
    assert!(mp.is_empty());
    assert_eq!(mp.len(), 0);
    mp.set("key1", "val1");
    mp.set("key2", "val2");
    assert_eq!(mp.len(), 2);
    assert!(!mp.is_empty());
    assert_eq!(mp.get("key1"), Some("val1"));
    assert!(mp.contains_key("key1"));
    assert!(!mp.contains_key("key3"));
}

// ── ManagedMap ────────────────────────────────────────────────────

#[test]
fn managed_map_full() {
    use vernal_beans::managed_map::ManagedMap;
    let mut m = ManagedMap::new();
    assert!(m.is_empty());
    assert_eq!(m.len(), 0);
    m.insert("a", 1);
    m.insert("b", 2);
    assert_eq!(m.len(), 2);
    assert!(!m.is_empty());
    assert_eq!(m.get(&"a"), Some(&1));
    assert!(m.contains_key(&"a"));
    let old = m.insert("a", 10);
    assert_eq!(old, Some(1));
}

#[test]
fn managed_map_from_hashmap() {
    use vernal_beans::managed_map::ManagedMap;
    let mut hm = std::collections::HashMap::new();
    hm.insert("x", 42);
    let m = ManagedMap::from_hashmap(hm);
    assert_eq!(m.len(), 1);
}

// ── ManagedList ───────────────────────────────────────────────────

#[test]
fn managed_list_full() {
    use vernal_beans::managed_list::ManagedList;
    let mut l = ManagedList::new();
    assert!(l.is_empty());
    l.push(1);
    l.push(2);
    l.push(3);
    assert_eq!(l.len(), 3);
    let items: Vec<&i32> = l.iter().collect();
    assert_eq!(items, vec![&1, &2, &3]);
}

#[test]
fn managed_list_from_vec() {
    use vernal_beans::managed_list::ManagedList;
    let l = ManagedList::from_vec(vec![10, 20, 30]);
    assert_eq!(l.len(), 3);
    let v = l.into_vec();
    assert_eq!(v, vec![10, 20, 30]);
}

// ── ManagedSet ────────────────────────────────────────────────────

#[test]
fn managed_set_full() {
    use vernal_beans::managed_set::ManagedSet;
    let mut s = ManagedSet::new();
    assert!(s.is_empty());
    assert!(s.insert(1));
    assert!(s.insert(2));
    assert!(!s.insert(1));
    assert_eq!(s.len(), 2);
    assert!(s.contains(&1));
}

// ── ManagedArray ──────────────────────────────────────────────────

#[test]
fn managed_array_full() {
    use vernal_beans::managed_array::ManagedArray;
    let mut a = ManagedArray::new();
    assert!(a.is_empty());
    a.push(1);
    a.push(2);
    assert_eq!(a.len(), 2);
    let v = a.into_vec();
    assert_eq!(v, vec![1, 2]);
}

// ── ModelView ─────────────────────────────────────────────────────

#[test]
fn model_view_full() {
    use vernal_beans::model_view::ModelView;
    let mut mv = ModelView::new("test_view");
    assert_eq!(mv.view_name(), "test_view");
    assert!(mv.is_empty());
    mv.add_object("key1", Box::new(42i32));
    mv.add_object("key2", Box::new("hello".to_string()));
    assert_eq!(mv.size(), 2);
    let obj = mv.get_object("key1");
    assert!(obj.is_some());
    assert_eq!(obj.unwrap().downcast_ref::<i32>(), Some(&42));
    assert!(mv.get_object("nonexistent").is_none());
}

// ── MultipartFile ─────────────────────────────────────────────────

#[test]
fn multipart_file_full() {
    use vernal_beans::multipart::MultipartFile;
    let f = MultipartFile::new("upload")
        .with_data(vec![1, 2, 3, 4])
        .with_content_type("application/octet-stream");
    assert_eq!(f.name(), "upload");
    assert_eq!(f.data(), &[1, 2, 3, 4]);
    assert_eq!(f.size(), 4);
}

// ── MethodDescriptor ──────────────────────────────────────────────

#[test]
fn method_descriptor_full() {
    use vernal_beans::method_descriptor::MethodDescriptor;
    let md = MethodDescriptor::new("doSomething")
        .with_return_type("void")
        .with_parameters(vec!["String".into(), "int".into()]);
    assert_eq!(md.get_name(), "doSomething");
    assert_eq!(md.get_return_type(), "void");
    assert_eq!(md.parameter_count(), 2);
}

// ── MethodOverrides ───────────────────────────────────────────────

#[test]
fn method_overrides_full() {
    use vernal_beans::method_overrides::MethodOverrides;
    use vernal_beans::lookup_override::LookupOverride;
    let mut mo = MethodOverrides::new();
    assert!(mo.is_empty());
    let lo = LookupOverride::new("doSomething", "myBean");
    mo.add("doSomething", Box::new(lo));
    assert_eq!(mo.len(), 1);
    assert!(mo.get("doSomething").is_some());
}

// ── ChildBeanDefinition ───────────────────────────────────────────

#[test]
fn child_bean_definition_full() {
    use vernal_beans::child_bean_definition::ChildBeanDefinition;
    use vernal_beans::bean_definition::BeanDefinition;
    let cbd = ChildBeanDefinition::new("parentBean");
    assert_eq!(cbd.parent_name(), "parentBean");
    assert_eq!(cbd.scope(), vernal_beans::Scope::Singleton);
    assert!(cbd.is_singleton());
}

// ── AnnotatedGenericBeanDefinition ────────────────────────────────

#[test]
fn annotated_generic_bean_definition_full() {
    use vernal_beans::annotated_generic_bean_definition::AnnotatedGenericBeanDefinition;
    use vernal_beans::bean_definition::BeanDefinition;
    let def = AnnotatedGenericBeanDefinition::new("com.example.MyAnnotation");
    assert_eq!(def.bean_class_name(), "unknown");
    assert!(def.is_singleton());
}

// ── StandardBeanExpressionResolver ────────────────────────────────

#[test]
fn standard_bean_expression_resolver_full() {
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;
    let resolver = StandardBeanExpressionResolver::new();
    assert_eq!(resolver.bean_count(), 0);
    resolver.register_bean("myBean".to_string(), Arc::new(42i32));
    assert_eq!(resolver.bean_count(), 1);
    resolver.clear_context();
    assert_eq!(resolver.bean_count(), 0);
}

// ── ApplicationEventMulticasterImpl ───────────────────────────────

#[test]
fn application_event_multicaster_impl_full() {
    use vernal_beans::application_event_multicaster_impl::ApplicationEventMulticasterImpl;
    use vernal_beans::application_event_multicaster::ApplicationEventMulticaster;
    let mc = ApplicationEventMulticasterImpl::new();
    assert_eq!(mc.listener_count(), 0);
    let event: Arc<dyn Any + Send + Sync> = Arc::new("event".to_string());
    mc.multicast_event(event);
}

// ── FileSystemResourceImpl ────────────────────────────────────────

#[test]
fn filesystem_resource_impl_full() {
    use vernal_beans::filesystem_resource_impl::FileSystemResourceImpl;
    let r = FileSystemResourceImpl::new("/tmp/test.txt");
    assert_eq!(r.path().to_str().unwrap(), "/tmp/test.txt");
    assert_eq!(r.filename(), Some("test.txt"));
}

// ── PluggableSchemaResolverImpl ───────────────────────────────────

#[test]
fn pluggable_schema_resolver_impl_new() {
    use vernal_beans::pluggable_schema_resolver_impl::PluggableSchemaResolverImpl;
    let _r = PluggableSchemaResolverImpl::new();
}

#[test]
fn pluggable_schema_resolver_impl_from_text() {
    use vernal_beans::pluggable_schema_resolver_impl::PluggableSchemaResolverImpl;
    let _r = PluggableSchemaResolverImpl::from_text("http://example.com=example.xsd");
}

// ── LookupOverride ────────────────────────────────────────────────

#[test]
fn lookup_override_full() {
    use vernal_beans::lookup_override::LookupOverride;
    let lo = LookupOverride::new("doSomething", "myBean");
    assert_eq!(lo.get_method_name(), "doSomething");
    assert_eq!(lo.get_bean_name(), "myBean");
}

// ── AbstractApplicationContext ────────────────────────────────────

#[test]
fn abstract_application_context_full() {
    use vernal_beans::abstract_application_context::AbstractApplicationContext;
    use vernal_beans::application_context::ApplicationContext;
    let mut ctx = AbstractApplicationContext::new("TestContext");
    assert_eq!(ctx.get_display_name(), "TestContext");
    assert!(!ctx.is_active());
    ctx.refresh();
    assert!(ctx.is_active());
}

// ── AbstractApplicationEventMulticaster ───────────────────────────

#[test]
fn abstract_event_multicaster_full() {
    use vernal_beans::abstract_application_event_multicaster::AbstractApplicationEventMulticaster;
    use vernal_beans::application_event_multicaster::ApplicationEventMulticaster;
    let mc = AbstractApplicationEventMulticaster::new();
    let event: Arc<dyn Any + Send + Sync> = Arc::new("event".to_string());
    mc.multicast_event(event);
}

// ── Multicaster with listener ─────────────────────────────────────

#[test]
fn multicaster_impl_with_listener() {
    use vernal_beans::application_event_multicaster_impl::ApplicationEventMulticasterImpl;
    use vernal_beans::application_event_multicaster::ApplicationEventMulticaster;
    use vernal_beans::application_listener::ApplicationListener;

    struct TestListener;
    impl ApplicationListener for TestListener {
        fn on_application_event(&self, _event: Arc<dyn Any + Send + Sync>) {}
    }

    let mut mc = ApplicationEventMulticasterImpl::new();
    let listener: Arc<dyn ApplicationListener> = Arc::new(TestListener);
    mc.add_application_listener(listener);
    assert_eq!(mc.listener_count(), 1);
    let event: Arc<dyn Any + Send + Sync> = Arc::new("event".to_string());
    mc.multicast_event(event);
}

#[test]
fn abstract_multicaster_with_listener() {
    use vernal_beans::abstract_application_event_multicaster::AbstractApplicationEventMulticaster;
    use vernal_beans::application_event_multicaster::ApplicationEventMulticaster;
    use vernal_beans::application_listener::ApplicationListener;

    struct TestListener;
    impl ApplicationListener for TestListener {
        fn on_application_event(&self, _event: Arc<dyn Any + Send + Sync>) {}
    }

    let mut mc = AbstractApplicationEventMulticaster::new();
    mc.add_application_listener(Arc::new(TestListener));
    let event: Arc<dyn Any + Send + Sync> = Arc::new("event".to_string());
    mc.multicast_event(event);
}

// ── Qualifier ─────────────────────────────────────────────────────

#[test]
fn qualifier_basic() {
    use vernal_beans::Qualifier;
    let q = Qualifier::new("primary").unwrap();
    assert_eq!(q.as_str(), "primary");
}

// ── ConversionService ─────────────────────────────────────────────

#[test]
fn conversion_service_basic() {
    use vernal_beans::conversion_service_new::ConversionServiceNew;
    let cs = ConversionServiceNew::new();
    let result = cs.convert(Box::new("hello".to_string()), std::any::TypeId::of::<String>());
    assert!(result.is_err());
}

// ── SimpleBeanDefinitionRegistry ──────────────────────────────────

#[test]
fn simple_bean_definition_registry_basic() {
    use vernal_beans::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    let reg = SimpleBeanDefinitionRegistry::new();
    assert_eq!(reg.bean_definition_count(), 0);
    assert!(reg.bean_definition_names().is_empty());
    assert!(!reg.contains_bean_definition("test"));
}

// ── StaticListableBeanFactory ─────────────────────────────────────

#[test]
fn static_listable_bean_factory_basic() {
    use vernal_beans::static_listable_bean_factory::StaticListableBeanFactory;
    let mut bf = StaticListableBeanFactory::new();
    bf.register_singleton("bean1", Arc::new(42i32));
    assert!(bf.contains_bean_name("bean1"));
    assert_eq!(bf.bean_names().len(), 1);
}

// ── BeanDefinitionHolder ──────────────────────────────────────────

#[test]
fn bean_definition_holder_basic() {
    use vernal_beans::bean_definition_holder::BeanDefinitionHolder;
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    let def = RootBeanDefinition::new();
    let holder = BeanDefinitionHolder::new("myBean", Arc::new(def));
    assert_eq!(holder.bean_name(), "myBean");
}

// ── InjectionMetadata ─────────────────────────────────────────────

#[test]
fn injection_metadata_basic() {
    use vernal_beans::injection_metadata::InjectionMetadata;
    let meta = InjectionMetadata::new();
    assert!(meta.is_empty());
    assert_eq!(meta.element_count(), 0);
}

// ── AutowiredFieldElement ─────────────────────────────────────────

#[test]
fn autowired_field_element_basic() {
    use vernal_beans::autowired_field_element::AutowiredFieldElement;
    let elem = AutowiredFieldElement::new("pool", true);
    assert_eq!(elem.get_name(), "pool");
    assert!(elem.is_required());
}

// ── AutowiredMethodElement ────────────────────────────────────────

#[test]
fn autowired_method_element_basic() {
    use vernal_beans::autowired_method_element::AutowiredMethodElement;
    let elem = AutowiredMethodElement::new("setPool", vec![std::any::TypeId::of::<String>()]);
    assert_eq!(elem.get_method_name(), "setPool");
    assert_eq!(elem.get_parameter_type_ids().len(), 1);
}

// ── InjectedElement ───────────────────────────────────────────────

#[test]
fn injected_element_basic() {
    use vernal_beans::injected_element::InjectedElement;
    let mut elem = InjectedElement::new("pool", false);
    assert_eq!(elem.get_name(), "pool");
    assert!(!elem.is_required());
    elem.inject(&mut "target").unwrap();
}

// ── HTTP RedirectView ─────────────────────────────────────────────

#[test]
fn redirect_view_basic() {
    use vernal_beans::redirect_view::RedirectView;
    let rv = RedirectView::new("/home");
    assert_eq!(rv.get_url(), "/home");
    assert_eq!(rv.get_status_code(), 302);
}

#[test]
fn redirect_view_with_status() {
    use vernal_beans::redirect_view::RedirectView;
    let rv = RedirectView::new("/login").with_status_code(301);
    assert_eq!(rv.get_status_code(), 301);
}

// ── HTTP RequestMapping ───────────────────────────────────────────

#[test]
fn request_mapping_basic() {
    use vernal_beans::request_mapping::RequestMapping;
    let rm = RequestMapping::new("/api/users", "GET");
    assert_eq!(rm.path(), "/api/users");
    assert_eq!(rm.method(), "GET");
}

// ── HTTP ResponseBody ─────────────────────────────────────────────

#[test]
fn response_body_basic() {
    use vernal_beans::response_body::ResponseBody;
    let rb = ResponseBody::new(b"OK".to_vec(), "text/plain");
    assert_eq!(rb.content(), b"OK");
    assert_eq!(rb.content_type(), "text/plain");
    assert_eq!(rb.size(), 2);
}

// ── HTTP RequestBody ──────────────────────────────────────────────

#[test]
fn request_body_basic() {
    use vernal_beans::request_body::RequestBody;
    let rb = RequestBody::new(b"{\"key\":\"value\"}".to_vec(), "application/json");
    assert_eq!(rb.data(), b"{\"key\":\"value\"}");
    assert_eq!(rb.content_type(), "application/json");
}

// ── RootBeanDefinition ────────────────────────────────────────────

#[test]
fn root_bean_definition_basic() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    use vernal_beans::bean_definition::BeanDefinition;
    let rbd = RootBeanDefinition::new();
    assert_eq!(rbd.scope(), vernal_beans::Scope::Singleton);
    assert!(rbd.is_singleton());
}

#[test]
fn root_bean_definition_setters() {
    use vernal_beans::root_bean_definition::RootBeanDefinition;
    use vernal_beans::bean_definition::BeanDefinition;
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("com.example.MyService");
    assert_eq!(rbd.bean_class_name(), "com.example.MyService");
    rbd.set_scope(vernal_beans::Scope::Transient);
    assert_eq!(rbd.scope(), vernal_beans::Scope::Transient);
    rbd.set_lazy_init(true);
    assert!(rbd.is_lazy_init());
}

// ── GenericBeanDefinition ─────────────────────────────────────────

#[test]
fn generic_bean_definition_basic() {
    use vernal_beans::generic_bean_definition::GenericBeanDefinition;
    let gbd = GenericBeanDefinition::new();
    assert_eq!(gbd.scope(), vernal_beans::Scope::Singleton);
}

// ── ResolveError variants ─────────────────────────────────────────

#[test]
fn resolve_error_all_variants_display() {
    use vernal_beans::{ResolveError, ComponentKey, TraitKey, ScopeKey};
    let errors: Vec<ResolveError> = vec![
        ResolveError::NotFound { component: "t".into(), path: vec!["r".into()] },
        ResolveError::Ambiguous { component: "t".into(), candidates: vec!["a".into()], path: vec!["r".into()] },
        ResolveError::UndeclaredDependency { component: ComponentKey::of::<String>(), dependency: "d".into() },
        ResolveError::TypeMismatch { component: ComponentKey::of::<String>() },
        ResolveError::TraitBindingTypeMismatch { binding: TraitKey::of::<dyn std::fmt::Debug>(), target: ComponentKey::of::<i32>() },
        ResolveError::Construction { component: ComponentKey::of::<String>(), source: Arc::new(std::io::Error::new(std::io::ErrorKind::Other, "e")) },
        ResolveError::CircularRuntime { path: vec!["a".into(), "b".into()] },
        ResolveError::ProviderUsedDuringConstruction { component: ComponentKey::of::<String>(), dependency: "d".into() },
        ResolveError::ScopeNotActive { component: ComponentKey::of::<String>(), scope: ScopeKey::of::<String>() },
        ResolveError::ScopeOwnerMismatch { scope: ScopeKey::of::<String>() },
    ];
    for e in &errors {
        let s = format!("{}", e);
        assert!(!s.is_empty());
    }
}

// ── BeanWiringInfo ────────────────────────────────────────────────

#[test]
fn bean_wiring_info_full() {
    use vernal_beans::bean_wiring_info::BeanWiringInfo;
    let info = BeanWiringInfo::new("myBean", "com.example.MyBean");
    assert_eq!(info.get_bean_name(), "myBean");
    assert_eq!(info.get_type_name(), "com.example.MyBean");
    assert!(info.is_default_dependency());
}

// ── Container deep coverage ───────────────────────────────────────

#[test]
fn container_resolve_multiple_types() {
    use vernal_beans::{Container, RegistryBuilder, ComponentDefinition};
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    b.register(ComponentDefinition::singleton::<f64, _>(|_| 3.14f64));
    let c = Container::new(b.build().unwrap());
    let s: Result<Arc<String>, _> = c.resolve();
    let i: Result<Arc<i32>, _> = c.resolve();
    let f: Result<Arc<f64>, _> = c.resolve();
    assert!(s.is_ok());
    assert!(i.is_ok());
    assert!(f.is_ok());
}

#[test]
fn container_warm_up_and_unused() {
    use vernal_beans::{Container, RegistryBuilder, ComponentDefinition};
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()));
    let c = Container::new(b.build().unwrap());
    assert!(c.warm_up().is_ok());
    let unused = c.unused_definitions();
    assert!(unused.is_empty());
}
