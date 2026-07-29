//! 综合覆盖测试 — 覆盖所有低覆盖率模块。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::BeanScope;
use vernal_beans::bean_expression_resolver::BeanExpressionResolver;

// ── field_metadata 测试 ──────────────────────────────────────────────────

#[test]
fn field_metadata_register_and_query_all() {
    use vernal_beans::field_metadata;
    field_metadata::clear_metadata();

    field_metadata::register_type_metadata(field_metadata::TypeMetadata {
        type_id: std::any::TypeId::of::<String>(),
        type_name: std::any::type_name::<String>(),
        fields: vec![field_metadata::FieldDescriptor::new(
            "inner",
            std::any::TypeId::of::<i32>(),
            std::any::type_name::<i32>(),
        )],
    });

    let all = field_metadata::get_all_metadata();
    assert!(!all.is_empty());

    let found = field_metadata::get_metadata_for_type(std::any::TypeId::of::<String>());
    assert!(found.is_some());

    let fields = field_metadata::find_fields_needing_type(std::any::TypeId::of::<i32>());
    assert!(!fields.is_empty());

    assert!(field_metadata::type_needs_injection(std::any::TypeId::of::<String>()));

    field_metadata::clear_metadata();
    assert!(field_metadata::get_all_metadata().is_empty());
}

#[test]
fn field_metadata_optional_and_qualifier() {
    use vernal_beans::field_metadata;
    field_metadata::clear_metadata();

    field_metadata::register_type_metadata(field_metadata::TypeMetadata {
        type_id: std::any::TypeId::of::<String>(),
        type_name: std::any::type_name::<String>(),
        fields: vec![field_metadata::FieldDescriptor::new(
            "inner",
            std::any::TypeId::of::<i32>(),
            std::any::type_name::<i32>(),
        )
        .with_optional()
        .with_qualifier("primary")],
    });

    let found = field_metadata::get_metadata_for_type(std::any::TypeId::of::<String>()).unwrap();
    assert!(found.fields[0].optional);
    assert_eq!(found.fields[0].qualifier, Some("primary"));

    field_metadata::clear_metadata();
}

#[test]
fn field_metadata_empty() {
    use vernal_beans::field_metadata;
    field_metadata::clear_metadata();
    assert!(field_metadata::get_all_metadata().is_empty());
    assert!(field_metadata::get_metadata_for_type(std::any::TypeId::of::<String>()).is_none());
    assert!(!field_metadata::type_needs_injection(std::any::TypeId::of::<String>()));
}

// ── field_metadata 测试（多类型）──────────────────────────────────────────

#[test]
fn field_metadata_multiple_types() {
    use vernal_beans::field_metadata;
    field_metadata::clear_metadata();

    field_metadata::register_type_metadata(field_metadata::TypeMetadata {
        type_id: std::any::TypeId::of::<String>(),
        type_name: std::any::type_name::<String>(),
        fields: vec![field_metadata::FieldDescriptor::new(
            "name",
            std::any::TypeId::of::<String>(),
            std::any::type_name::<String>(),
        )],
    });

    field_metadata::register_type_metadata(field_metadata::TypeMetadata {
        type_id: std::any::TypeId::of::<i32>(),
        type_name: std::any::type_name::<i32>(),
        fields: vec![field_metadata::FieldDescriptor::new(
            "value",
            std::any::TypeId::of::<String>(),
            std::any::type_name::<String>(),
        )],
    });

    let all = field_metadata::get_all_metadata();
    assert_eq!(all.len(), 2);

    let found = field_metadata::get_metadata_for_type(std::any::TypeId::of::<String>());
    assert!(found.is_some());
    assert_eq!(found.unwrap().fields[0].name, "name");

    let found = field_metadata::get_metadata_for_type(std::any::TypeId::of::<i32>());
    assert!(found.is_some());
    assert_eq!(found.unwrap().fields[0].name, "value");

    let fields = field_metadata::find_fields_needing_type(std::any::TypeId::of::<String>());
    assert!(fields.len() >= 2);

    field_metadata::clear_metadata();
}

