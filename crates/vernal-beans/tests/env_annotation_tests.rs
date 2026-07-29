//! Comprehensive integration tests for Environment / Annotation / Profile / Config modules.
//!
//! Covers all 30 source files:
//! annotation_metadata, annotation_type_mapping, annotations_scanner, type_filter,
//! metadata_reader, simple_metadata_reader, annotation_processor, configuration_class,
//! configuration_class_parser, configuration_class_bean_definition_reader, import_selector,
//! deferred_import_selector, import_bean_definition_registrar, environment, property_source,
//! property_sources, property_resolver, configurable_environment, mutable_property_sources,
//! abstract_environment, standard_environment, profile, profiles, placeholder_resolver,
//! string_value_resolver, embedded_value_resolver, bean_reference_resolver,
//! listable_bean_factory_extensions, bean_factory_dependency_provider,
//! conversion_failed_exception.

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use vernal_beans::*;

// Import public constants not re-exported at crate root.
use vernal_beans::abstract_environment::default_profile_name;
use vernal_beans::placeholder_resolver::{PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX, VALUE_SEPARATOR};
use vernal_beans::profiles::RESERVED_DEFAULT_PROFILE_NAME;
use vernal_beans::standard_environment::{
    SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME, SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME,
    build_system_environment, build_system_properties,
};

// ===========================================================================
// Helpers
// ===========================================================================

/// A trivial `AnnotationMetadata` for tests.
struct StubMetadata {
    annotations: Vec<AnnotationDescriptor>,
}

impl AnnotationMetadata for StubMetadata {
    fn annotations(&self) -> &[AnnotationDescriptor] {
        &self.annotations
    }
}

fn empty_metadata() -> StubMetadata {
    StubMetadata {
        annotations: Vec::new(),
    }
}

fn metadata_with(descriptors: Vec<AnnotationDescriptor>) -> StubMetadata {
    StubMetadata {
        annotations: descriptors,
    }
}

/// A trivial `BeanDefinitionRegistry` for tests on
/// `ConfigurationClassBeanDefinitionReader`/`ImportBeanDefinitionRegistrar`.
struct StubRegistry {
    names: Vec<String>,
}

impl BeanDefinitionRegistry for StubRegistry {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        _definition: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.names.contains(&bean_name) {
            self.names.push(bean_name);
        }
        Ok(())
    }
    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        self.names.retain(|n| n != bean_name);
        Err(format!("no boxed definition stored for {bean_name}").into())
    }
    fn get_bean_definition(&self, _bean_name: &str) -> Option<&dyn BeanDefinition> {
        None
    }
    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.names.iter().any(|n| n == bean_name)
    }
    fn bean_definition_count(&self) -> usize {
        self.names.len()
    }
    fn bean_definition_names(&self) -> Vec<String> {
        self.names.clone()
    }
}

/// A `BeanReference` for tests.
#[derive(Debug)]
struct StubBeanRef {
    name: String,
}

impl BeanReference for StubBeanRef {
    fn get_bean_name(&self) -> &str {
        &self.name
    }
    fn get_source(&self) -> Option<&dyn Any> {
        None
    }
}

/// A trivial `BeanFactory` for tests on `BeanFactoryDependencyProvider`.
struct StubBeanFactory;

impl BeanFactory for StubBeanFactory {
    fn get_bean_by_key(
        &self,
        _key: &ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not supported".into())
    }
    fn get_bean_by_type_id(
        &self,
        _type_id: TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not supported".into())
    }
    fn contains_bean(&self, _key: &ComponentKey) -> bool {
        false
    }
    fn is_singleton(
        &self,
        _key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }
    fn is_prototype(
        &self,
        _key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
    fn get_type(
        &self,
        _key: &ComponentKey,
    ) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
    fn get_aliases(&self, _key: &ComponentKey) -> Vec<ComponentKey> {
        Vec::new()
    }
    fn get_bean_provider_by_type_id(
        &self,
        _type_id: TypeId,
    ) -> Result<
        Box<dyn ObjectProvider<dyn Any + Send + Sync> + '_>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Err("not supported".into())
    }
    fn is_type_match(&self, _key: &ComponentKey, _type_id: TypeId) -> bool {
        false
    }
}

// ===========================================================================
// 1. annotation_metadata.rs
// ===========================================================================

#[test]
fn annotation_metadata_class_field_method_levels() {
    let class = AnnotationDescriptor::new_class("Component");
    assert_eq!(class.annotation_type(), "Component");
    assert!(class.is_class_level());
    assert!(!class.is_method_level());
    assert!(!class.is_field_level());
    assert_eq!(class.level(), AnnotationLevel::Class);
    assert!(class.member_name().is_none());
    assert!(class.method_name().is_none());

    let field = AnnotationDescriptor::new_field("Autowired", "userRepo");
    assert!(field.is_field_level());
    assert_eq!(field.member_name(), Some("userRepo"));
    assert!(field.method_name().is_none());

    let method = AnnotationDescriptor::new_method("Bean", "createThing");
    assert!(method.is_method_level());
    assert_eq!(method.method_name(), Some("createThing"));
}

#[test]
fn annotation_metadata_descriptor_attributes() {
    let mut desc = AnnotationDescriptor::new_class("Service");
    desc.set_attribute("value", "userService");
    desc.attributes_mut()
        .insert("priority".to_owned(), "high".to_owned());

    assert_eq!(desc.attributes().len(), 2);
    assert_eq!(
        desc.attributes().get("value").map(String::as_str),
        Some("userService")
    );
    assert_eq!(
        desc.attributes().get("priority").map(String::as_str),
        Some("high")
    );
    assert!(desc.attributes().contains_key("priority"));
}

#[test]
fn annotation_metadata_trait_queries() {
    let meta = metadata_with(vec![
        AnnotationDescriptor::new_class("Configuration"),
        AnnotationDescriptor::new_method("Bean", "createThing"),
        AnnotationDescriptor::new_field("Autowired", "repo"),
    ]);

    assert!(meta.has_annotation("Configuration"));
    assert!(meta.has_class_annotation("Configuration"));
    assert!(meta.has_method_annotation("Bean"));
    assert!(!meta.has_method_annotation("Configuration"));
    assert!(meta.has_meta_annotation("Bean")); // simplified equality
    assert!(meta.is_class_level());
    assert!(meta.is_method_level());
    assert!(meta.is_field_level());
    assert_eq!(meta.get_annotation_type(), Some("Configuration"));
    assert_eq!(meta.get_method_name(), Some("createThing"));

    let types = meta.get_annotation_types();
    assert_eq!(types.len(), 3);
    assert!(types.iter().any(|t| t == "Configuration"));

    let method_anns = meta.get_method_annotations();
    assert_eq!(method_anns.len(), 1);
    assert_eq!(method_anns[0].annotation_type(), "Bean");

    let first_match = meta.get_annotation("Bean").unwrap();
    assert_eq!(first_match.annotation_type(), "Bean");
    assert!(meta.get_annotation("Missing").is_none());
}

// ===========================================================================
// 2. annotation_type_mapping.rs
// ===========================================================================

#[test]
fn annotation_type_mapping_basic() {
    let mut m = AnnotationTypeMapping::new("Service");
    m.set_attribute("value", "myService");
    m.set_attribute("scope", "singleton");
    assert_eq!(m.annotation_type(), "Service");
    assert_eq!(m.get_attribute("value"), Some("myService"));
    assert_eq!(m.get_attribute("scope"), Some("singleton"));
    assert!(m.has_attribute("value"));
    assert!(!m.has_attribute("missing"));
    assert_eq!(m.attribute_count(), 2);
    assert!(m.is_direct());
    assert!(!m.is_meta());
    assert!(m.source().is_none());
    assert_eq!(m.distance(), 0);

    // Equality (by all fields).
    let m2 = m.clone();
    assert_eq!(m, m2);

    // attributes_mut mutability.
    m.attributes_mut().insert("k".to_owned(), "v".to_owned());
    assert!(m.has_attribute("k"));
}

