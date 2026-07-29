//! Boundary tests for Environment / Annotation / Configuration modules.
//!
//! Targets 29 source files and exercises previously uncovered paths:
//! placeholder resolution, profile management, configurable environment, parser
//! composition, type filters, bean reference resolution, etc.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use vernal_beans::*;

// Module helpers used in tests.
use vernal_beans::abstract_environment::default_profile_name;
use vernal_beans::placeholder_resolver::{
    PLACEHOLDER_PREFIX, PLACEHOLDER_SUFFIX, PlaceholderResolver, UnresolvablePlaceholderError,
    VALUE_SEPARATOR,
};
use vernal_beans::profiles::RESERVED_DEFAULT_PROFILE_NAME;
use vernal_beans::standard_environment::{
    SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME, SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME,
    build_system_environment, build_system_properties,
};

// ===========================================================================
// 1. environment.rs - SimpleEnvironment
// ===========================================================================

#[test]
fn env_simple_contains_and_get_required_property() {
    let mut env = SimpleEnvironment::new();
    env.set_property("present", "yes");
    assert!(env.contains_property("present"));
    assert!(!env.contains_property("missing"));

    let value = env.get_property("present");
    assert_eq!(value.as_deref(), Some("yes"));

    let err = env.get_required_property("missing");
    assert!(err.is_err());
    assert_eq!(err.unwrap_err().key, "missing");
}

#[test]
fn env_simple_active_profiles_and_resolution() {
    let mut env = SimpleEnvironment::new();
    env.set_active_profiles(["dev", "cloud"]);
    env.set_default_profiles(["fallback"]);

    assert_eq!(env.get_active_profiles(), vec!["dev", "cloud"]);
    assert_eq!(env.get_default_profiles(), vec!["fallback"]);
    assert!(env.accepts_profile("dev"));
    assert!(!env.accepts_profile("prod"));
    assert!(env.accepts_profiles(&["cloud".to_owned(), "nope".to_owned()]));

    // verify defaults used when no active set
    let mut env2 = SimpleEnvironment::new();
    assert_eq!(env2.get_active_profiles(), vec!["default"]);
}

#[test]
fn env_simple_placeholder_resolution() {
    let mut env = SimpleEnvironment::new();
    env.set_property("name", "vernal");
    env.set_property("nested", "Hello ${name}-world");

    assert_eq!(env.resolve_placeholders("${name}"), "vernal");
    assert_eq!(env.resolve_placeholders("Hi ${name}"), "Hi vernal");
    assert_eq!(env.resolve_placeholders("${nested}"), "Hello vernal-world");
    // unknown placeholder without default -> kept as-is
    assert_eq!(env.resolve_placeholders("${unknown}"), "${unknown}");
    // with default
    assert_eq!(env.resolve_placeholders("${unknown:fb}"), "fb");

    // required placeholders
    let ok = env.resolve_required_placeholders("${name}");
    assert_eq!(ok.unwrap(), "vernal");
    let err = env.resolve_required_placeholders("${missing}");
    assert!(err.is_err());
}

// ===========================================================================
// 2. property_source.rs
// ===========================================================================

#[test]
fn property_source_all_getters() {
    let mut ps = PropertySource::new("app");
    assert_eq!(ps.name(), "app");
    assert!(ps.is_empty());

    ps.set_property("a", "1");
    ps.set_property("b", "2");
    assert!(ps.contains_property("a"));
    assert!(!ps.contains_property("missing"));
    assert_eq!(ps.get_property("a"), Some("1"));
    assert_eq!(ps.get_property("missing"), None);

    assert_eq!(ps.len(), 2);
    let mut names: Vec<&str> = ps.property_names().into_iter().collect();
    names.sort();
    assert_eq!(names, vec!["a", "b"]);

    let removed = ps.remove_property("a");
    assert_eq!(removed.as_deref(), Some("1"));
    assert!(!ps.contains_property("a"));
}

#[test]
fn property_source_with_name_and_source() {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("x".to_owned(), "1".to_owned());
    map.insert("y".to_owned(), "2".to_owned());
    let ps = PropertySource::with_source("custom", map.clone());
    assert_eq!(ps.name(), "custom");
    assert_eq!(ps.source(), &map);

    let ps2 = PropertySource::from_pairs(
        "pairs",
        vec![
            ("k1".to_owned(), "v1".to_owned()),
            ("k2".to_owned(), "v2".to_owned()),
        ],
    );
    assert_eq!(ps2.get_property("k1"), Some("v1"));
    assert_eq!(ps2.len(), 2);

    // source_mut
    let mut ps3 = PropertySource::new("mut");
    ps3.source_mut().insert("k".to_owned(), "v".to_owned());
    assert_eq!(ps3.get_property("k"), Some("v"));
}

// ===========================================================================
// 3. property_sources.rs
// ===========================================================================

#[test]
fn property_sources_add_first_and_iterator() {
    let mut sources = PropertySources::new();
    let mut map1: HashMap<String, String> = HashMap::new();
    map1.insert("k".to_owned(), "1".to_owned());
    sources.add(PropertySource::with_source("first", map1));

    let mut map2: HashMap<String, String> = HashMap::new();
    map2.insert("k".to_owned(), "2".to_owned());
    sources.add(PropertySource::with_source("second", map2));

    assert_eq!(sources.len(), 2);
    assert_eq!(sources.names(), vec!["first", "second"]);
    assert_eq!(sources.get_property("k"), Some("1"));

    // iterate
    let collected: Vec<&str> = sources.sources().iter().map(|s| s.name()).collect();
    assert_eq!(collected, vec!["first", "second"]);
}

#[test]
fn property_sources_remove_and_get_by_name() {
    let mut sources = PropertySources::new();
    let p1 = PropertySource::from_pairs("a", vec![("v".to_owned(), "1".to_owned())]);
    let p2 = PropertySource::from_pairs("b", vec![("v".to_owned(), "2".to_owned())]);
    sources.add(p1);
    sources.add(p2);

    assert!(sources.contains("a"));
    assert!(sources.get("a").is_some());
    assert!(sources.get_mut("a").is_some());
    assert!(sources.contains_property("v"));

    let got = sources.get("a").unwrap();
    assert_eq!(got.get_property("v"), Some("1"));

    let removed = sources.remove("a");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name(), "a");
    assert!(!sources.contains("a"));

    // remove missing -> None
    assert!(sources.remove("missing").is_none());

    // clear empties
    sources.clear();
    assert!(sources.is_empty());
}

// ===========================================================================
// 4. mutable_property_sources.rs
// ===========================================================================