// ── standard_bean_expression_resolver 测试 ────────────────────────────────

#[test]
fn expression_resolver_simple_identifier() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    assert!(resolver.evaluate("myBean", None).unwrap().is_none());
}

#[test]
fn expression_resolver_empty() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    assert!(resolver.evaluate("", None).unwrap().is_none());
}

#[test]
fn expression_resolver_non_identifier() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    assert!(resolver.evaluate("123abc", None).unwrap().is_none());
}

// ── root_bean_definition 测试 ────────────────────────────────────────────

#[test]
fn root_bean_definition_fluent() {
    let bd = vernal_beans::RootBeanDefinition::new()
        .with_bean_class_name("MyService")
        .with_scope(vernal_beans::Scope::Singleton)
        .with_lazy_init(false)
        .with_primary(true)
        .with_init_method("init")
        .with_destroy_method("destroy");

    assert_eq!(bd.bean_class_name(), "MyService");
    assert_eq!(bd.scope(), vernal_beans::Scope::Singleton);
    assert!(!bd.is_lazy_init());
    assert!(bd.is_primary());
    assert_eq!(bd.init_method_name(), Some("init"));
    assert_eq!(bd.destroy_method_name(), Some("destroy"));
}

#[test]
fn root_bean_definition_setters() {
    let mut bd = vernal_beans::RootBeanDefinition::new();
    bd.set_bean_class_name("Test");
    bd.set_scope(vernal_beans::Scope::Transient);
    bd.set_lazy_init(true);
    bd.set_primary(false);
    bd.set_abstract(true);
    bd.set_init_method_name("init");
    bd.set_destroy_method_name("destroy");
    bd.set_parent_name("Parent");
    bd.set_factory_bean_name("Factory");
    bd.set_factory_method_name("create");
    bd.set_init_order(100);

    assert_eq!(bd.bean_class_name(), "Test");
    assert_eq!(bd.scope(), vernal_beans::Scope::Transient);
    assert!(bd.is_lazy_init());
    assert!(!bd.is_primary());
    assert!(bd.is_abstract());
    assert_eq!(bd.init_method_name(), Some("init"));
    assert_eq!(bd.destroy_method_name(), Some("destroy"));
    assert_eq!(bd.parent_name(), Some("Parent"));
    assert_eq!(bd.factory_bean_name(), Some("Factory"));
    assert_eq!(bd.factory_method_name(), Some("create"));
    assert_eq!(bd.init_order_value(), 100);
}

#[test]
fn root_bean_definition_from_generic() {
    let mut generic = vernal_beans::GenericBeanDefinition::new();
    generic.set_bean_class_name("MyService");
    generic.set_scope(vernal_beans::Scope::Singleton);
    generic.set_init_method_name("init");

    let root = vernal_beans::RootBeanDefinition::from_generic(generic);
    assert_eq!(root.bean_class_name(), "MyService");
    assert_eq!(root.scope(), vernal_beans::Scope::Singleton);
    assert_eq!(root.init_method_name(), Some("init"));
}

#[test]
fn root_bean_definition_default() {
    let bd = vernal_beans::RootBeanDefinition::default();
    assert_eq!(bd.bean_class_name(), "unknown");
    assert_eq!(bd.scope(), vernal_beans::Scope::Singleton);
}

// ── generic_bean_definition 测试 ─────────────────────────────────────────