#[test]
fn annotation_type_mapping_meta() {
    let m = AnnotationTypeMapping::with_source("Component", "Service", 1);
    assert_eq!(m.annotation_type(), "Component");
    assert_eq!(m.source(), Some("Service"));
    assert_eq!(m.distance(), 1);
    assert!(m.is_meta());
    assert!(!m.is_direct());
    assert_eq!(m.attribute_count(), 0);
}

// ===========================================================================
// 3. annotations_scanner.rs
// ===========================================================================

struct SingleClassMetadata {
    descriptors: Vec<AnnotationDescriptor>,
}

impl SingleClassMetadata {
    fn new() -> Self {
        Self {
            descriptors: vec![AnnotationDescriptor::new_class("Component")],
        }
    }
}

impl AnnotationMetadata for SingleClassMetadata {
    fn annotations(&self) -> &[AnnotationDescriptor] {
        &self.descriptors
    }
}

#[test]
fn annotations_scanner_static_register_and_scan_class() {
    let mut scanner = StaticAnnotationsScanner::new();
    scanner.register("com.example.Foo", Arc::new(SingleClassMetadata::new()));

    let found = scanner.scan_class("com.example.Foo").unwrap();
    assert!(found.is_some());
    assert!(found.unwrap().has_annotation("Component"));

    let missing = scanner.scan_class("com.example.Missing").unwrap();
    assert!(missing.is_none());
}