#[test]
fn mutable_property_sources_add_first_and_last() {
    let mut mps = MutablePropertySources::new();
    let p1 = PropertySource::from_pairs("a", vec![("k".to_owned(), "1".to_owned())]);
    let p2 = PropertySource::from_pairs("b", vec![("k".to_owned(), "2".to_owned())]);
    mps.add_last(p1);
    mps.add_first(p2);
    assert_eq!(mps.names(), vec!["b", "a"]);
    assert_eq!(mps.get_property("k"), Some("2"));
    assert_eq!(mps.len(), 2);
    assert!(!mps.is_empty());

    // add_first of existing name promotes & dedupes
    let p1_dup = PropertySource::from_pairs("a", vec![("k".to_owned(), "9".to_owned())]);
    mps.add_first(p1_dup);
    assert_eq!(mps.len(), 2);
    assert_eq!(mps.get_property("k"), Some("9"));
}

#[test]
fn mutable_property_sources_remove_and_replace() {
    let mut mps = MutablePropertySources::new();
    mps.add_last(PropertySource::from_pairs(
        "a",
        vec![("x".to_owned(), "1".to_owned())],
    ));
    mps.add_last(PropertySource::from_pairs(
        "b",
        vec![("x".to_owned(), "2".to_owned())],
    ));

    let removed = mps.remove("a");
    assert!(removed.is_some());
    assert_eq!(mps.len(), 1);

    let replaced = mps.replace(PropertySource::from_pairs(
        "b",
        vec![("x".to_owned(), "99".to_owned())],
    ));
    assert!(replaced);
    assert_eq!(mps.get_property("x"), Some("99"));

    assert!(mps.get("b").is_some());
    assert!(mps.contains("b"));
    assert!(mps.get_mut("b").is_some());

    // as_property_sources / into_property_sources
    let _ref = mps.as_property_sources();
    let inner_prop_sources = mps.into_property_sources();
    assert_eq!(inner_prop_sources.len(), 1);
}

#[test]
fn mutable_property_sources_add_before_after() {
    let mut mps = MutablePropertySources::new();
    mps.add_last(PropertySource::from_pairs(
        "a",
        vec![("k".to_owned(), "a".to_owned())],
    ));
    mps.add_last(PropertySource::from_pairs(
        "c",
        vec![("k".to_owned(), "c".to_owned())],
    ));

    mps.add_before(
        "c",
        PropertySource::from_pairs("b", vec![("k".to_owned(), "b".to_owned())]),
    );
    assert_eq!(mps.names(), vec!["a", "b", "c"]);

    mps.add_after(
        "a",
        PropertySource::from_pairs("z", vec![("k".to_owned(), "z".to_owned())]),
    );
    let mut after_names = mps.names();
    after_names.sort();
    assert!(after_names.contains(&"z"));
}

// ===========================================================================
// 5. property_resolver.rs - nested placeholders
// ===========================================================================

#[test]
fn property_resolver_placeholder_nested_and_ignore() {
    let mut env = SimpleEnvironment::new();
    env.set_property("A", "x");
    env.set_property("B", "${A}");
    env.set_property("C", "${B}-suffix");

    // chained resolution
    assert_eq!(env.resolve_placeholders("${C}"), "x-suffix");

    // ignore_unresolvable: keep as-is on missing
    assert_eq!(env.resolve_placeholders("${missing}"), "${missing}");
    assert_eq!(
        env.resolve_placeholders("X=${missing},Y=${A}"),
        "X=${missing},Y=x"
    );

    // required surfaces error
    let required = env.resolve_required_placeholders("${missing}");
    assert!(required.is_err());
}

#[test]
fn property_resolver_get_property_or_default() {
    let mut env = SimpleEnvironment::new();
    env.set_property("k", "v");
    assert_eq!(env.get_property_or("k", "fb"), "v");
    assert_eq!(env.get_property_or("missing", "fb"), "fb");
}

// ===========================================================================
// 6. configurable_environment.rs
// ===========================================================================

#[test]
fn configurable_environment_sources_and_active_profile() {
    let mut env = AbstractEnvironment::new();
    let sources = env.property_sources();
    {
        let mut guard = sources.write().unwrap();
        guard.add_last(PropertySource::from_pairs(
            "composite",
            vec![("k".to_owned(), "v".to_owned())],
        ));
    }
    assert!(env.property_sources().read().unwrap().contains("composite"));

    assert_eq!(env.get_active_profiles(), vec!["default"]);

    env.add_active_profile("dev");
    assert!(env.get_active_profiles().contains(&"dev".to_owned()));

    env.set_active_profiles(&["prod".to_owned()]);
    assert_eq!(env.get_active_profiles(), vec!["prod"]);

    env.set_default_profiles(&["base".to_owned()]);
    assert_eq!(env.get_default_profiles(), vec!["base"]);

    // system_properties / system_environment default empty
    assert!(env.get_system_properties().is_empty());
    assert!(env.get_system_environment().is_empty());
}

#[test]
fn configurable_environment_merge() {
    let mut parent = AbstractEnvironment::new();
    {
        let parent_sources = parent.property_sources();
        let mut guard = parent_sources.write().unwrap();
        guard.add_last(PropertySource::from_pairs(
            "parentOnly",
            vec![("parentKey".to_owned(), "parentVal".to_owned())],
        ));
    }
    let mut child = AbstractEnvironment::new();
    child.merge(&parent);
    assert_eq!(
        child.get_property("parentKey"),
        Some("parentVal".to_owned())
    );

    // child already has "parentOnly" -> not re-added
    {
        let child_sources = child.property_sources();
        let mut guard = child_sources.write().unwrap();
        guard.add_last(PropertySource::from_pairs(
            "parentOnly",
            vec![("parentKey".to_owned(), "childVal".to_owned())],
        ));
    }
    let mut child2 = AbstractEnvironment::new();
    child2.merge(&parent);
    // not added because child already has it - but child2 doesn't have it, so it IS added
    let child2_sources = child2.property_sources();
    assert_eq!(child2_sources.read().unwrap().len(), 1);
}

// ===========================================================================
// 7. abstract_environment.rs - resolve_placeholders with active profiles
// ===========================================================================