#[test]
fn generic_bean_definition_setters() {
    let mut bd = vernal_beans::GenericBeanDefinition::new();
    bd.set_bean_class_name("Test");
    bd.set_scope(vernal_beans::Scope::Transient);
    bd.set_lazy_init(true);
    bd.set_primary(true);
    bd.set_abstract(true);
    bd.set_init_method_name("init");
    bd.set_destroy_method_name("destroy");
    bd.set_parent_name("Parent");
    bd.set_factory_bean_name("Factory");
    bd.set_factory_method_name("create");
    bd.add_depends_on("Dep1");
    bd.add_depends_on("Dep2");
    bd.set_autowire_mode(vernal_beans::Autowire::ByType);

    assert_eq!(bd.get_bean_class_name(), Some("Test"));
    assert_eq!(bd.scope(), vernal_beans::Scope::Transient);
    assert!(bd.is_lazy_init());
    assert!(bd.is_primary());
    assert!(bd.is_abstract());
    assert_eq!(bd.init_method_name(), Some("init"));
    assert_eq!(bd.destroy_method_name(), Some("destroy"));
    assert_eq!(bd.get_parent_name(), Some("Parent"));
    assert_eq!(bd.factory_bean_name(), Some("Factory"));
    assert_eq!(bd.factory_method_name(), Some("create"));
    assert_eq!(bd.depends_on(), &["Dep1", "Dep2"]);
    assert_eq!(bd.autowire_mode(), vernal_beans::Autowire::ByType);
}

#[test]
fn generic_bean_definition_default() {
    let bd = vernal_beans::GenericBeanDefinition::default();
    assert_eq!(bd.get_bean_class_name(), None);
}

// ── bean_definition_builder 测试 ─────────────────────────────────────────

#[test]
fn bean_definition_builder_generic() {
    let bd = vernal_beans::BeanDefinitionBuilder::generic("MyService")
        .set_scope(vernal_beans::Scope::Singleton)
        .set_lazy_init(false)
        .set_primary(true)
        .set_init_method("init")
        .set_destroy_method("cleanup")
        .add_depends_on("DataSource")
        .add_depends_on("Config")
        .build();

    assert_eq!(bd.get_bean_class_name(), Some("MyService"));
    assert_eq!(bd.scope(), vernal_beans::Scope::Singleton);
    assert!(!bd.is_lazy_init());
    assert!(bd.is_primary());
    assert_eq!(bd.init_method_name(), Some("init"));
    assert_eq!(bd.destroy_method_name(), Some("cleanup"));
    assert_eq!(bd.depends_on(), &["DataSource", "Config"]);
}

#[test]
fn bean_definition_builder_root() {
    let bd = vernal_beans::BeanDefinitionBuilder::root("MyService")
        .set_scope(vernal_beans::Scope::Singleton)
        .build();

    assert_eq!(bd.bean_class_name(), "MyService");
    assert_eq!(bd.scope(), vernal_beans::Scope::Singleton);
}

// ── autowire 测试 ────────────────────────────────────────────────────────

#[test]
fn autowire_enum_all_values() {
    assert_eq!(vernal_beans::Autowire::No.as_int(), 0);
    assert_eq!(vernal_beans::Autowire::ByName.as_int(), 1);
    assert_eq!(vernal_beans::Autowire::ByType.as_int(), 2);
    assert_eq!(vernal_beans::Autowire::Constructor.as_int(), 3);
    assert_eq!(vernal_beans::Autowire::from_int(0), Some(vernal_beans::Autowire::No));
    assert_eq!(vernal_beans::Autowire::from_int(1), Some(vernal_beans::Autowire::ByName));
    assert_eq!(vernal_beans::Autowire::from_int(2), Some(vernal_beans::Autowire::ByType));
    assert_eq!(vernal_beans::Autowire::from_int(3), Some(vernal_beans::Autowire::Constructor));
    assert_eq!(vernal_beans::Autowire::from_int(4), None);
}

// ── constructor_argument_values 测试 ─────────────────────────────────────