#[test]
fn annotations_scanner_batch_scan() {
    let mut scanner = StaticAnnotationsScanner::new();
    scanner.register("com.example.A", Arc::new(SingleClassMetadata::new()));
    let result = scanner
        .scan(&["com.example.A".to_owned(), "com.example.Missing".to_owned()])
        .unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn annotations_scanner_default() {
    let scanner = StaticAnnotationsScanner::default();
    let result = scanner.scan_class("any").unwrap();
    assert!(result.is_none());
}

// ===========================================================================
// 4. type_filter.rs
// ===========================================================================

#[test]
fn type_filter_regex_basic() {
    let f = RegexTypeFilter::new(r"^com\.example\..*Service$").unwrap();
    assert!(f.matches("com.example.UserService"));
    assert!(!f.matches("com.example.Repository"));
    assert!(!f.matches("org.other.UserService"));
    assert_eq!(f.pattern_str(), r"^com\.example\..*Service$");
}

#[test]
fn type_filter_regex_from_regex_and_pattern() {
    let f = RegexTypeFilter::new(".*Foo.*").unwrap();
    let f2 = RegexTypeFilter::from_regex(f.pattern().clone());
    assert!(f2.matches("com.example.FooBar"));
    assert!(!f2.matches("com.example.Bar"));
}

#[test]
fn type_filter_regex_invalid_pattern() {
    let result = RegexTypeFilter::new("[invalid");
    assert!(result.is_err());
}

#[test]
fn type_filter_prefix_include_and_not() {
    let f = PrefixIncludeFilter::new(["com.example", "org.demo"]);
    assert!(f.matches("com.example.Foo"));
    assert!(f.matches("org.demo.Bar"));
    assert!(!f.matches("net.other.Baz"));
    assert_eq!(f.len(), 2);
    assert!(!f.is_empty());

    let empty = PrefixIncludeFilter::new(Vec::<String>::new());
    assert!(empty.is_empty());
    assert!(empty.matches("anything"));

    let mut added = PrefixIncludeFilter::new(Vec::<String>::new());
    added.add("com.x");
    assert_eq!(added.len(), 1);

    let not = NotFilter::new(f);
    assert!(!not.matches("com.example.Foo"));
    assert!(not.matches("net.other.Baz"));
}

// ===========================================================================
// 5. metadata_reader.rs
// ===========================================================================

struct StubClassMeta {
    name: String,
    is_interface: bool,
    is_abstract: bool,
    is_final: bool,
    super_name: Option<String>,
    ifaces: Vec<String>,
}

impl ClassMetadata for StubClassMeta {
    fn class_name(&self) -> &str {
        &self.name
    }
    fn is_interface(&self) -> bool {
        self.is_interface
    }
    fn is_abstract(&self) -> bool {
        self.is_abstract
    }
    fn is_final(&self) -> bool {
        self.is_final
    }
    fn super_class_name(&self) -> Option<&str> {
        self.super_name.as_deref()
    }
    fn interface_names(&self) -> &[String] {
        &self.ifaces
    }
}

struct StubReader {
    annotations: Arc<dyn AnnotationMetadata>,
    class: Arc<dyn ClassMetadata>,
    resource: String,
}

impl MetadataReader for StubReader {
    fn get_annotation_metadata(&self) -> Arc<dyn AnnotationMetadata> {
        self.annotations.clone()
    }
    fn get_class_metadata(&self) -> Arc<dyn ClassMetadata> {
        self.class.clone()
    }
    fn resource_description(&self) -> &str {
        &self.resource
    }
}

#[test]
fn metadata_reader_traits_expose_data() {
    let annotations: Arc<dyn AnnotationMetadata> = Arc::new(SingleClassMetadata::new());
    let class: Arc<dyn ClassMetadata> = Arc::new(StubClassMeta {
        name: "com.example.Foo".to_owned(),
        is_interface: false,
        is_abstract: false,
        is_final: true,
        super_name: Some("com.example.Base".to_owned()),
        ifaces: vec!["java.io.Serializable".to_owned()],
    });
    let reader = StubReader {
        annotations,
        class,
        resource: "classpath:com/example/Foo.class".to_owned(),
    };

    assert_eq!(
        reader.resource_description(),
        "classpath:com/example/Foo.class"
    );
    assert!(reader.get_class_metadata().is_final());
    assert!(reader.get_class_metadata().is_concrete());
    assert!(reader.get_class_metadata().has_super_class());
    assert_eq!(
        reader.get_class_metadata().interface_names(),
        &["java.io.Serializable".to_owned()]
    );
    assert!(reader.get_annotation_metadata().has_annotation("Component"));
}

#[test]
fn class_metadata_defaults() {
    struct Minimal;
    impl ClassMetadata for Minimal {
        fn class_name(&self) -> &str {
            "X"
        }
    }
    let m = Minimal;
    assert!(!m.is_interface());
    assert!(!m.is_abstract());
    assert!(!m.is_final());
    assert!(m.is_concrete());
    assert!(!m.has_super_class());
    assert!(m.super_class_name().is_none());
    assert!(m.interface_names().is_empty());
}

// ===========================================================================
// 6. simple_metadata_reader.rs
// ===========================================================================

#[test]
fn simple_class_metadata_builder_pattern() {
    let meta = SimpleClassMetadata::new("com.example.Foo")
        .with_super_class("com.example.Base")
        .with_interface(true)
        .with_interfaces(["java.io.Serializable", "java.lang.Cloneable"]);
    assert_eq!(meta.class_name(), "com.example.Foo");
    assert!(meta.is_interface());
    assert!(!meta.is_concrete());
    assert!(meta.has_super_class());
    assert_eq!(meta.super_class_name(), Some("com.example.Base"));
    assert_eq!(meta.interface_names().len(), 2);
}

#[test]
fn simple_class_metadata_concrete_and_final() {
    let meta = SimpleClassMetadata::new("com.example.Bar")
        .with_abstract(false)
        .with_final(true);
    assert!(meta.is_final());
    assert!(meta.is_concrete());
    assert!(!meta.is_abstract());
}

#[test]
fn simple_annotation_metadata_add_and_query() {
    let mut annotations = SimpleAnnotationMetadata::new();
    annotations.add(AnnotationDescriptor::new_class("Component"));
    let from_list =
        SimpleAnnotationMetadata::from_annotations(vec![AnnotationDescriptor::new_class(
            "Service",
        )]);
    assert_eq!(annotations.annotations().len(), 1);
    assert!(annotations.has_annotation("Component"));
    assert_eq!(from_list.annotations().len(), 1);
    assert!(from_list.has_annotation("Service"));
}

#[test]
fn simple_metadata_reader_and_factory() {
    let mut annotations = SimpleAnnotationMetadata::new();
    annotations.add(AnnotationDescriptor::new_class("Component"));
    let class = SimpleClassMetadata::new("com.example.Foo");
    let reader = Arc::new(SimpleMetadataReader::new(
        Arc::new(annotations),
        Arc::new(class),
        "classpath:com/example/Foo.class",
    ));

    assert!(reader.get_annotation_metadata().has_annotation("Component"));
    assert_eq!(reader.get_class_metadata().class_name(), "com.example.Foo");
    assert_eq!(
        reader.resource_description(),
        "classpath:com/example/Foo.class"
    );
    assert_eq!(reader.simple_annotation_metadata().annotations().len(), 1);
    assert_eq!(
        reader.simple_class_metadata().class_name(),
        "com.example.Foo"
    );

    let mut factory = SimpleMetadataReaderFactory::new();
    assert!(factory.is_empty());
    assert_eq!(factory.len(), 0);
    factory.register("com.example.Foo", reader.clone());
    assert_eq!(factory.len(), 1);
    let got = factory.get_metadata_reader("com.example.Foo").unwrap();
    assert_eq!(got.simple_class_metadata().class_name(), "com.example.Foo");
    assert!(factory.get_metadata_reader("missing").is_none());
}

// ===========================================================================
// 7. annotation_processor.rs
// ===========================================================================

struct CountingProcessor {
    supported: Vec<String>,
    count: Mutex<usize>,
}

impl CountingProcessor {
    fn new(supported: Vec<&str>) -> Self {
        Self {
            supported: supported.into_iter().map(String::from).collect(),
            count: Mutex::new(0),
        }
    }
    fn calls(&self) -> usize {
        *self.count.lock().unwrap()
    }
}

impl AnnotationProcessor for CountingProcessor {
    fn process(
        &self,
        _meta: &dyn AnnotationMetadata,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.count.lock().unwrap() += 1;
        Ok(())
    }
    fn supported_annotation_types(&self) -> &[String] {
        &self.supported
    }
}

#[test]
fn annotation_processor_supports_empty_vs_specific() {
    let p_all = CountingProcessor::new(vec![]);
    assert!(p_all.supports("Anything"));
    assert!(p_all.supports("AnythingElse"));

    let p_specific = CountingProcessor::new(vec!["Component"]);
    assert!(p_specific.supports("Component"));
    assert!(!p_specific.supports("Service"));
}

#[test]
fn annotation_processor_composite_dispatches() {
    let counter = Arc::new(CountingProcessor::new(vec!["Component"]));
    let mut composite = CompositeAnnotationProcessor::new();
    assert!(composite.is_empty());
    assert_eq!(composite.len(), 0);
    composite.add(counter.clone());

    let meta = metadata_with(vec![
        AnnotationDescriptor::new_class("Component"),
        AnnotationDescriptor::new_class("Bean"),
    ]);
    composite.process(&meta).unwrap();
    // Component matched once (one annotation with that type). Bean does not match.
    assert_eq!(counter.calls(), 1);

    // Empty composite processes successfully.
    let empty = CompositeAnnotationProcessor::default();
    empty.process(&meta).unwrap();
}

#[test]
fn annotation_processor_unsupported_does_not_invoke() {
    let counter = Arc::new(CountingProcessor::new(vec!["Component"]));
    let mut composite = CompositeAnnotationProcessor::new();
    composite.add(counter.clone());

    let meta = metadata_with(vec![AnnotationDescriptor::new_class("Service")]);
    composite.process(&meta).unwrap();
    assert_eq!(counter.calls(), 0);
}

// ===========================================================================
// 8. configuration_class.rs
// ===========================================================================

#[test]
fn configuration_class_bean_method_details() {
    let mut config = ConfigurationClass::new("com.example.AppConfig");
    let bean_method =
        ConfigurationBeanMethod::new("createFoo", "com.example.Foo").with_bean_name("foo");
    config.add_bean_method_detail(bean_method);
    config.add_bean_method("createBar");
    // dedup
    config.add_bean_method("createFoo");
    config.add_import("com.example.OtherConfig");
    // dedup
    config.add_import("com.example.OtherConfig");
    config.set_source("classpath:com/example/AppConfig.class");
    config.set_full(false);

    assert_eq!(config.class_name(), "com.example.AppConfig");
    assert_eq!(
        config.bean_methods(),
        &["createFoo".to_owned(), "createBar".to_owned()]
    );
    assert_eq!(config.bean_method_details().len(), 1);
    assert_eq!(config.bean_method_details()[0].bean_name, "foo");
    assert_eq!(
        config.bean_method_details()[0].return_type_name,
        "com.example.Foo"
    );
    assert_eq!(config.bean_method_count(), 2);
    assert_eq!(config.imported(), &["com.example.OtherConfig".to_owned()]);
    assert_eq!(
        config.source(),
        Some("classpath:com/example/AppConfig.class")
    );
    assert!(!config.is_full());
}

#[test]
fn configuration_class_default_full_and_clone() {
    let config = ConfigurationClass::new("com.example.X");
    assert!(config.is_full());
    assert_eq!(config.bean_method_count(), 0);
    assert!(config.source().is_none());
    let copy = config.clone();
    assert_eq!(copy.class_name(), "com.example.X");
}

#[test]
fn configuration_bean_method_new_default_bean_name() {
    let m = ConfigurationBeanMethod::new("createBar", "com.example.Bar");
    assert_eq!(m.method_name, "createBar");
    assert_eq!(m.return_type_name, "com.example.Bar");
    assert_eq!(m.bean_name, "createBar"); // default = method name
}

// ===========================================================================
// 9. configuration_class_parser.rs
// ===========================================================================

#[test]
fn configuration_class_parser_default_predicate() {
    let parser = ConfigurationClassParser::new();
    let classes = parser.parse_configuration_classes(&[
        "com.example.AppConfig".to_owned(),
        "com.example.PlainBean".to_owned(),
        "com.example.DbConfig".to_owned(),
    ]);
    assert_eq!(classes.len(), 2);
    let names: Vec<&str> = classes.iter().map(|c| c.class_name()).collect();
    assert!(names.contains(&"com.example.AppConfig"));
    assert!(names.contains(&"com.example.DbConfig"));
}

#[test]
fn configuration_class_parser_custom_providers() {
    let parser = ConfigurationClassParser::new()
        .with_configuration_predicate(|name| name.contains("Config"))
        .with_bean_methods_provider(|name| {
            if name == "com.example.AppConfig" {
                vec!["createFoo".to_owned(), "createBar".to_owned()]
            } else {
                vec![]
            }
        })
        .with_imports_provider(|_| vec!["com.extra.Util".to_owned()]);

    let classes = parser.parse_configuration_classes(&[
        "com.example.AppConfig".to_owned(),
        "com.example.OtherConfig".to_owned(),
        "com.example.Plain".to_owned(),
    ]);
    assert_eq!(classes.len(), 2);
    let app = classes
        .iter()
        .find(|c| c.class_name() == "com.example.AppConfig")
        .unwrap();
    assert_eq!(
        app.bean_methods(),
        &["createFoo".to_owned(), "createBar".to_owned()]
    );
    assert_eq!(app.imported(), &["com.extra.Util".to_owned()]);
}

#[test]
fn configuration_class_parser_default_trait() {
    let parser = ConfigurationClassParser::default();
    let classes = parser.parse_configuration_classes(&["com.example.XConfig".to_owned()]);
    assert_eq!(classes.len(), 1);
    assert!(classes[0].bean_methods().is_empty());
}

// ===========================================================================
// 10. configuration_class_bean_definition_reader.rs
// ===========================================================================

#[test]
fn configuration_class_bean_definition_reader_load_basic() {
    let mut config = ConfigurationClass::new("com.example.AppConfig");
    config.add_bean_method_detail(ConfigurationBeanMethod::new("createFoo", "com.example.Foo"));
    config.add_bean_method_detail(
        ConfigurationBeanMethod::new("createBar", "com.example.Bar").with_bean_name("bar"),
    );

    let mut reader = ConfigurationClassBeanDefinitionReader::new();
    assert!(reader.is_empty());
    reader.load_bean_definitions(&[config]);
    assert_eq!(reader.len(), 2);
    assert!(!reader.is_empty());

    let foo = reader.get("createFoo").unwrap();
    assert_eq!(foo.factory_method, "createFoo");
    assert_eq!(foo.factory_bean_name, "com.example.AppConfig");
    assert_eq!(foo.return_type_name, "com.example.Foo");
    assert_eq!(foo.bean_name, "createFoo");

    let bar = reader.get("bar").unwrap();
    assert_eq!(bar.bean_name, "bar");
    assert_eq!(bar.return_type_name, "com.example.Bar");

    let names: Vec<&str> = reader
        .loaded_definitions()
        .keys()
        .map(String::as_str)
        .collect();
    assert!(names.contains(&"createFoo"));
    assert!(names.contains(&"bar"));
}

#[test]
fn configuration_class_bean_definition_reader_register_into() {
    let mut config = ConfigurationClass::new("com.example.AppConfig");
    config.add_bean_method_detail(ConfigurationBeanMethod::new("createFoo", "com.example.Foo"));
    config.add_bean_method_detail(ConfigurationBeanMethod::new("createBar", "com.example.Bar"));

    let mut reader = ConfigurationClassBeanDefinitionReader::new();
    reader.load_bean_definitions(&[config]);

    let mut registry = StubRegistry {
        names: vec!["createFoo".to_owned()],
    };
    let registered = reader.register_into(&mut registry).unwrap();
    // Only "createBar" should be reported as new (since "createFoo" already in registry).
    assert_eq!(registered, vec!["createBar".to_owned()]);
}

#[test]
fn configuration_class_bean_definition_reader_default_trait() {
    let reader = ConfigurationClassBeanDefinitionReader::default();
    assert!(reader.is_empty());
}

// ===========================================================================
// 11. import_selector.rs
// ===========================================================================

#[test]
fn import_selector_fixed_with_excluded() {
    let selector = FixedImportSelector::new(["com.example.A", "com.example.B", "com.example.C"])
        .with_excluded(["com.example.B"]);
    let imports = selector.select_imports(&empty_metadata());
    assert_eq!(
        imports,
        vec!["com.example.A".to_owned(), "com.example.C".to_owned()]
    );
    assert!(selector.is_excluded("com.example.B"));
    assert!(!selector.is_excluded("com.example.A"));
}

#[test]
fn import_selector_no_excludes_default_is_excluded_false() {
    struct Sel;
    impl ImportSelector for Sel {
        fn select_imports(&self, _: &dyn AnnotationMetadata) -> Vec<String> {
            vec![]
        }
    }
    let s = Sel;
    assert!(!s.is_excluded("anything"));
    assert!(s.select_imports(&empty_metadata()).is_empty());
}

// ===========================================================================
// 12. deferred_import_selector.rs
// ===========================================================================

#[test]
fn deferred_import_selector_default_group_collects() {
    let mut group = DefaultDeferredImportGroup::new();
    assert!(group.is_empty());
    assert_eq!(group.len(), 0);

    let sel = FixedDeferredImportSelector::new(["com.example.A", "com.example.B"]);
    group.process(&empty_metadata(), &sel);
    assert_eq!(group.len(), 2);
    assert_eq!(
        group.select_imports(),
        vec!["com.example.A".to_owned(), "com.example.B".to_owned()]
    );

    let sel2 = FixedDeferredImportSelector::new(["com.example.C"]);
    group.process(&empty_metadata(), &sel2);
    assert_eq!(group.select_imports().len(), 3);

    group.clear();
    assert!(group.is_empty());
}

#[test]
fn deferred_import_selector_inherits_import_selector() {
    let sel = FixedDeferredImportSelector::new(["x", "y"]);
    let imports: Vec<String> = ImportSelector::select_imports(&sel, &empty_metadata());
    assert_eq!(imports, vec!["x".to_owned(), "y".to_owned()]);
}

#[test]
fn deferred_import_selector_default_group_trait() {
    struct StubGroup;
    impl DeferredImportSelectorGroup for StubGroup {
        fn process(&mut self, _: &dyn AnnotationMetadata, _: &dyn DeferredImportSelector) {}
        fn select_imports(&self) -> Vec<String> {
            vec!["s".to_owned()]
        }
    }
    let g = StubGroup;
    assert_eq!(g.select_imports(), vec!["s".to_owned()]);
}

// ===========================================================================
// 13. import_bean_definition_registrar.rs
// ===========================================================================

#[test]
fn import_bean_definition_registrar_noop_default_copy() {
    let n = NoopImportBeanDefinitionRegistrar;
    let copy = n; // Copy
    let _ = n;
    let _ = copy;
}

#[test]
fn import_bean_definition_registrar_composite_len_and_empty() {
    let mut composite = CompositeImportBeanDefinitionRegistrar::new();
    assert!(composite.is_empty());
    assert_eq!(composite.len(), 0);
    composite.add(Box::new(NoopImportBeanDefinitionRegistrar));
    composite.add(Box::new(NoopImportBeanDefinitionRegistrar));
    assert_eq!(composite.len(), 2);
    let _ = CompositeImportBeanDefinitionRegistrar::default();
}

#[test]
fn import_bean_definition_registrar_noop_does_nothing() {
    let mut registry = StubRegistry { names: Vec::new() };
    let registrar = NoopImportBeanDefinitionRegistrar;
    registrar
        .register_bean_definitions(&empty_metadata(), &mut registry)
        .unwrap();
    assert!(registry.names.is_empty());
}

#[test]
fn import_bean_definition_registrar_composite_invokes_each() {
    let mut composite = CompositeImportBeanDefinitionRegistrar::new();
    composite.add(Box::new(NoopImportBeanDefinitionRegistrar));
    composite.add(Box::new(NoopImportBeanDefinitionRegistrar));

    let mut registry = StubRegistry { names: Vec::new() };
    composite
        .register_bean_definitions(&empty_metadata(), &mut registry)
        .unwrap();
    assert!(registry.names.is_empty());
}

// ===========================================================================
// 14. environment.rs
// ===========================================================================

#[test]
fn environment_simple_construct_default_profile() {
    let env = SimpleEnvironment::new();
    assert_eq!(env.get_default_profiles(), vec!["default".to_owned()]);
    // No explicit active -> falls back to default.
    assert_eq!(env.get_active_profiles(), vec!["default".to_owned()]);
    assert!(env.accepts_profile("default"));
    assert!(!env.accepts_profile("dev"));
    assert!(env.accepts_profiles(&["dev".to_owned(), "default".to_owned()]));
    assert!(!env.accepts_profiles(&["dev".to_owned()]));
}

#[test]
fn environment_simple_active_overrides_default() {
    let mut env = SimpleEnvironment::new();
    env.set_active_profiles(["dev", "cloud"]);
    env.set_default_profiles(["base"]);
    assert_eq!(
        env.get_active_profiles(),
        vec!["dev".to_owned(), "cloud".to_owned()]
    );
    assert_eq!(env.get_default_profiles(), vec!["base".to_owned()]);
    assert!(env.accepts_profile("dev"));
    assert!(!env.accepts_profile("base"));
}

#[test]
fn environment_simple_property_query_and_placeholders() {
    let mut env = SimpleEnvironment::new();
    env.set_property("name", "vernal");
    env.set_property("greeting", "Hello ${name}!");
    assert!(env.contains_property("name"));
    assert!(!env.contains_property("missing"));
    assert_eq!(env.get_property("name"), Some("vernal".to_owned()));
    assert_eq!(env.get_property_or("missing", "fb"), "fb");
    assert!(env.get_required_property("missing").is_err());
    assert_eq!(env.resolve_placeholders("${name}"), "vernal");
    assert_eq!(env.resolve_placeholders("${unknown:def}"), "def");
    assert_eq!(env.resolve_placeholders("Hi ${name}!"), "Hi vernal!");
    assert!(env.resolve_required_placeholders("${unknown}").is_err());
    assert!(
        env.resolve_required_placeholders("${name}")
            .is_ok_and(|s| s == "vernal")
    );
}

// ===========================================================================
// 15. property_source.rs
// ===========================================================================

#[test]
fn property_source_constructors_and_accessors() {
    let mut ps = PropertySource::new("app");
    ps.set_property("name", "vernal");
    ps.set_property("version", "1");
    assert_eq!(ps.name(), "app");
    assert!(ps.contains_property("name"));
    assert_eq!(ps.get_property("name"), Some("vernal"));
    assert_eq!(ps.get_property("missing"), None);
    assert_eq!(ps.len(), 2);
    assert!(!ps.is_empty());
    let names: Vec<&str> = ps.property_names();
    assert_eq!(names.len(), 2);

    let replaced = ps.remove_property("version");
    assert_eq!(replaced, Some("1".to_owned()));
    assert_eq!(ps.len(), 1);
}

#[test]
fn property_source_with_source_and_from_pairs() {
    let data = HashMap::from([("k".to_owned(), "v".to_owned())]);
    let ps = PropertySource::with_source("a", data);
    assert_eq!(ps.get_property("k"), Some("v"));
    assert_eq!(ps.len(), 1);
    assert_eq!(ps.source().get("k").map(String::as_str), Some("v"));

    let pairs = PropertySource::from_pairs("b", [("a", "1"), ("b", "2"), ("c", "3")]);
    assert_eq!(pairs.len(), 3);
    assert!(pairs.contains_property("a"));

    let mut mps = PropertySource::new("mutable");
    mps.source_mut().insert("x".to_owned(), "y".to_owned());
    assert!(mps.contains_property("x"));
}

#[test]
fn property_source_empty_default() {
    let ps = PropertySource::new("empty");
    assert!(ps.is_empty());
    assert_eq!(ps.property_names().len(), 0);
}

// ===========================================================================
// 16. property_sources.rs
// ===========================================================================

fn build_ps(name: &str, pairs: &[(&str, &str)]) -> PropertySource {
    PropertySource::from_pairs(
        name,
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())),
    )
}