#[test]
fn abstract_environment_resolve_with_active_profile() {
    let mut env = AbstractEnvironment::new();
    {
        let sources_arc = env.property_sources();
        let mut sources = sources_arc.write().unwrap();
        sources.add_first(PropertySource::from_pairs(
            "app",
            vec![
                ("name".to_owned(), "vernal".to_owned()),
                ("greeting".to_owned(), "Hi ${name}".to_owned()),
                ("complex".to_owned(), "${greeting} friend".to_owned()),
            ],
        ));
    }

    // active profiles
    env.set_active_profiles(&["dev".to_owned()]);
    assert_eq!(env.get_active_profiles(), vec!["dev"]);
    assert!(env.accepts_profile("dev"));
    assert!(env.accepts_profiles(&["dev".to_owned()]));
    assert!(!env.accepts_profiles(&["prod".to_owned()]));

    // resolve nested
    assert_eq!(env.resolve_placeholders("${name}"), "vernal");
    assert_eq!(env.resolve_placeholders("${greeting}"), "Hi vernal");
    assert_eq!(env.resolve_placeholders("${complex}"), "Hi vernal friend");

    // default profile fallback
    let env2 = AbstractEnvironment::new();
    assert_eq!(env2.get_active_profiles(), vec!["default"]);
    assert_eq!(default_profile_name(), RESERVED_DEFAULT_PROFILE_NAME);
}

#[test]
fn abstract_environment_system_properties_and_environment() {
    let mut env = AbstractEnvironment::new();
    let mut sys_props: HashMap<String, String> = HashMap::new();
    sys_props.insert("foo".to_owned(), "bar".to_owned());
    env.set_system_properties(sys_props.clone());
    let mut sys_env: HashMap<String, String> = HashMap::new();
    sys_env.insert("BAZ".to_owned(), "qux".to_owned());
    env.set_system_environment(sys_env.clone());

    assert_eq!(env.get_property("foo"), Some("bar".to_owned()));
    assert_eq!(env.get_property("BAZ"), Some("qux".to_owned()));
    assert!(env.contains_property("foo"));
    assert_eq!(env.get_system_properties(), sys_props);
    assert_eq!(env.get_system_environment(), sys_env);

    // required missing
    assert!(env.get_required_property("missing").is_err());
}

// ===========================================================================
// 8. standard_environment.rs
// ===========================================================================

#[test]
fn standard_environment_system_sources() {
    let env = StandardEnvironment::new();
    let sources = env.property_sources();
    let guard = sources.read().unwrap();
    assert!(guard.contains(SYSTEM_PROPERTIES_PROPERTY_SOURCE_NAME));
    assert!(guard.contains(SYSTEM_ENVIRONMENT_PROPERTY_SOURCE_NAME));

    // build_system_environment / build_system_properties exist
    let _ = build_system_properties();
    let _ = build_system_environment();
}

#[test]
fn standard_environment_with_sources_resolves() {
    let mut props: HashMap<String, String> = HashMap::new();
    props.insert("KEY1".to_owned(), "VAL1".to_owned());
    let mut env: HashMap<String, String> = HashMap::new();
    env.insert("KEY2".to_owned(), "VAL2".to_owned());

    let std_env = StandardEnvironment::with_sources(props, env);
    assert_eq!(std_env.get_property("KEY1"), Some("VAL1".to_owned()));
    assert_eq!(std_env.get_property("KEY2"), Some("VAL2".to_owned()));
    assert!(std_env.contains_property("KEY1"));
    assert_eq!(std_env.get_active_profiles(), vec!["default"]);

    // construct with default's call path
    let _ = StandardEnvironment::default();
}

// ===========================================================================
// 9. profile.rs
// ===========================================================================

#[test]
fn profile_create_equality_default() {
    let p = Profile::new("dev");
    assert_eq!(p.name(), "dev");
    assert!(!p.is_default());
    assert_eq!(p.to_string(), "dev");

    let mut d = Profile::new_default("fallback");
    assert!(d.is_default());
    d.set_default(false);
    assert!(!d.is_default());

    let p2 = Profile::new("dev");
    assert_eq!(p, p2);
    // hash equality (compile-time check via use in HashSet)
    let mut set: std::collections::HashSet<Profile> = std::collections::HashSet::new();
    set.insert(p.clone());
    set.insert(p2.clone());
    assert_eq!(set.len(), 1);
}

#[test]
fn profile_ordering() {
    let a = Profile::new("a");
    let b = Profile::new("b");
    assert!(a < b);
    assert!(b > a);
}

// ===========================================================================
// 10. profiles.rs
// ===========================================================================

#[test]
fn profiles_contains_and_default() {
    let mut p = Profiles::new();
    assert!(p.is_active("default"));
    assert!(p.is_active(RESERVED_DEFAULT_PROFILE_NAME));
    assert!(!p.has_explicit_active());
    assert_eq!(p.get_active(), vec!["default"]);
    assert_eq!(p.get_default(), vec!["default"]);

    p.add("dev");
    p.add("dev"); // dedup
    assert!(p.is_active("dev"));
    assert!(p.is_active("dev"));
    assert_eq!(p.active_count(), 1);
    assert!(p.has_explicit_active());
    assert!(!p.is_active("default"));

    p.remove("dev");
    assert!(!p.is_active("dev"));
    assert_eq!(p.active_count(), 0);

    // set_active / set_default replacements
    p.set_active(&["test".to_owned()]);
    assert!(p.is_active("test"));
    p.set_default(&["base-default".to_owned()]);
    assert_eq!(p.get_default(), vec!["base-default"]);

    // add_default
    p.add_default("extra");
    let mut defs = p.get_default();
    defs.sort();
    assert!(defs.contains(&"extra"));

    // accept
    p.accept(&["prod".to_owned()]);
    assert_eq!(p.get_active(), vec!["prod"]);
}

#[test]
fn profiles_accept_method() {
    let mut p = Profiles::new();
    p.accept(&["a".to_owned(), "b".to_owned()]);
    assert!(p.is_active("a"));
    assert!(p.is_active("b"));
    assert!(!p.is_active("default"));
}

// ===========================================================================
// 11. placeholder_resolver.rs - multiple placeholders
// ===========================================================================

#[test]
fn placeholder_resolver_multiple_placeholders() {
    let mut sources = PropertySources::new();
    sources.add(PropertySource::from_pairs(
        "app",
        vec![
            ("name".to_owned(), "vernal".to_owned()),
            ("version".to_owned(), "1.0".to_owned()),
            ("banner".to_owned(), "${name}@${version}".to_owned()),
        ],
    ));
    let resolver = PlaceholderResolver::new(&sources);

    assert_eq!(
        resolver.resolve_placeholders("App: ${name}-${version}"),
        "App: vernal-1.0"
    );
    assert_eq!(resolver.resolve_placeholders("${banner}"), "vernal@1.0");

    // resolve_required_placeholders: success
    let ok = resolver.resolve_required_placeholders("${name}").unwrap();
    assert_eq!(ok, "vernal");

    // resolve_required_placeholders: failure with details
    let err = resolver
        .resolve_required_placeholders("${missing}")
        .unwrap_err();
    let display = format!("{}", err);
    assert!(display.contains("missing"));
    assert_eq!(err.placeholders, vec!["${missing}".to_owned()]);

    // nested
    let nested = resolver.resolve_placeholders("${name:${version}}");
    assert_eq!(nested, "vernal");
}

