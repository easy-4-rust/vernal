//! Resource/XML, Environment, ApplicationContext 系统测试。
use std::sync::Arc;

// ── Resource 系统 ──────────────────────────────────────────────────

#[test]
fn resource_trait_abstract() {
    use vernal_beans::resource::Resource;
    #[derive(Debug)]
struct TestRes;
    impl Resource for TestRes {
        fn description(&self) -> &str { "test" }
    }
    assert!(TestRes.exists());
    assert!(TestRes.is_readable());
}

#[test]
fn abstract_resource_basic() {
    use vernal_beans::abstract_resource::AbstractResource;
    use vernal_beans::resource::Resource;
    let r = AbstractResource::new("test.txt");
    assert!(r.exists());
    assert_eq!(r.description(), "test.txt");
}

#[test]
fn filesystem_resource_basic() {
    use vernal_beans::filesystem_resource::FileSystemResource;
    use vernal_beans::resource::Resource;
    let r = FileSystemResource::new("/tmp/test.txt");
    assert_eq!(r.filename(), Some("test.txt"));
    assert!(!r.exists()); // doesn't exist
}

#[test]
fn classpath_resource_basic() {
    use vernal_beans::classpath_resource::ClassPathResource;
    use vernal_beans::resource::Resource;
    let r = ClassPathResource::new("com/example/config.xml");
    assert_eq!(r.get_path(), "com/example/config.xml");
}

#[test]
fn url_resource_basic() {
    use vernal_beans::url_resource::UrlResource;
    use vernal_beans::resource::Resource;
    let r = UrlResource::new("https://example.com/file.txt");
    assert_eq!(r.url_str(), "https://example.com/file.txt");
    assert_eq!(r.scheme(), Some("https"));
    assert_eq!(r.filename(), Some("file.txt"));
}

#[test]
fn input_stream_resource_basic() {
    use vernal_beans::input_stream_resource::InputStreamResource;
    use vernal_beans::resource::Resource;
    let r = InputStreamResource::new(vec![1, 2, 3], "stream");
    assert!(r.exists());
    assert_eq!(r.content_len(), 3);
    assert!(r.is_open());
    assert_eq!(r.read_to_bytes().unwrap(), vec![1, 2, 3]);
}

#[test]
fn descriptive_resource_basic() {
    use vernal_beans::descriptive_resource::DescriptiveResource;
    use vernal_beans::resource::Resource;
    let r = DescriptiveResource::new("test", "A test resource");
    assert_eq!(r.description(), "A test resource");
}

// ── DefaultResourceLoader ──────────────────────────────────────────

#[test]
fn default_resource_loader_basic() {
    use vernal_beans::default_resource_loader::{DefaultResourceLoader, ResourceLoader};
    let loader = DefaultResourceLoader::new();
    let resource = loader.get_resource("classpath:com/example/test.xml").unwrap();
    assert!(resource.description().contains("com/example/test.xml"));
}

#[test]
fn default_resource_loader_file_prefix() {
    use vernal_beans::default_resource_loader::{DefaultResourceLoader, ResourceLoader};
    let loader = DefaultResourceLoader::new();
    let resource = loader.get_resource("file:/tmp/test.txt").unwrap();
    assert!(resource.description().contains("test.txt"));
}

// ── Environment 系统 ─────────────────────────────────────────────

#[test]
fn property_source_basic() {
    use vernal_beans::property_source::PropertySource;
    let mut ps = PropertySource::new("test");
    ps.set("key1", "value1");
    assert_eq!(ps.get_property("key1"), Some("value1"));
    assert!(ps.contains_property("key1"));
    assert!(!ps.contains_property("nonexistent"));
    assert_eq!(ps.len(), 1);
}

#[test]
fn property_sources_basic() {
    use vernal_beans::property_source::PropertySource;
    use vernal_beans::property_sources::PropertySources;
    let mut sources = PropertySources::new();
    let mut ps = PropertySource::new("env");
    ps.set("KEY", "VALUE");
    sources.add(ps);
    assert_eq!(sources.len(), 1);
    assert!(sources.contains("env"));
}

#[test]
fn placeholder_resolver_basic() {
    use vernal_beans::placeholder_resolver::PlaceholderResolver;
    use vernal_beans::property_source::PropertySource;
    let mut ps = PropertySource::new("props");
    ps.set("name", "world");
    let mut resolver = PlaceholderResolver::new();
    resolver.add_source(ps);
    let result = resolver.resolve("Hello, ${name}!").unwrap();
    assert_eq!(result, "Hello, world!");
}

#[test]
fn profile_new() {
    use vernal_beans::profile::Profile;
    let p = Profile::new("dev");
    assert_eq!(p.name(), "dev");
    assert!(!p.is_default());
    let dp = Profile::new_default("default");
    assert!(dp.is_default());
}

#[test]
fn profiles_basic() {
    use vernal_beans::profiles::Profiles;
    use vernal_beans::profile::Profile;
    let mut ps = Profiles::new();
    ps.add(Profile::new("dev"));
    assert!(ps.is_active("dev"));
    assert!(!ps.is_active("prod"));
    ps.remove("dev");
    assert!(!ps.is_active("dev"));
}

// ── ApplicationContext 系统 ────────────────────────────────────────

#[test]
fn generic_application_event_basic() {
    use vernal_beans::application_event::GenericApplicationEvent;
    let e = GenericApplicationEvent::new();
    assert!(e.timestamp > 0);
}

// ── Document loader ────────────────────────────────────────────────

#[test]
fn document_loader_basic() {
    use vernal_beans::document_loader::DocumentLoader;
    use vernal_beans::default_document_loader::DefaultDocumentLoader;
    let loader = DefaultDocumentLoader::new();
    let doc = loader.load_document("<beans/>").unwrap();
    assert!(doc.root.is_some());
    assert_eq!(doc.encoding, Some("UTF-8".to_string()));
}

// ── Entity resolver ─────────────────────────────────────────────────

#[test]
fn dtd_resolver_basic() {
    use vernal_beans::entity_resolver::EntityResolver;
    use vernal_beans::dtd_resolver::DtdResolver;
    let resolver = DtdResolver::new();
    assert!(resolver.resolve_entity("public", "system").is_none());
}

#[test]
fn pluggable_schema_resolver_basic() {
    use vernal_beans::entity_resolver::EntityResolver;
    use vernal_beans::pluggable_schema_resolver::PluggableSchemaResolver;
    let resolver = PluggableSchemaResolver::from_text("http://example.com/schema.xsd=/local/schema.xsd");
    let entity = resolver.resolve_entity("", "http://example.com/schema.xsd");
    assert!(entity.is_some());
}

// ── Protocol resolver ───────────────────────────────────────────────

#[test]
fn closure_protocol_resolver_basic() {
    use vernal_beans::protocol_resolver::{ClosureProtocolResolver, ProtocolResolver};
    let resolver = ClosureProtocolResolver::new(Box::new(|proto: &str, _loc: &str| -> Option<Box<dyn vernal_beans::resource::Resource>> {
        if proto == "custom" {
            Some(Box::new(vernal_beans::descriptive_resource::DescriptiveResource::new("test", "test")))
        } else {
            None
        }
    }));
    assert!(resolver.resolve("custom", "location").is_some());
    assert!(resolver.resolve("unknown", "location").is_none());
}