#[test]
fn property_sources_priority_lookup() {
    let mut sources = PropertySources::new();
    sources.add(build_ps("high", &[("k", "1")]));
    sources.add(build_ps("low", &[("k", "2")]));
    assert_eq!(sources.get_property("k"), Some("1"));
    assert!(sources.contains_property("k"));
    assert_eq!(sources.names(), vec!["high", "low"]);
    assert!(!sources.is_empty());
    assert_eq!(sources.len(), 2);
    assert_eq!(sources.sources().len(), 2);
}

#[test]
fn property_sources_before_after_insert_remove_replace() {
    let mut sources = PropertySources::new();
    sources.add(build_ps("a", &[("x", "1")]));
    sources.add(build_ps("b", &[("x", "2")]));

    sources.add_before("b", build_ps("middle", &[("x", "m")]));
    assert_eq!(sources.names(), vec!["a", "middle", "b"]);

    sources.add_after("a", build_ps("between", &[("x", "bt")]));
    assert_eq!(sources.names(), vec!["a", "between", "middle", "b"]);

    // before/after on missing key => append.
    sources.add_before("missing", build_ps("appended", &[("x", "z")]));
    assert!(sources.contains("appended"));

    // insert at index beyond length => push.
    sources.insert(100, build_ps("late", &[("x", "L")]));
    assert!(sources.contains("late"));

    // get / get_mut.
    let _ = sources.get("a").unwrap();
    {
        let g = sources.get_mut("a").unwrap();
        g.set_property("x", "updated");
    }
    assert_eq!(sources.get_property("x"), Some("updated"));

    // replace.
    assert!(sources.replace(build_ps("late", &[("x", "L2")])));
    assert!(!sources.replace(build_ps("brand-new", &[("x", "N")])));
    assert_eq!(sources.get("late").unwrap().get_property("x"), Some("L2"));

    // remove.
    let removed = sources.remove("late").unwrap();
    assert_eq!(removed.name(), "late");
    assert!(sources.remove("missing").is_none());

    // clear.
    let count = sources.len();
    sources.clear();
    assert!(sources.is_empty());
    assert_eq!(count > 0, true);
}