#[test]
fn placeholder_resolver_required_partial() {
    let mut sources = PropertySources::new();
    sources.add(PropertySource::from_pairs(
        "app",
        vec![("a".to_owned(), "x".to_owned())],
    ));
    let resolver = PlaceholderResolver::new(&sources);
    let result = resolver.resolve_required_placeholders("${a} and ${b}");
    assert!(result.is_err());
    let err: UnresolvablePlaceholderError = result.unwrap_err();
    assert_eq!(err.placeholders, vec!["${b}".to_owned()]);

    // constants
    assert_eq!(PLACEHOLDER_PREFIX, "${");
    assert_eq!(PLACEHOLDER_SUFFIX, "}");
    assert_eq!(VALUE_SEPARATOR, ":");
}

// ===========================================================================
// 12. string_value_resolver.rs
// ===========================================================================

#[test]
fn string_value_resolver_default_impls() {
    let identity = IdentityStringValueResolver;
    assert_eq!(identity.resolve_string_value("hello"), "hello");
    assert_eq!(identity.resolve_string_value(""), "");

    // closure as resolver
    let closure: Box<dyn StringValueResolver> = Box::new(|v: &str| format!("[{}]", v));
    assert_eq!(closure.resolve_string_value("x"), "[x]");
}

#[test]
fn string_value_resolver_send_sync() {
    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn StringValueResolver>();
    assert_send_sync::<IdentityStringValueResolver>();
}

// ===========================================================================
// 13. embedded_value_resolver.rs
// ===========================================================================

#[test]
fn embedded_value_resolver_various_inputs() {
    let mut env = SimpleEnvironment::new();
    env.set_property("greeting", "hi");
    env.set_property("name", "vernal");
    let resolver = EmbeddedValueResolver::new(Arc::new(env));

    // exact placeholder
    assert_eq!(resolver.resolve_string_value("${greeting}"), "hi");

    // embedded in text
    assert_eq!(
        resolver.resolve_string_value("Hello ${name}!"),
        "Hello vernal!"
    );

    // missing placeholder kept as-is
    assert_eq!(resolver.resolve_string_value("${missing}"), "${missing}");

    // access environment
    let _ = resolver.environment();
}

#[test]
fn embedded_value_resolver_arc_access() {
    let mut env = SimpleEnvironment::new();
    env.set_property("k", "v");
    let env_arc: Arc<dyn Environment + Send + Sync> = Arc::new(env);
    let resolver = EmbeddedValueResolver::new(env_arc.clone());
    let _ = resolver.environment();
    let _ = env_arc.get_property("k");
}

// ===========================================================================
// 14. configuration_class.rs
// ===========================================================================

#[test]
fn configuration_class_create_with_all_fields() {
    let mut config = ConfigurationClass::new("com.example.AppConfig");
    config.set_source("classpath:com/example/AppConfig.rs");
    config.set_full(false);
    let mut method = ConfigurationBeanMethod::new("createFoo", "com.example.Foo");
    method = method.with_bean_name("fooBean");
    config.add_bean_method_detail(method);
    config.add_bean_method("createFoo"); // dedup
    config.add_bean_method("createBar");
    config.add_import("com.example.OtherConfig");
    config.add_import("com.example.OtherConfig"); // dedup

    assert_eq!(config.class_name(), "com.example.AppConfig");
    assert_eq!(config.source(), Some("classpath:com/example/AppConfig.rs"));
    assert!(!config.is_full());
    assert_eq!(config.bean_method_count(), 2);
    assert_eq!(config.bean_methods(), &["createFoo", "createBar"]);
    assert_eq!(config.bean_method_details().len(), 1);
    assert_eq!(config.bean_method_details()[0].bean_name, "fooBean");
    assert_eq!(config.bean_method_details()[0].method_name, "createFoo");
    assert_eq!(
        config.bean_method_details()[0].return_type_name,
        "com.example.Foo"
    );
    assert_eq!(config.imported(), &["com.example.OtherConfig".to_owned()]);
}

#[test]
fn configuration_class_default_full() {
    let config = ConfigurationClass::new("Demo");
    assert!(config.is_full());
    assert!(config.source().is_none());
    assert!(config.bean_methods().is_empty());
    assert!(config.bean_method_details().is_empty());
    assert!(config.imported().is_empty());
}

// ===========================================================================
// 15. configuration_class_parser.rs
// ===========================================================================

#[test]
fn configuration_class_parser_parse_default() {
    let parser = ConfigurationClassParser::new();
    let classes = parser.parse_configuration_classes(&[
        "com.example.AppConfig".to_owned(),
        "com.example.PlainBean".to_owned(),
        "com.example.DatabaseConfig".to_owned(),
    ]);
    assert_eq!(classes.len(), 2);
    assert_eq!(classes[0].class_name(), "com.example.AppConfig");
    assert_eq!(classes[1].class_name(), "com.example.DatabaseConfig");
}

#[test]
fn configuration_class_parser_custom_providers() {
    let parser = ConfigurationClassParser::new()
        .with_configuration_predicate(|n| n.contains("Config"))
        .with_bean_methods_provider(|n| {
            if n == "com.example.AppConfig" {
                vec!["createFoo".to_owned(), "createBar".to_owned()]
            } else {
                vec![]
            }
        })
        .with_imports_provider(|n| vec![format!("{}.Import", n)]);
    let classes = parser.parse_configuration_classes(&[
        "com.example.AppConfig".to_owned(),
        "com.example.NotConfig".to_owned(),
    ]);
    assert_eq!(classes.len(), 2);
    let app = classes
        .iter()
        .find(|c| c.class_name() == "com.example.AppConfig")
        .unwrap();
    assert_eq!(app.bean_methods().len(), 2);
    assert_eq!(app.imported().len(), 1);
    assert_eq!(app.imported()[0], "com.example.AppConfig.Import");
}

// ===========================================================================
// 16. configuration_class_bean_definition_reader.rs
// ===========================================================================