#[test]
fn constructor_argument_values_basic() {
    let mut cav = vernal_beans::ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, vernal_beans::ValueHolder::new(Arc::new(42i32)));
    cav.add_indexed_argument_value(1, vernal_beans::ValueHolder::with_type(Arc::new("hello".to_string()), "String"));

    assert!(cav.has_indexed_argument_value(0));
    assert!(cav.has_indexed_argument_value(1));
    assert!(!cav.has_indexed_argument_value(2));
    assert_eq!(cav.argument_count(), 2);
    assert!(!cav.is_empty());
}

#[test]
fn constructor_argument_values_generic() {
    use vernal_beans::ValueHolder;
    let mut cav = vernal_beans::ConstructorArgumentValues::new();
    cav.add_generic_argument_value(ValueHolder::with_type_and_name(Arc::new(3.14f64), "f64", "pi"));

    let found = cav.get_generic_argument_value("f64");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name(), Some("pi"));
}

#[test]
fn constructor_argument_values_clear() {
    let mut cav = vernal_beans::ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, vernal_beans::ValueHolder::new(Arc::new(1i32)));
    assert_eq!(cav.argument_count(), 1);
    cav.clear();
    assert_eq!(cav.argument_count(), 0);
    assert!(cav.is_empty());
}

#[test]
fn constructor_argument_values_get_argument_value() {
    let mut cav = vernal_beans::ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, vernal_beans::ValueHolder::new(Arc::new(42i32)));

    let found = cav.get_argument_value(0, None, None);
    assert!(found.is_some());
    assert!(!found.unwrap().is_converted());
}

#[test]
fn constructor_argument_values_contains_named() {
    use vernal_beans::ValueHolder;
    let mut cav = vernal_beans::ConstructorArgumentValues::new();
    cav.add_generic_argument_value(ValueHolder::with_type_and_name(Arc::new(42i32), "i32", "count"));
    assert!(cav.contains_named_argument());
}

// ── type_converter_delegate 测试 ─────────────────────────────────────────

#[test]
fn type_converter_delegate_custom_converter() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    delegate.register_converter(
        std::any::TypeId::of::<String>(),
        std::any::TypeId::of::<i32>(),
        |value| {
            let s = value.downcast_ref::<String>().ok_or("Not a String")?;
            let v = s.trim().parse::<i32>().map_err(|e| format!("Parse error: {}", e))?;
            Ok(Box::new(v))
        },
    );

    let source = "42".to_string();
    let result = delegate.convert_if_necessary(
        Some("port"),
        &source,
        std::any::TypeId::of::<i32>(),
    );
    assert!(result.is_ok());
    let value = result.unwrap();
    let converted = value.downcast_ref::<i32>().unwrap();
    assert_eq!(*converted, 42);
}

// ── bean_expression_resolver 测试 ────────────────────────────────────────

#[test]
fn expression_resolver_with_context() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("myBean".to_string(), Arc::new(42i32));
    let result = resolver.evaluate("myBean", Some("other")).unwrap();
    assert!(result.is_some());
}

// ── placeholder_configurer 测试 ──────────────────────────────────────────

#[test]
fn placeholder_configurer_complex_nested() {
    let mut configurer = vernal_beans::placeholder_configurer::PlaceholderConfigurerSupport::new();
    configurer.set_property("a".to_string(), "${b}".to_string());
    configurer.set_property("b".to_string(), "resolved".to_string());

    let result = configurer.resolve_placeholder("${a}");
    assert_eq!(result, "resolved");
}

#[test]
fn placeholder_configurer_empty_prefix() {
    let mut configurer = vernal_beans::placeholder_configurer::PlaceholderConfigurerSupport::new();
    configurer.set_property("key".to_string(), "value".to_string());
    let result = configurer.resolve_placeholder("no_placeholder_here");
    assert_eq!(result, "no_placeholder_here");
}

// ── request_scope 测试 ────────────────────────────────────────────────────

#[test]
fn request_scope_basic() {
    let scope = vernal_beans::RequestScope::new("req-001");
    let obj = scope.get("test", &|| Box::new(42i32)).unwrap();
    // obj: Box<dyn Any + Send + Sync>，内部是 Arc<dyn Any + Send + Sync>
    let inner = obj.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    let val = inner.downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);
}