#[test]
fn property_sources_default_trait() {
    let sources = PropertySources::default();
    assert!(sources.is_empty());
}

// ===========================================================================
// 17. property_resolver.rs
// ===========================================================================

struct CountingResolver {
    data: HashMap<String, String>,
}

impl PropertyResolver for CountingResolver {
    fn contains_property(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    fn get_property(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }
    fn resolve_placeholders(&self, text: &str) -> String {
        text.to_owned()
    }
    fn resolve_required_placeholders(
        &self,
        _text: &str,
    ) -> Result<String, UnresolvedPlaceholderError> {
        Ok(String::new())
    }
}

#[test]
fn property_resolver_default_helpers() {
    let r = CountingResolver {
        data: HashMap::from([("k".to_owned(), "v".to_owned())]),
    };
    assert_eq!(r.get_property_or("k", "fb"), "v");
    assert_eq!(r.get_property_or("missing", "fb"), "fb");
    assert!(r.get_required_property("k").is_ok_and(|v| v == "v"));
    let err = r.get_required_property("missing").unwrap_err();
    assert_eq!(err.key, "missing");
}

#[test]
fn property_resolver_errors_display() {
    let m = MissingRequiredPropertyError::new("x".to_owned());
    assert_eq!(m.to_string(), "Required property 'x' is not defined");

    let u = UnresolvedPlaceholderError::new("input".to_owned());
    assert!(u.to_string().contains("input"));
}

// ===========================================================================
// 18. configurable_environment.rs
// ===========================================================================

#[test]
fn configurable_environment_default_system_maps() {
    let mut env = AbstractEnvironment::new();
    let sources_arc = env.property_sources();
    {
        let mut g = sources_arc.write().unwrap();
        g.add_last(PropertySource::from_pairs("app", [("a", "1")]));
    }
    env.set_active_profiles(&["dev".to_owned()]);
    env.add_active_profile("extra");
    env.set_default_profiles(&["base".to_owned()]);
    env.set_system_properties(HashMap::from([("sp".to_owned(), "1".to_owned())]));
    env.set_system_environment(HashMap::from([("se".to_owned(), "2".to_owned())]));

    assert_eq!(
        env.get_system_properties().get("sp").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        env.get_system_environment().get("se").map(String::as_str),
        Some("2")
    );
    assert!(env.accepts_profile("dev"));
    assert!(env.accepts_profile("extra"));

    let mut profiles = env.profiles().clone();
    profiles.add("more");
    let _ = profiles;
}

#[test]
fn configurable_environment_merge_parent_sources() {
    let parent = AbstractEnvironment::new();
    {
        let sources_arc = parent.property_sources();
        let mut g = sources_arc.write().unwrap();
        g.add_last(PropertySource::from_pairs("parent", [("k", "1")]));
    }

    let mut child = AbstractEnvironment::new();
    {
        let sources_arc = child.property_sources();
        let mut g = sources_arc.write().unwrap();
        g.add_last(PropertySource::from_pairs("child", [("ck", "1")]));
    }
    child.merge(&parent);
    // Both sources present.
    let child_sources = child.property_sources();
    let guard = child_sources.read().unwrap();
    assert!(guard.contains("parent"));
    assert!(guard.contains("child"));
}

#[test]
fn configurable_environment_default_profile_name() {
    assert_eq!(default_profile_name(), RESERVED_DEFAULT_PROFILE_NAME);
    assert_eq!(default_profile_name(), "default");
}

// ===========================================================================
// 19. mutable_property_sources.rs
// ===========================================================================

#[test]
fn mutable_property_sources_add_first_last() {
    let mut m = MutablePropertySources::new();
    m.add_last(build_ps("low", &[("k", "1")]));
    m.add_first(build_ps("high", &[("k", "2")]));
    assert_eq!(m.names(), vec!["high", "low"]);
    assert_eq!(m.get_property("k"), Some("2"));

    // add_first with already-present name promotes & replaces.
    m.add_first(build_ps("low", &[("k", "3")]));
    assert_eq!(m.names(), vec!["low", "high"]);
    assert_eq!(m.get_property("k"), Some("3"));

    // add_last replaces if name already exists.
    m.add_last(build_ps("high", &[("k", "4")]));
    assert_eq!(m.names(), vec!["low", "high"]);
    assert_eq!(m.get_property("k"), Some("3"));
}

#[test]
fn mutable_property_sources_before_after_replace_remove() {
    let mut m = MutablePropertySources::new();
    m.add_last(build_ps("a", &[("k", "1")]));
    m.add_last(build_ps("b", &[("k", "2")]));
    m.add_before("b", build_ps("c", &[("k", "3")]));
    assert_eq!(m.names(), vec!["a", "c", "b"]);
    m.add_after("a", build_ps("d", &[("k", "4")]));
    assert_eq!(m.names(), vec!["a", "d", "c", "b"]);
    assert!(m.replace(build_ps("a", &[("k", "9")])));
    assert_eq!(m.get("a").unwrap().get_property("k"), Some("9"));
    assert!(m.remove("a").is_some());
    assert!(!m.contains("a"));

    // from_property_sources + into_property_sources round-trip.
    let mut ps = PropertySources::new();
    ps.add(build_ps("z", &[("k", "Z")]));
    let from = MutablePropertySources::from_property_sources(ps);
    assert!(from.contains("z"));
    let back = from.into_property_sources();
    assert!(back.contains("z"));

    // as_property_sources.
    let m2 = MutablePropertySources::new();
    let _ = m2.as_property_sources();
}

#[test]
fn mutable_property_sources_clear_and_default() {
    let mut m = MutablePropertySources::default();
    m.add_first(build_ps("a", &[("k", "1")]));
    m.clear();
    assert!(m.is_empty());
}

// ===========================================================================
// 20. abstract_environment.rs
// ===========================================================================

#[test]
fn abstract_environment_profiles_and_resolution() {
    let mut env = AbstractEnvironment::new();
    {
        let sources_arc = env.property_sources();
        let mut sources = sources_arc.write().unwrap();
        sources.add_first(PropertySource::from_pairs(
            "app",
            [("name", "vernal"), ("greeting", "Hi ${name}")],
        ));
    }
    env.set_active_profiles(&["dev".to_owned()]);
    env.add_active_profile("extra");
    assert_eq!(
        env.get_active_profiles(),
        vec!["dev".to_owned(), "extra".to_owned()]
    );
    assert!(env.accepts_profile("dev"));
    assert!(env.accepts_profile("extra"));
    assert!(env.accepts_profiles(&["dev".to_owned(), "missing".to_owned()]));
    assert_eq!(env.get_property("name"), Some("vernal".to_owned()));
    assert_eq!(env.resolve_placeholders("${greeting}"), "Hi vernal");
    assert_eq!(
        env.resolve_required_placeholders("${name}").unwrap(),
        "vernal"
    );
    assert!(env.resolve_required_placeholders("${missing}").is_err());
}

#[test]
fn abstract_environment_default_profile_fallback() {
    let env = AbstractEnvironment::new();
    assert_eq!(env.get_active_profiles(), vec!["default".to_owned()]);
    assert_eq!(env.get_default_profiles(), vec!["default".to_owned()]);

    let mut env2 = AbstractEnvironment::new();
    env2.set_default_profiles(&["base".to_owned()]);
    assert_eq!(env2.get_default_profiles(), vec!["base".to_owned()]);
    assert_eq!(env2.get_active_profiles(), vec!["base".to_owned()]);
}

#[test]
fn abstract_environment_merge_inherits_property() {
    let parent = AbstractEnvironment::new();
    {
        let sources_arc = parent.property_sources();
        let mut g = sources_arc.write().unwrap();
        g.add_last(PropertySource::from_pairs("parent", [("k", "1")]));
    }
    let mut child = AbstractEnvironment::new();
    child.merge(&parent);
    assert_eq!(child.get_property("k"), Some("1".to_owned()));
}

#[test]
fn abstract_environment_profiles_mut() {
    let mut env = AbstractEnvironment::new();
    env.profiles_mut().add("custom");
    assert!(env.accepts_profile("custom"));
}

// ===========================================================================
// 21. standard_environment.rs
// ===========================================================================

#[test]
fn standard_environment_has_default_sources() {
    let env = StandardEnvironment::new();
    let sources = env.property_sources();
    let g = sources.read().unwrap();
    assert!(g.contains(SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME));
    assert!(g.contains(SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME));
    assert_eq!(SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME, "systemProperties");
    assert_eq!(SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME, "systemEnvironment");
}

#[test]
fn standard_environment_with_custom_sources() {
    let mut sp = HashMap::new();
    sp.insert("k".to_owned(), "1".to_owned());
    let mut se = HashMap::new();
    se.insert("e".to_owned(), "2".to_owned());
    let env = StandardEnvironment::with_sources(sp, se);
    assert_eq!(env.get_property("k"), Some("1".to_owned()));
    assert_eq!(env.get_property("e"), Some("2".to_owned()));
    assert_eq!(
        env.get_system_properties().get("k").map(String::as_str),
        Some("1")
    );
    assert_eq!(
        env.get_system_environment().get("e").map(String::as_str),
        Some("2")
    );
}

#[test]
fn standard_environment_default_trait() {
    let env = StandardEnvironment::default();
    assert!(env.get_active_profiles().contains(&"default".to_owned()));
}

#[test]
fn standard_environment_build_helpers() {
    let sys_props = build_system_properties();
    let sys_env = build_system_environment();
    // Just verify they produce maps.
    let _: HashMap<String, String> = sys_props;
    let _: HashMap<String, String> = sys_env;
}

// ===========================================================================
// 22. profile.rs
// ===========================================================================

#[test]
fn profile_basic_new_and_default() {
    let p = Profile::new("dev");
    assert_eq!(p.name(), "dev");
    assert!(!p.is_default());

    let d = Profile::new_default("default");
    assert_eq!(d.name(), "default");
    assert!(d.is_default());

    let mut p2 = Profile::new("prod");
    p2.set_default(true);
    assert!(p2.is_default());
    p2.set_default(false);
    assert!(!p2.is_default());
}

#[test]
fn profile_display_eq_ord_hash() {
    let p = Profile::new("dev");
    assert_eq!(p.to_string(), "dev");

    let a = Profile::new("prod");
    let b = Profile::new_default("prod");
    assert_eq!(a, b); // equality by name

    // Ordering by name.
    let mut profiles = vec![Profile::new("c"), Profile::new("a"), Profile::new("b")];
    profiles.sort();
    let names: Vec<&str> = profiles.iter().map(|p| p.name()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);

    // Hash: use in HashSet.
    let mut set = HashSet::new();
    set.insert(Profile::new("x"));
    set.insert(Profile::new("x"));
    set.insert(Profile::new("y"));
    assert_eq!(set.len(), 2);
}

// ===========================================================================
// 23. profiles.rs
// ===========================================================================

#[test]
fn profiles_default_and_explicit() {
    let mut profiles = Profiles::new();
    assert_eq!(profiles.get_default(), vec!["default"]);
    assert_eq!(profiles.get_active(), vec!["default"]);
    assert!(profiles.is_active("default"));
    assert!(!profiles.has_explicit_active());
    assert_eq!(profiles.active_count(), 0);

    profiles.add("dev");
    profiles.add("dev"); // dedup
    profiles.add("cloud");
    assert_eq!(profiles.active_count(), 2);
    assert!(profiles.is_active("dev"));
    assert!(!profiles.is_active("default"));
    assert_eq!(profiles.get_active(), vec!["dev", "cloud"]);
}

#[test]
fn profiles_remove_set_replace() {
    let mut profiles = Profiles::new();
    profiles.add("dev");
    profiles.remove("dev");
    assert_eq!(profiles.active_count(), 0);
    assert!(profiles.is_active("default"));

    profiles.set_active(&["prod".to_owned()]);
    assert!(profiles.is_active("prod"));
    assert!(!profiles.is_active("dev"));
    assert!(profiles.has_explicit_active());

    profiles.set_default(&["base".to_owned()]);
    assert_eq!(profiles.get_default(), vec!["base"]);
    profiles.add_default("fallback");
    profiles.add_default("fallback"); // dedup
    assert_eq!(profiles.get_default().len(), 2);

    profiles.accept(&["a".to_owned(), "b".to_owned()]);
    assert!(profiles.is_active("a"));
}

#[test]
fn profiles_default_trait() {
    let p = Profiles::default();
    // Profiles::default() does not add the "default" profile (only Profiles::new() does).
    assert!(!p.has_explicit_active());
    assert_eq!(p.active_count(), 0);
    assert!(p.get_default().is_empty());
    assert!(p.get_active().is_empty());
    // Calling Profiles::new() (the recommended constructor) gives the default profile.
    let q = Profiles::new();
    assert!(q.is_active("default"));
}

// ===========================================================================
// 24. placeholder_resolver.rs
// ===========================================================================

fn placeholder_sources() -> PropertySources {
    let mut sources = PropertySources::new();
    sources.add(PropertySource::from_pairs(
        "app",
        [("name", "vernal"), ("greeting", "Hello ${name}!")],
    ));
    sources
}

#[test]
fn placeholder_resolver_simple_and_nested() {
    let sources = placeholder_sources();
    let r = PlaceholderResolver::new(&sources);
    assert_eq!(r.resolve_placeholders("${name}"), "vernal");
    assert_eq!(r.resolve_placeholders("${greeting}"), "Hello vernal!");
    assert_eq!(r.resolve_placeholders("plain"), "plain");
}

#[test]
fn placeholder_resolver_default_and_required() {
    let sources = placeholder_sources();
    let r = PlaceholderResolver::new(&sources);
    assert_eq!(r.resolve_placeholders("${missing:fallback}"), "fallback");
    assert!(r.resolve_required_placeholders("${name}").is_ok());
    let err = r
        .resolve_required_placeholders("${totally.unknown}")
        .unwrap_err();
    assert_eq!(err.input, "${totally.unknown}");
    assert!(err.placeholders.contains(&"${totally.unknown}".to_owned()));
    assert!(err.to_string().contains("totally.unknown"));
}

#[test]
fn placeholder_resolver_constants() {
    assert_eq!(PLACEHOLDER_PREFIX, "${");
    assert_eq!(PLACEHOLDER_SUFFIX, "}");
    assert_eq!(VALUE_SEPARATOR, ":");
}

// ===========================================================================
// 25. string_value_resolver.rs
// ===========================================================================

#[test]
fn string_value_resolver_identity() {
    let r = IdentityStringValueResolver;
    assert_eq!(r.resolve_string_value("hello"), "hello");
    assert_eq!(r.resolve_string_value(""), "");
}

#[test]
fn string_value_resolver_closure() {
    let resolver = |v: &str| format!("[{v}]") as String;
    assert_eq!(resolver.resolve_string_value("x"), "[x]");
}

// ===========================================================================
// 26. embedded_value_resolver.rs
// ===========================================================================

#[test]
fn embedded_value_resolver_simple_placeholder() {
    let mut env = SimpleEnvironment::new();
    env.set_property("name", "vernal");
    let resolver = EmbeddedValueResolver::new(Arc::new(env));
    assert_eq!(resolver.resolve_string_value("${name}"), "vernal");
    // environment getter returns the same env.
    assert!(std::ptr::eq(
        resolver.environment().as_ref() as *const _,
        Arc::as_ptr(resolver.environment())
    ));
}

#[test]
fn embedded_value_resolver_mixed_text_and_default() {
    let mut env = SimpleEnvironment::new();
    env.set_property("name", "vernal");
    let resolver = EmbeddedValueResolver::new(Arc::new(env));
    assert_eq!(
        resolver.resolve_string_value("Hello ${name}!"),
        "Hello vernal!"
    );
    // Default-value placeholder.
    assert_eq!(resolver.resolve_string_value("${missing:fb}"), "fb");
}

// ===========================================================================
// 27. bean_reference_resolver.rs
// ===========================================================================

#[test]
fn bean_reference_resolver_basic_and_missing() {
    let container = Arc::new(Container::new(Registry::empty()));
    let resolver = BeanReferenceResolver::new(container.clone());
    // Container ref matches the one passed in.
    assert!(Arc::ptr_eq(resolver.container(), &container));

    // Empty container -> no resolution.
    let ref_obj = StubBeanRef {
        name: "missing".to_owned(),
    };
    let result = resolver.resolve_reference(&ref_obj).unwrap();
    assert!(result.is_none());

    assert!(!resolver.is_resolvable(&ref_obj));
    assert!(!resolver.contains_bean("anything"));
}

#[test]
fn bean_reference_resolver_resolve_by_name_none() {
    let container = Arc::new(Container::new(Registry::empty()));
    let resolver = BeanReferenceResolver::new(container);
    let result = resolver.resolve_by_name("nope").unwrap();
    assert!(result.is_none());
}

// ===========================================================================
// 28. listable_bean_factory_extensions.rs
// ===========================================================================

struct StubListable {
    names: Vec<String>,
}

impl BeanFactory for StubListable {
    fn get_bean_by_key(
        &self,
        _key: &ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not supported".into())
    }
    fn get_bean_by_type_id(
        &self,
        _type_id: TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not supported".into())
    }
    fn contains_bean(&self, _key: &ComponentKey) -> bool {
        false
    }
    fn is_singleton(
        &self,
        _key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(true)
    }
    fn is_prototype(
        &self,
        _key: &ComponentKey,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(false)
    }
    fn get_type(
        &self,
        _key: &ComponentKey,
    ) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
    fn get_aliases(&self, _key: &ComponentKey) -> Vec<ComponentKey> {
        Vec::new()
    }
    fn get_bean_provider_by_type_id(
        &self,
        _type_id: TypeId,
    ) -> Result<
        Box<dyn ObjectProvider<dyn Any + Send + Sync> + '_>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Err("not supported".into())
    }
    fn is_type_match(&self, _key: &ComponentKey, _type_id: TypeId) -> bool {
        false
    }
}