#[test]
fn configuration_class_bean_definition_reader_load() {
    let mut config = ConfigurationClass::new("com.example.AppConfig");
    config.add_bean_method_detail(
        ConfigurationBeanMethod::new("createFoo", "com.example.Foo").with_bean_name("foo"),
    );
    config.add_bean_method_detail(
        ConfigurationBeanMethod::new("createBar", "com.example.Bar").with_bean_name("bar"),
    );

    let mut reader = ConfigurationClassBeanDefinitionReader::new();
    reader.load_bean_definitions(&[config]);

    assert_eq!(reader.len(), 2);
    assert!(!reader.is_empty());

    let foo = reader.get("foo").unwrap();
    assert_eq!(foo.factory_method, "createFoo");
    assert_eq!(foo.factory_bean_name, "com.example.AppConfig");
    assert_eq!(foo.return_type_name, "com.example.Foo");

    let bar = reader.get("bar").unwrap();
    assert_eq!(bar.method_name_unused(), "createBar");

    let definitions = reader.loaded_definitions();
    assert_eq!(definitions.len(), 2);
}

#[test]
fn configuration_class_bean_definition_reader_default_method() {
    // default ctor
    let mut reader: ConfigurationClassBeanDefinitionReader = Default::default();
    assert!(reader.is_empty());
    let config = ConfigurationClass::new("X");
    reader.load_bean_definitions(&[config]);
    assert_eq!(reader.len(), 0);
}

// ===========================================================================
// 17. import_selector.rs
// ===========================================================================

#[test]
fn import_selector_default_select_imports() {
    let selector = FixedImportSelector::new(["com.example.A", "com.example.B"]);
    let meta = EmptyMetadata;

    assert_eq!(
        selector.select_imports(&meta),
        vec!["com.example.A".to_owned(), "com.example.B".to_owned()]
    );
    assert!(!selector.is_excluded("com.example.A"));
}

#[test]
fn import_selector_excluded() {
    let selector = FixedImportSelector::new(["com.example.A", "com.example.B"])
        .with_excluded(["com.example.B"]);
    let meta = EmptyMetadata;
    assert_eq!(
        selector.select_imports(&meta),
        vec!["com.example.A".to_owned()]
    );
    assert!(selector.is_excluded("com.example.B"));
    assert!(!selector.is_excluded("com.example.A"));
}

// ===========================================================================
// 18. deferred_import_selector.rs
// ===========================================================================

#[test]
fn deferred_import_selector_get_import_group() {
    let selector = FixedDeferredImportSelector::new(["com.example.A", "com.example.B"]);
    let meta = EmptyMetadata;
    // default implementation returns None
    assert!(selector.group().is_none());

    // select_imports returns the fixed imports
    assert_eq!(
        selector.select_imports(&meta),
        vec!["com.example.A".to_owned(), "com.example.B".to_owned()]
    );
}

#[test]
fn deferred_import_selector_default_group() {
    let mut group = DefaultDeferredImportGroup::new();
    assert!(group.is_empty());
    assert_eq!(group.len(), 0);

    let selector = FixedDeferredImportSelector::new(["com.example.A"]);
    let meta = EmptyMetadata;
    group.process(&meta, &selector);
    assert_eq!(group.len(), 1);
    assert_eq!(group.select_imports(), vec!["com.example.A".to_owned()]);

    group.clear();
    assert!(group.is_empty());
}

// ===========================================================================
// 19. import_bean_definition_registrar.rs
// ===========================================================================

#[test]
fn import_bean_definition_registrar_register() {
    let mut composite = CompositeImportBeanDefinitionRegistrar::new();
    assert!(composite.is_empty());
    assert_eq!(composite.len(), 0);

    composite.add(Box::new(NoopImportBeanDefinitionRegistrar));
    composite.add(Box::new(NoopImportBeanDefinitionRegistrar));
    assert_eq!(composite.len(), 2);
    assert!(!composite.is_empty());

    // Default impl
    let _ = CompositeImportBeanDefinitionRegistrar::default();
}

#[test]
fn import_bean_definition_registrar_composite_runs() {
    let recorder = Arc::new(Mutex::new(Vec::<String>::new()));
    let rec = recorder.clone();
    let reg = TestRegistrar::new("alpha", rec.clone());
    let reg2 = TestRegistrar::new("beta", rec.clone());

    let mut composite = CompositeImportBeanDefinitionRegistrar::new();
    composite.add(Box::new(reg));
    composite.add(Box::new(reg2));

    let mut registry = DummyRegistry::new();
    let meta = EmptyMetadata;
    composite
        .register_bean_definitions(&meta, &mut registry)
        .unwrap();
    let names = recorder.lock().unwrap().clone();
    assert_eq!(names, vec!["alpha".to_owned(), "beta".to_owned()]);
}

// ===========================================================================
// 20. annotation_metadata.rs - custom impl
// ===========================================================================

#[test]
fn annotation_metadata_custom_impl() {
    let meta = SampleMetadata {
        annotations: vec![
            AnnotationDescriptor::new_class("Component"),
            AnnotationDescriptor::new_method("Bean", "createFoo"),
            AnnotationDescriptor::new_field("Autowired", "field"),
        ],
    };

    assert!(meta.has_annotation("Component"));
    assert!(meta.has_meta_annotation("Bean"));
    assert!(meta.has_class_annotation("Component"));
    assert!(!meta.has_class_annotation("Bean"));
    assert!(meta.has_method_annotation("Bean"));
    assert!(meta.is_class_level());
    assert!(meta.is_method_level());
    assert!(meta.is_field_level());

    let types = meta.get_annotation_types();
    assert!(types.contains(&"Component".to_owned()));
    assert!(types.contains(&"Bean".to_owned()));
    assert_eq!(types.len(), 3);

    let method_anns = meta.get_method_annotations();
    assert_eq!(method_anns.len(), 1);
    assert_eq!(method_anns[0].method_name(), Some("createFoo"));

    assert_eq!(meta.get_annotation_type(), Some("Component"));
    assert_eq!(meta.get_method_name(), Some("createFoo"));

    let first = meta.get_annotation("Component").unwrap();
    assert_eq!(first.annotation_type(), "Component");
}

#[test]
fn annotation_descriptor_attribute_mutation() {
    let mut d = AnnotationDescriptor::new_class("Service");
    assert!(d.attributes().is_empty());
    d.set_attribute("value", "myService");
    assert_eq!(
        d.attributes().get("value").map(String::as_str),
        Some("myService")
    );
    assert!(d.is_class_level());
    assert_eq!(d.member_name(), None);

    let mut via_mut = d.attributes_mut();
    via_mut.insert("role".to_owned(), "admin".to_owned());

    let field = AnnotationDescriptor::new_field("Inject", "f");
    assert!(field.is_field_level());
    assert_eq!(field.member_name(), Some("f"));
}

// ===========================================================================
// 21. annotation_type_mapping.rs - builder pattern
// ===========================================================================