#[test]
fn request_scope_caches() {
    let scope = vernal_beans::RequestScope::new("req-002");
    let _obj1 = scope.get("test", &|| Box::new(42i32)).unwrap();
    let _obj2 = scope.get("test", &|| Box::new(100i32)).unwrap();
    assert_eq!(scope.cached_count(), 1);
}

#[test]
fn request_scope_remove() {
    let scope = vernal_beans::RequestScope::new("req-003");
    scope.get("test", &|| Box::new(42i32)).unwrap();
    let removed = scope.remove("test").unwrap();
    assert!(removed.is_some());
    assert_eq!(scope.cached_count(), 0);
}

#[test]
fn request_scope_destroy() {
    let scope = vernal_beans::RequestScope::new("req-004");
    scope.get("test", &|| Box::new(42i32)).unwrap();
    scope.register_destruction_callback("test", Box::new(|| {}));
    scope.destroy();
    assert_eq!(scope.cached_count(), 0);
}

#[test]
fn request_scope_conversation_id() {
    let scope = vernal_beans::RequestScope::new("req-005");
    assert_eq!(scope.conversation_id(), Some("req-005"));
}

#[test]
fn request_scope_contextual_object() {
    let scope = vernal_beans::RequestScope::new("req-006");
    let ctx = scope.resolve_contextual_object("request");
    assert!(ctx.is_some());
    let unwrapped = ctx.unwrap();
    let ctx_str = unwrapped.downcast_ref::<String>().unwrap();
    assert_eq!(ctx_str, "req-006");
}

#[test]
fn request_scope_bean_scope_trait() {
    let scope = vernal_beans::RequestScope::new("req-007");
    let _ = scope.get("test", &|| Box::new(42i32)).unwrap();
    let _ = scope.remove("test").unwrap();
    scope.register_destruction_callback("test", Box::new(|| {}));
    let _ = scope.resolve_contextual_object("request");
    let _ = scope.conversation_id();
}

// ── session_scope 测试 ───────────────────────────────────────────────────

#[test]
fn session_scope_basic() {
    let scope = vernal_beans::SessionScope::new("sess-001");
    let obj = scope.get("test", &|| Box::new(42i32)).unwrap();
    let inner = obj.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    let val = inner.downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);
}

#[test]
fn session_scope_caches() {
    let scope = vernal_beans::SessionScope::new("sess-002");
    let _obj1 = scope.get("test", &|| Box::new(42i32)).unwrap();
    let _obj2 = scope.get("100", &|| Box::new(100i32)).unwrap();
    assert_eq!(scope.cached_count(), 2);
}

#[test]
fn session_scope_remove() {
    let scope = vernal_beans::SessionScope::new("sess-003");
    scope.get("test", &|| Box::new(42i32)).unwrap();
    let removed = scope.remove("test").unwrap();
    assert!(removed.is_some());
}

#[test]
fn session_scope_destroy() {
    let scope = vernal_beans::SessionScope::new("sess-004");
    scope.get("test", &|| Box::new(42i32)).unwrap();
    scope.register_destruction_callback("test", Box::new(|| {}));
    scope.destroy();
    assert_eq!(scope.cached_count(), 0);
}

#[test]
fn session_scope_conversation_id() {
    let scope = vernal_beans::SessionScope::new("sess-005");
    assert_eq!(scope.conversation_id(), Some("sess-005"));
}

#[test]
fn session_scope_contextual_object() {
    let scope = vernal_beans::SessionScope::new("sess-006");
    let ctx = scope.resolve_contextual_object("session");
    assert!(ctx.is_some());
}

// ── application_scope 测试 ────────────────────────────────────────────────