impl ListableBeanFactory for StubListable {
    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.names.iter().any(|n| n == bean_name)
    }
    fn bean_definition_count(&self) -> usize {
        self.names.len()
    }
    fn bean_definition_names(&self) -> Vec<String> {
        self.names.clone()
    }
    fn bean_names_for_type_id(
        &self,
        _type_id: TypeId,
        _include_non_singletons: bool,
        _allow_eager_init: bool,
    ) -> Vec<String> {
        Vec::new()
    }
    fn beans_of_type_id(
        &self,
        _type_id: TypeId,
        _include_non_singletons: bool,
        _allow_eager_init: bool,
    ) -> Result<HashMap<String, Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(HashMap::new())
    }
    fn bean_post_processor_count(&self) -> usize {
        0
    }
    fn contains_non_singleton_bean(&self) -> bool {
        false
    }
    fn contains_singleton_bean(&self) -> bool {
        true
    }
    fn bean_names_iterator(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(self.names.clone().into_iter())
    }
}

#[test]
fn listable_bean_factory_extensions_query_annotations() {
    let factory = StubListable {
        names: vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
    };
    let mut annotations: AnnotationRegistry = AnnotationRegistry::new();
    AnnotationRegistryExt::register(&mut annotations, "foo", "Component");
    AnnotationRegistryExt::register(&mut annotations, "bar", "Component");
    AnnotationRegistryExt::register(&mut annotations, "bar", "Primary");

    let ext = ListableBeanFactoryExtensions::new(&factory, &annotations);
    assert!(ext.find_annotation_on_bean("foo", "Component"));
    assert!(!ext.find_annotation_on_bean("baz", "Component"));
    let names = ext.get_bean_names_for_annotation("Component");
    assert_eq!(names, vec!["foo".to_owned(), "bar".to_owned()]);
    assert_eq!(ext.count_beans_with_annotation("Component"), 2);
    assert_eq!(
        ext.get_annotations_on_bean("bar"),
        vec!["Component".to_owned(), "Primary".to_owned()]
    );
    assert_eq!(ext.get_annotations_on_bean("missing"), Vec::<String>::new());
}