#[test]
fn annotation_type_mapping_basic() {
    let mut m = AnnotationTypeMapping::new("Service");
    m.set_attribute("value", "myService");
    m.set_attribute("scope", "singleton");

    assert_eq!(m.annotation_type(), "Service");
    assert_eq!(m.get_attribute("value"), Some("myService"));
    assert!(m.has_attribute("value"));
    assert!(m.is_direct());
    assert!(!m.is_meta());
    assert_eq!(m.attribute_count(), 2);

    let _ = m.attributes_mut();
}

#[test]
fn annotation_type_mapping_meta_with_source() {
    let mut m = AnnotationTypeMapping::with_source("Component", "Service", 1);
    m.set_attribute("value", "myComponent");
    assert_eq!(m.annotation_type(), "Component");
    assert_eq!(m.source(), Some("Service"));
    assert_eq!(m.distance(), 1);
    assert!(m.is_meta());
    assert!(!m.is_direct());

    let a = AnnotationTypeMapping::new("X");
    let b = AnnotationTypeMapping::new("X");
    assert_eq!(a, b);
}

// ===========================================================================
// 22. annotations_scanner.rs - custom impl
// ===========================================================================

#[test]
fn annotations_scanner_custom_impl() {
    let mut scanner = StaticAnnotationsScanner::new();
    let meta = Arc::new(SampleMetadata {
        annotations: vec![AnnotationDescriptor::new_class("Component")],
    });
    scanner.register("com.example.Foo", meta.clone());
    scanner.register("com.example.Bar", Arc::new(EmptyMetadata));

    let one = scanner.scan_class("com.example.Foo").unwrap();
    assert!(one.is_some());
    assert!(one.unwrap().has_annotation("Component"));

    let missing = scanner.scan_class("missing").unwrap();
    assert!(missing.is_none());

    let batch = scanner.scan(&[
        "com.example.Foo".to_owned(),
        "com.example.Bar".to_owned(),
        "missing".to_owned(),
    ]);
    assert_eq!(batch.unwrap().len(), 2);
}

#[test]
fn annotations_scanner_default_constructor() {
    let scanner: StaticAnnotationsScanner = Default::default();
    // empty
    let res = scanner.scan_class("anything").unwrap();
    assert!(res.is_none());
}

// ===========================================================================
// 23. type_filter.rs - RegexTypeFilter
// ===========================================================================

#[test]
fn type_filter_regex_patterns() {
    let filter = RegexTypeFilter::new(r"^com\.example\..*Service$").unwrap();
    assert!(filter.matches("com.example.UserService"));
    assert!(filter.matches("com.example.OrderService"));
    assert!(!filter.matches("com.example.Repository"));
    assert!(!filter.matches("org.other.UserService"));
    assert_eq!(filter.pattern_str(), r"^com\.example\..*Service$");
    let _ = filter.pattern();
}

#[test]
fn type_filter_prefix_and_not() {
    let prefix = PrefixIncludeFilter::new(["com.example", "org.demo"]);
    assert!(prefix.matches("com.example.Foo"));
    assert!(prefix.matches("org.demo.Bar"));
    assert!(!prefix.matches("net.other.Baz"));
    assert_eq!(prefix.len(), 2);
    assert!(!prefix.is_empty());

    let mut mut_prefix = PrefixIncludeFilter::new(Vec::<String>::new());
    assert!(mut_prefix.is_empty());
    mut_prefix.add("added");
    assert_eq!(mut_prefix.len(), 1);
    assert!(mut_prefix.matches("added.Suffix"));

    let not = NotFilter::new(PrefixIncludeFilter::new(["com.example"]));
    assert!(!not.matches("com.example.Foo"));
    assert!(not.matches("net.other.Baz"));

    // empty prefix matches everything
    let empty_prefix = PrefixIncludeFilter::new(Vec::<String>::new());
    assert!(empty_prefix.matches("anything"));
}

// ===========================================================================
// 24. metadata_reader.rs - custom impl
// ===========================================================================

#[test]
fn metadata_reader_custom_impl() {
    let reader = SimpleMetadataReaderImpl::new(
        Arc::new(SampleMetadata {
            annotations: vec![AnnotationDescriptor::new_class("Component")],
        }),
        Arc::new(SimpleClassMetadata::new("com.example.Foo")),
        "classpath:com/example/Foo.class",
    );

    assert_eq!(
        reader.resource_description(),
        "classpath:com/example/Foo.class"
    );
    let ann = reader.get_annotation_metadata();
    assert!(ann.has_annotation("Component"));
    let class = reader.get_class_metadata();
    assert_eq!(class.class_name(), "com.example.Foo");
    assert!(!class.is_interface());
    assert!(!class.is_final());
    assert!(class.is_concrete());
    assert!(!class.has_super_class());
    assert!(class.interface_names().is_empty());
}

#[test]
fn metadata_reader_class_metadata_variants() {
    let interface = SimpleClassMetadata::new("I").with_interface(true);
    assert!(interface.is_interface());
    assert!(!interface.is_concrete());

    let abstract_cls = SimpleClassMetadata::new("A").with_abstract(true);
    assert!(abstract_cls.is_abstract());
    assert!(!abstract_cls.is_concrete());

    let final_cls = SimpleClassMetadata::new("F").with_final(true);
    assert!(final_cls.is_final());

    let with_super = SimpleClassMetadata::new("Sub").with_super_class("Base");
    assert_eq!(with_super.super_class_name(), Some("Base"));
    assert!(with_super.has_super_class());

    let with_ifaces = SimpleClassMetadata::new("X")
        .with_interfaces(["java.io.Serializable", "java.lang.Cloneable"]);
    assert_eq!(with_ifaces.interface_names().len(), 2);
}

// ===========================================================================
// 25. simple_metadata_reader.rs - all read methods
// ===========================================================================

#[test]
fn simple_metadata_reader_all_read_methods() {
    let mut annotations = SimpleAnnotationMetadata::new();
    annotations.add(AnnotationDescriptor::new_class("Component"));
    annotations.add(AnnotationDescriptor::new_method("Bean", "createFoo"));

    let class = SimpleClassMetadata::new("com.example.Foo")
        .with_super_class("com.example.Base")
        .with_interface(true);

    let reader = SimpleMetadataReader::new(
        Arc::new(annotations),
        Arc::new(class),
        "classpath:com/example/Foo.class",
    );

    let dyn_ann = reader.get_annotation_metadata();
    assert!(dyn_ann.has_annotation("Component"));
    assert!(dyn_ann.has_method_annotation("Bean"));

    let dyn_class = reader.get_class_metadata();
    assert_eq!(dyn_class.class_name(), "com.example.Foo");
    assert!(dyn_class.is_interface());
    assert_eq!(dyn_class.super_class_name(), Some("com.example.Base"));

    assert_eq!(
        reader.resource_description(),
        "classpath:com/example/Foo.class"
    );

    // typed accessors
    let typed_ann = reader.simple_annotation_metadata();
    assert_eq!(typed_ann.annotations().len(), 2);
    let typed_class = reader.simple_class_metadata();
    assert_eq!(typed_class.class_name(), "com.example.Foo");
}