#[test]
fn application_scope_basic() {
    let scope = vernal_beans::ApplicationScope::new();
    let obj = scope.get("test", &|| Box::new(42i32)).unwrap();
    let inner = obj.downcast_ref::<Arc<dyn Any + Send + Sync>>().unwrap();
    let val = inner.downcast_ref::<i32>().unwrap();
    assert_eq!(*val, 42);
}

#[test]
fn application_scope_caches() {
    let scope = vernal_beans::ApplicationScope::new();
    let _obj1 = scope.get("test", &|| Box::new(42i32)).unwrap();
    let _obj2 = scope.get("test", &|| Box::new(100i32)).unwrap();
    assert_eq!(scope.cached_count(), 1);
}

#[test]
fn application_scope_remove() {
    let scope = vernal_beans::ApplicationScope::new();
    scope.get("test", &|| Box::new(42i32)).unwrap();
    let removed = scope.remove("test").unwrap();
    assert!(removed.is_some());
}

#[test]
fn application_scope_destroy() {
    let scope = vernal_beans::ApplicationScope::new();
    scope.get("test", &|| Box::new(42i32)).unwrap();
    scope.register_destruction_callback("test", Box::new(|| {}));
    scope.destroy();
    assert_eq!(scope.cached_count(), 0);
}

#[test]
fn application_scope_conversation_id() {
    let scope = vernal_beans::ApplicationScope::new();
    assert_eq!(scope.conversation_id(), Some("application"));
}

#[test]
fn application_scope_contextual_object() {
    let scope = vernal_beans::ApplicationScope::new();
    let ctx = scope.resolve_contextual_object("application");
    assert!(ctx.is_some());
}

// ── mutable_property_values 测试 ─────────────────────────────────────────

#[test]
fn mutable_property_values_large_batch() {
    let mut pvs = vernal_beans::MutablePropertyValues::new();
    for i in 0..100 {
        pvs.add_value(format!("key_{}", i), Arc::new(i as i32));
    }
    assert_eq!(pvs.len(), 100);

    for i in 0..100 {
        assert!(pvs.contains(&format!("key_{}", i)));
    }
}

#[test]
fn mutable_property_values_get_ref() {
    let mut pvs = vernal_beans::MutablePropertyValues::new();
    pvs.add_value("name", Arc::new("Alice".to_string()));
    let pv = pvs.get("name").unwrap();
    let val = pv.value().downcast_ref::<String>().unwrap();
    assert_eq!(val, "Alice");
}

#[test]
fn mutable_property_values_get_property_values_ref() {
    let mut pvs = vernal_beans::MutablePropertyValues::new();
    pvs.add_value("a", Arc::new(1i32));
    pvs.add_value("b", Arc::new(2i32));

    let values = pvs.get_property_values();
    assert_eq!(values.len(), 2);
    for pv in values {
        assert!(!pv.name().is_empty());
    }
}

// ── property_editor 批量测试 ──────────────────────────────────────────────