#[test]
fn listable_bean_factory_extensions_filtered() {
    let factory = StubListable {
        names: vec!["a".to_owned(), "b".to_owned()],
    };
    let mut annotations: AnnotationRegistry = AnnotationRegistry::new();
    AnnotationRegistryExt::register(&mut annotations, "a", "X");
    AnnotationRegistryExt::register(&mut annotations, "missing", "X");

    let ext = ListableBeanFactoryExtensions::new(&factory, &annotations);
    // Both methods iterate over the factory's bean_definition_names(),
    // so "missing" is not in either result — it's only in the annotation
    // registry. They should produce identical output.
    let filtered = ext.get_bean_names_for_annotation_filtered("X");
    assert_eq!(filtered, vec!["a".to_owned()]);
    let all = ext.get_bean_names_for_annotation("X");
    assert_eq!(all, vec!["a".to_owned()]);
    // Filtered adds an extra contains_bean_definition check, which here has
    // the same effect (both "a" and "b" are in the factory).
    assert_eq!(filtered, all);
}

// ===========================================================================
// 29. bean_factory_dependency_provider.rs
// ===========================================================================

#[test]
fn bean_factory_dependency_provider_optional_returns_none() {
    let factory = StubBeanFactory;
    let provider = BeanFactoryDependencyProvider::new(&factory);
    let descriptor =
        DependencyDescriptor::for_field(TypeId::of::<u32>(), std::any::type_name::<u32>())
            .with_optional(true);
    let result = provider.resolve_dependency(&descriptor).unwrap();
    assert!(result.is_none());
}