#[test]
fn simple_metadata_reader_factory() {
    let mut factory = SimpleMetadataReaderFactory::new();
    assert!(factory.is_empty());
    assert_eq!(factory.len(), 0);

    let reader = Arc::new(SimpleMetadataReader::new(
        Arc::new(SimpleAnnotationMetadata::new()),
        Arc::new(SimpleClassMetadata::new("com.example.Foo")),
        "classpath:com/example/Foo.class",
    ));
    factory.register("com.example.Foo", reader.clone());

    let got = factory.get_metadata_reader("com.example.Foo");
    assert!(got.is_some());
    assert_eq!(
        got.unwrap().resource_description(),
        "classpath:com/example/Foo.class"
    );

    let missing = factory.get_metadata_reader("missing");
    assert!(missing.is_none());

    // default ctor
    let factory_default: SimpleMetadataReaderFactory = Default::default();
    assert!(factory_default.is_empty());

    // from_annotations
    let meta =
        SimpleAnnotationMetadata::from_annotations(vec![AnnotationDescriptor::new_class("C")]);
    assert_eq!(meta.annotations().len(), 1);
}

// ===========================================================================
// 26. annotation_processor.rs - custom impl
// ===========================================================================

#[test]
fn annotation_processor_custom_impl() {
    let counter = Arc::new(CountingProcessor::new(vec!["Component".to_owned()]));
    let proc = counter.clone();
    let meta = SampleMetadata {
        annotations: vec![AnnotationDescriptor::new_class("Component")],
    };
    proc.process(&meta).unwrap();
    assert_eq!(*counter.calls.lock().unwrap(), 1);

    // supports()
    assert!(proc.supports("Component"));
    assert!(!proc.supports("Service"));
}

#[test]
fn annotation_processor_composite_and_supports_all() {
    let counter1 = Arc::new(CountingProcessor::new(vec![]));
    let counter2 = Arc::new(CountingProcessor::new(vec!["Component".to_owned()]));

    let mut composite = CompositeAnnotationProcessor::new();
    assert!(composite.is_empty());
    composite.add(counter1.clone());
    composite.add(counter2.clone());
    assert_eq!(composite.len(), 2);

    let meta = SampleMetadata {
        annotations: vec![
            AnnotationDescriptor::new_class("Component"),
            AnnotationDescriptor::new_class("Service"),
        ],
    };
    composite.process(&meta).unwrap();
    // counter1 fires for both annotations -> 2
    assert_eq!(*counter1.calls.lock().unwrap(), 2);
    // counter2 fires only once (for Component) -> 1
    assert_eq!(*counter2.calls.lock().unwrap(), 1);

    // default impl of supports when supported empty -> always true
    assert!(counter1.supports("Anything"));
    assert!(counter1.supports("Other"));
}

// ===========================================================================
// 27. bean_reference_resolver.rs - all resolve methods
// ===========================================================================

#[test]
fn bean_reference_resolver_resolve_methods() {
    let container = Arc::new(Container::new(Registry::empty()));
    let resolver = BeanReferenceResolver::new(container.clone());

    // container access
    let _ = resolver.container();

    // resolve_by_name returns None in current impl
    let r = resolver.resolve_by_name("missing").unwrap();
    assert!(r.is_none());

    // contains_bean always false in current impl
    assert!(!resolver.contains_bean("anything"));

    // resolve_reference delegates to resolve_by_name
    let reference = DummyBeanReference::new("myBean");
    let result = resolver.resolve_reference(&reference).unwrap();
    assert!(result.is_none());

    // is_resolvable -> contains_bean -> false
    assert!(!resolver.is_resolvable(&reference));
}

#[test]
fn bean_reference_resolver_dummy_reference() {
    let reference = DummyBeanReference::new("foo");
    assert_eq!(reference.get_bean_name(), "foo");
    assert!(reference.get_source().is_none());
}

// ===========================================================================
// 28. listable_bean_factory_extensions.rs - find_annotation_on_bean
// ===========================================================================

#[test]
fn listable_bean_factory_extensions_find_annotation_on_bean() {
    let factory = StubListable {
        names: vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()],
    };
    let mut annotations: AnnotationRegistry = AnnotationRegistry::new();
    AnnotationRegistryExt::register(&mut annotations, "foo", "Component");
    AnnotationRegistryExt::register(&mut annotations, "bar", "Component");
    AnnotationRegistryExt::register(&mut annotations, "bar", "Primary");

    let ext = ListableBeanFactoryExtensions::new(&factory, &annotations);

    assert!(ext.find_annotation_on_bean("foo", "Component"));
    assert!(!ext.find_annotation_on_bean("foo", "Primary"));
    assert!(ext.find_annotation_on_bean("bar", "Primary"));
    assert!(!ext.find_annotation_on_bean("baz", "Component"));

    let names = ext.get_bean_names_for_annotation("Component");
    assert_eq!(names, vec!["foo".to_owned(), "bar".to_owned()]);

    let filtered = ext.get_bean_names_for_annotation_filtered("Component");
    assert_eq!(filtered, vec!["foo".to_owned(), "bar".to_owned()]);

    assert_eq!(ext.count_beans_with_annotation("Component"), 2);
    assert_eq!(
        ext.get_annotations_on_bean("bar"),
        vec!["Component".to_owned(), "Primary".to_owned()]
    );
    assert!(ext.get_annotations_on_bean("missing").is_empty());
}

#[test]
fn listable_bean_factory_extensions_no_match() {
    let factory = StubListable {
        names: vec!["foo".to_owned()],
    };
    let annotations: AnnotationRegistry = AnnotationRegistry::new();
    let ext = ListableBeanFactoryExtensions::new(&factory, &annotations);
    assert!(!ext.find_annotation_on_bean("foo", "Component"));
    assert!(ext.get_bean_names_for_annotation("Component").is_empty());
    assert_eq!(ext.count_beans_with_annotation("Component"), 0);
}

// ===========================================================================
// 29. bean_factory_dependency_provider.rs - get_dependency
// ===========================================================================