#[test]
fn property_editor_all_types() {
    use vernal_beans::PropertyEditor as PE;

    let mut e1 = vernal_beans::string_trimmer_editor::StringTrimmerEditor::new();
    e1.set_as_text("test_value").unwrap();
    assert!(e1.get_value().is_some());

    let mut e2 = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    e2.set_as_text("true").unwrap();
    assert!(e2.get_value().is_some());

    let mut e3 = vernal_beans::number_editor::CustomNumberEditor::new();
    e3.set_as_text("42").unwrap();
    assert!(e3.get_value().is_some());

    let mut e4 = vernal_beans::uri_editor::URIEditor::new();
    e4.set_as_text("https://example.com").unwrap();
    assert!(e4.get_value().is_some());

    let mut e5 = vernal_beans::uuid_editor::UUIDEditor::new();
    e5.set_as_text("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert!(e5.get_value().is_some());

    let mut e6 = vernal_beans::file_editor::FileEditor::new();
    e6.set_as_text("test_value").unwrap();
    assert!(e6.get_value().is_some());

    let mut e7 = vernal_beans::class_editor::ClassEditor::new();
    e7.set_as_text("java.lang.String").unwrap();
    assert!(e7.get_value().is_some());

    let mut e8 = vernal_beans::locale_editor::LocaleEditor::new();
    e8.set_as_text("en_US").unwrap();
    assert!(e8.get_value().is_some());

    let mut e9 = vernal_beans::charset_editor::CharsetEditor::new();
    e9.set_as_text("UTF-8").unwrap();
    assert!(e9.get_value().is_some());

    let mut e10 = vernal_beans::path_editor::PathEditor::new();
    e10.set_as_text("/tmp/test").unwrap();
    assert!(e10.get_value().is_some());

    let mut e11 = vernal_beans::pattern_editor::PatternEditor::new();
    e11.set_as_text("\\d+").unwrap();
    assert!(e11.get_value().is_some());

    let mut e12 = vernal_beans::timezone_editor::TimeZoneEditor::new();
    e12.set_as_text("UTC").unwrap();
    assert!(e12.get_value().is_some());

    let mut e13 = vernal_beans::zone_id_editor::ZoneIdEditor::new();
    e13.set_as_text("Asia/Shanghai").unwrap();
    assert!(e13.get_value().is_some());

    let mut e14 = vernal_beans::input_stream_editor::InputStreamEditor::new();
    e14.set_as_text("test_value").unwrap();
    assert!(e14.get_value().is_some());

    let mut e15 = vernal_beans::char_array_editor::CharArrayPropertyEditor::new();
    e15.set_as_text("test_value").unwrap();
    assert!(e15.get_value().is_some());

    let mut e16 = vernal_beans::byte_array_editor::ByteArrayPropertyEditor::new();
    e16.set_as_text("test_value").unwrap();
    assert!(e16.get_value().is_some());

    let mut e17 = vernal_beans::properties_editor::PropertiesEditor::new();
    e17.set_as_text("key=value").unwrap();
    assert!(e17.get_value().is_some());

    let mut e18 = vernal_beans::reader_editor::ReaderEditor::new();
    e18.set_as_text("test_value").unwrap();
    assert!(e18.get_value().is_some());

    let mut e19 = vernal_beans::string_array_editor::StringArrayPropertyEditor::new();
    e19.set_as_text("a,b,c").unwrap();
    assert!(e19.get_value().is_some());

    let mut e20 = vernal_beans::class_array_editor::ClassArrayEditor::new();
    e20.set_as_text("String,Integer").unwrap();
    assert!(e20.get_value().is_some());

    let mut e21 = vernal_beans::currency_editor::CurrencyEditor::new();
    e21.set_as_text("USD").unwrap();
    assert!(e21.get_value().is_some());

    let mut e22 = vernal_beans::input_source_editor::InputSourceEditor::new();
    e22.set_as_text("test_value").unwrap();
    assert!(e22.get_value().is_some());

    let mut e23 = vernal_beans::char_property_editor::CharacterEditor::new();
    e23.set_as_text("A").unwrap();
    assert!(e23.get_value().is_some());

    let mut e24 = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    e24.set_as_text("UTF-8").unwrap();
    assert!(e24.get_value().is_some());

    let mut e25 = vernal_beans::path_property_editor::PathPropertyEditor::new();
    e25.set_as_text("test_value").unwrap();
    assert!(e25.get_value().is_some());

    let mut e26 = vernal_beans::file_array_editor::FileArrayEditor::new();
    e26.set_as_text("test_value").unwrap();
    assert!(e26.get_value().is_some());

    let mut e27 = vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor::new();
    e27.set_as_text("test_value").unwrap();
    assert!(e27.get_value().is_some());

    let mut e28 = vernal_beans::resource_bundle_editor::ResourceBundleEditor::new();
    e28.set_as_text("test_value").unwrap();
    assert!(e28.get_value().is_some());
}