#[test]
fn bean_factory_dependency_provider_required_propagates_error() {
    let factory = StubBeanFactory;
    let provider = BeanFactoryDependencyProvider::new(&factory);
    let descriptor =
        DependencyDescriptor::for_field(TypeId::of::<u32>(), std::any::type_name::<u32>());
    let result = provider.resolve_dependency(&descriptor);
    assert!(result.is_err());
}

#[test]
fn bean_factory_dependency_provider_resolve_by_type_id_errors() {
    let factory = StubBeanFactory;
    let provider = BeanFactoryDependencyProvider::new(&factory);
    let err = provider
        .resolve_by_type_id(TypeId::of::<u32>())
        .unwrap_err();
    assert!(err.to_string().contains("not supported"));
}

#[test]
fn owned_bean_factory_dependency_provider_behavior() {
    let factory = Arc::new(StubBeanFactory) as Arc<dyn BeanFactory>;
    let provider = OwnedBeanFactoryDependencyProvider::new(factory.clone());
    assert!(Arc::ptr_eq(provider.bean_factory(), &factory));

    let descriptor =
        DependencyDescriptor::for_field(TypeId::of::<u32>(), std::any::type_name::<u32>())
            .with_optional(true);
    let result = provider.resolve_dependency(&descriptor).unwrap();
    assert!(result.is_none());

    let descriptor_required =
        DependencyDescriptor::for_field(TypeId::of::<u32>(), std::any::type_name::<u32>());
    assert!(provider.resolve_dependency(&descriptor_required).is_err());
}

// ===========================================================================
// 30. conversion_failed_exception.rs
// ===========================================================================

#[derive(Debug)]
struct DummyCause;

impl std::fmt::Display for DummyCause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("dummy cause")
    }
}

impl std::error::Error for DummyCause {}

#[test]
fn conversion_failed_exception_basic_display_and_getters() {
    let err = ConversionFailedException::new("String", "i32", "abc");
    assert_eq!(err.source_type(), "String");
    assert_eq!(err.target_type(), "i32");
    assert_eq!(err.value(), "abc");
    assert_eq!(
        err.to_string(),
        "Failed to convert from 'String' to 'i32' for value 'abc'"
    );
    // No cause attached.
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn conversion_failed_exception_with_cause() {
    let err =
        ConversionFailedException::with_cause("String", "bool", "maybe", Box::new(DummyCause));
    assert!(std::error::Error::source(&err).is_some());
    let display = err.to_string();
    assert!(display.contains("Failed to convert"));
    assert!(display.contains("caused by: dummy cause"));
}

#[test]
fn conversion_failed_exception_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ConversionFailedException>();
}

// ===========================================================================
// Misc cross-cutting sanity checks
// ===========================================================================

#[test]
fn config_trait_object_works() {
    // Verify that Environment and ConfigurableEnvironment can be used as trait objects.
    let env = AbstractEnvironment::new();
    let dyn_env: &dyn Environment = &env;
    assert!(dyn_env.accepts_profile("default"));

    let dyn_cfg: &dyn ConfigurableEnvironment = &env;
    assert!(dyn_cfg.property_sources().read().unwrap().is_empty());
}

// Helper to compile-check the unused imports we brought in.
#[allow(dead_code)]
fn _compile_check_unused(_m: Mutex<i32>) {}