#[test]
fn bean_factory_dependency_provider_resolve_by_type_id() {
    let factory = FailingFactory;
    let provider = BeanFactoryDependencyProvider::new(&factory);
    // resolve_by_type_id forwards call; failing factory returns Err
    let result = provider.resolve_by_type_id(TypeId::of::<u32>());
    assert!(result.is_err());

    // Successful resolution
    let factory_ok = OkFactory::new();
    let provider_ok = BeanFactoryDependencyProvider::new(&factory_ok);
    let bean = provider_ok.resolve_by_type_id(TypeId::of::<u32>()).unwrap();
    let _ = bean.downcast_ref::<u32>().unwrap();
}

#[test]
fn bean_factory_dependency_provider_resolve_via_descriptor() {
    let factory_ok = OkFactory::new();
    let provider = BeanFactoryDependencyProvider::new(&factory_ok);
    let descriptor =
        DependencyDescriptor::for_field(TypeId::of::<u32>(), std::any::type_name::<u32>());
    let result = provider.resolve_dependency(&descriptor).unwrap();
    assert!(result.is_some());

    // owned variant
    let factory_arc: Arc<dyn BeanFactory> = Arc::new(OkFactory::new());
    let owned = OwnedBeanFactoryDependencyProvider::new(factory_arc);
    let r2 = owned.resolve_dependency(&descriptor).unwrap();
    assert!(r2.is_some());

    let _ = provider.bean_factory();
    let _ = owned.bean_factory();
}

// ===========================================================================
// Helpers
// ===========================================================================

struct SampleMetadata {
    annotations: Vec<AnnotationDescriptor>,
}

impl AnnotationMetadata for SampleMetadata {
    fn annotations(&self) -> &[AnnotationDescriptor] {
        &self.annotations
    }
}

struct EmptyMetadata;

impl AnnotationMetadata for EmptyMetadata {
    fn annotations(&self) -> &[AnnotationDescriptor] {
        &[]
    }
}

#[derive(Debug)]
struct DummyBeanReference {
    name: String,
}

impl DummyBeanReference {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

impl BeanReference for DummyBeanReference {
    fn get_bean_name(&self) -> &str {
        &self.name
    }
    fn get_source(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

/// Custom failure-mode helpers for tests that need error propagation.
struct CountingProcessor {
    supported: Vec<String>,
    calls: Mutex<usize>,
}

impl CountingProcessor {
    fn new(supported: Vec<String>) -> Self {
        Self {
            supported,
            calls: Mutex::new(0),
        }
    }

    fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }
}

impl AnnotationProcessor for CountingProcessor {
    fn process(
        &self,
        _annotation_metadata: &dyn AnnotationMetadata,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        *self.calls.lock().unwrap() += 1;
        Ok(())
    }

    fn supported_annotation_types(&self) -> &[String] {
        &self.supported
    }
}

struct SimpleMetadataReaderImpl {
    annotation_metadata: Arc<dyn AnnotationMetadata>,
    class_metadata: Arc<SimpleClassMetadata>,
    resource_description: String,
}

impl SimpleMetadataReaderImpl {
    fn new(
        annotation_metadata: Arc<dyn AnnotationMetadata>,
        class_metadata: Arc<SimpleClassMetadata>,
        resource_description: &str,
    ) -> Self {
        Self {
            annotation_metadata,
            class_metadata,
            resource_description: resource_description.to_owned(),
        }
    }
}

impl MetadataReader for SimpleMetadataReaderImpl {
    fn get_annotation_metadata(&self) -> Arc<dyn AnnotationMetadata> {
        self.annotation_metadata.clone()
    }
    fn get_class_metadata(&self) -> Arc<dyn ClassMetadata> {
        self.class_metadata.clone()
    }
    fn resource_description(&self) -> &str {
        &self.resource_description
    }
}

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
    ) -> Result<
        std::collections::HashMap<String, Arc<dyn Any + Send + Sync>>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        Ok(std::collections::HashMap::new())
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

struct FailingFactory;

impl BeanFactory for FailingFactory {
    fn get_bean_by_key(
        &self,
        _key: &ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not found".into())
    }
    fn get_bean_by_type_id(
        &self,
        _type_id: TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not found".into())
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

struct OkFactory;

impl OkFactory {
    fn new() -> Self {
        Self
    }
}

impl BeanFactory for OkFactory {
    fn get_bean_by_key(
        &self,
        _key: &ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not supported".into())
    }
    fn get_bean_by_type_id(
        &self,
        type_id: TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if type_id == TypeId::of::<u32>() {
            Ok(Arc::new(42u32))
        } else {
            Err("not found".into())
        }
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

struct TestRegistrar {
    name: String,
    recorder: Arc<Mutex<Vec<String>>>,
}

impl TestRegistrar {
    fn new(name: &str, recorder: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            name: name.to_owned(),
            recorder,
        }
    }
}

impl ImportBeanDefinitionRegistrar for TestRegistrar {
    fn register_bean_definitions(
        &self,
        _importing_metadata: &dyn AnnotationMetadata,
        _registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.recorder.lock().unwrap().push(self.name.clone());
        Ok(())
    }
}

struct DummyRegistry {
    store: RwLock<std::collections::HashMap<String, Box<dyn BeanDefinition>>>,
}

impl DummyRegistry {
    fn new() -> Self {
        Self {
            store: RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl BeanDefinitionRegistry for DummyRegistry {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        bean_definition: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.store.write().unwrap().insert(bean_name, bean_definition);
        Ok(())
    }
    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        self.store
            .write()
            .unwrap()
            .remove(bean_name)
            .ok_or_else(|| "not found".into())
    }
    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.store.read().unwrap().contains_key(bean_name)
    }
    fn get_bean_definition(&self, _bean_name: &str) -> Option<&dyn BeanDefinition> {
        // Cannot return reference with the lock guard; always return None.
        None
    }
    fn bean_definition_names(&self) -> Vec<String> {
        self.store.read().unwrap().keys().cloned().collect()
    }
    fn bean_definition_count(&self) -> usize {
        self.store.read().unwrap().len()
    }
}

// ===========================================================================
// Trait helper extension to allow reading BeanMethodDefinition fields uniformly.
// ===========================================================================

trait BeanMethodDefinitionExt {
    fn method_name_unused(&self) -> String;
}

impl BeanMethodDefinitionExt for BeanMethodDefinition {
    fn method_name_unused(&self) -> String {
        self.factory_method.clone()
    }
}

// ===========================================================================
// Defer-counting helpers for tests that need to assert call counts.
// ===========================================================================

trait CountAccess {
    fn calls(&self) -> usize;
}

impl CountAccess for CountingProcessor {
    fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }
}
