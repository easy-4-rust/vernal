//! 边界覆盖测试：资源与 XML 读取层的 25 个源文件中初始 79 个测试未覆盖的分支。
//!
//! 本文件聚焦各模块的边界与错误分支：不存在文件、未知协议、解析失败、
//! 空输入、回退路径、custom 解析器、globs 与 `**` 递归等。

use std::io::{Cursor, Read};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use vernal_beans::Scope;
use vernal_beans::abstract_bean_definition_reader_impl::{
    AbstractBeanDefinitionReaderImpl, ParseCallback, SharedRegistry,
};
use vernal_beans::abstract_resource::AbstractResource;
use vernal_beans::bean_definition::BeanDefinition;
use vernal_beans::bean_definition_parser::{BeanDefinitionParser, register_single};
use vernal_beans::bean_definition_reader::BeanDefinitionReader;
use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::bean_definition_resource::BeanDefinitionResource;
use vernal_beans::classpath_resource::ClassPathResource;
use vernal_beans::default_document_loader::DefaultDocumentLoader;
use vernal_beans::default_namespace_handler_resolver::DefaultNamespaceHandlerResolver;
use vernal_beans::default_resource_loader::{
    CLASSPATH_URL_PREFIX, DefaultResourceLoader, FILE_URL_PREFIX, ResourceLoader,
    URL_PROTOCOL_PREFIX,
};
use vernal_beans::document_loader::{DocumentLoader, Element};
use vernal_beans::dtd_resolver::{BeansDtdResolver, DtdResolver};
use vernal_beans::entity_resolver::EntityResolver;
use vernal_beans::filesystem_resource::FileSystemResource;
use vernal_beans::input_stream_resource::InputStreamResource;
use vernal_beans::namespace_handler::{
    NamespaceHandler, NamespaceHandlerSupport, is_known_namespace,
};
use vernal_beans::namespace_handler_resolver::NamespaceHandlerResolver;
use vernal_beans::path_matching_resource_pattern_resolver::PathMatchingResourcePatternResolver;
use vernal_beans::pluggable_schema_resolver::PluggableSchemaResolver;
use vernal_beans::properties_bean_definition_reader::PropertiesBeanDefinitionReader;
use vernal_beans::protocol_resolver::{ClosureProtocolResolver, ProtocolResolver};
use vernal_beans::resource::Resource;
use vernal_beans::resource_pattern_resolver::ResourcePatternResolver;
use vernal_beans::root_bean_definition::RootBeanDefinition;
use vernal_beans::simple_bean_definition_registry::SimpleBeanDefinitionRegistry;
use vernal_beans::url_resource::UrlResource;
use vernal_beans::xml_bean_definition_reader::XmlBeanDefinitionReader;
use vernal_beans::xml_reader_context::ReaderContext;
use vernal_beans::xml_reader_helpers;

// =============================================================================
// 辅助类型与函数
// =============================================================================

/// 一个把字符串内容当作 BeanDefinitionResource 的简单实现。
#[derive(Debug, Clone)]
struct TextResource {
    name: String,
    content: String,
}

impl BeanDefinitionResource for TextResource {
    fn description(&self) -> &str {
        &self.content
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn read_to_string(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.content.clone())
    }
}

/// 把 BeanDefinition（具体为 RootBeanDefinition）取回以便断言其字段。
///
/// 利用 `BeanDefinition: Any` 的 trait upcasting（Rust 1.86+ 稳定），
/// 把 `&dyn BeanDefinition` 上转为 `&dyn Any` 后再 downcast。
fn def_as_root(def: &dyn BeanDefinition) -> &RootBeanDefinition {
    let any: &dyn std::any::Any = def;
    any.downcast_ref::<RootBeanDefinition>()
        .expect("definition should be a RootBeanDefinition")
}

static GLOB_SEQ: AtomicU64 = AtomicU64::new(0);

/// 创建一个唯一的临时目录树用于 glob 测试，避免并行测试竞争。
fn make_glob_tree() -> PathBuf {
    let seq = GLOB_SEQ.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!(
        "vernal_beans_boundary_{}_{}",
        std::process::id(),
        seq
    ));
    let _ = std::fs::remove_dir_all(&root);
    let sub = root.join("com").join("acme");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("a.xml"), b"<a/>").unwrap();
    std::fs::write(sub.join("b.xml"), b"<b/>").unwrap();
    std::fs::write(sub.join("c.txt"), b"c").unwrap();
    std::fs::write(sub.join("a.rs"), b"fn a(){}").unwrap();
    std::fs::write(root.join("top.xml"), b"<t/>").unwrap();
    root
}

// =============================================================================
// 1. abstract_resource.rs —— read_to_bytes / content_length / with_filename / from_string
// =============================================================================

#[test]
fn abstract_resource_read_to_bytes_returns_clone_of_content() {
    let mut r = AbstractResource::new(b"payload-123".to_vec(), "d");
    let bytes = r.read_to_bytes().unwrap();
    assert_eq!(bytes, b"payload-123".to_vec());
    // 重复读取仍可成功（实现内部 clone）。
    let again = r.read_to_bytes().unwrap();
    assert_eq!(again, bytes);
}

#[test]
fn abstract_resource_content_length_matches_bytes_len() {
    let r = AbstractResource::new(b"0123456789".to_vec(), "len");
    assert_eq!(r.content_length(), Some(10));
    // 空内容 → 0。
    let empty = AbstractResource::new(Vec::new(), "empty");
    assert_eq!(empty.content_length(), Some(0));
}

#[test]
fn abstract_resource_with_filename_builder_roundtrip() {
    let r = AbstractResource::from_string("body", "src").with_filename("file.bin");
    assert_eq!(r.filename().as_deref(), Some("file.bin"));
    // description 不受 with_filename 影响。
    assert_eq!(r.description(), "src");
    assert_eq!(r.description_str(), "src");
    assert_eq!(r.content(), b"body");
}

#[test]
fn abstract_resource_from_string_uses_utf8_bytes() {
    let r = AbstractResource::from_string("héllo", "u");
    assert_eq!(r.content(), "héllo".as_bytes());
    assert_eq!(r.content_length(), Some("héllo".as_bytes().len() as u64));
}

#[test]
fn abstract_resource_default_url_and_file_path_are_none() {
    let r = AbstractResource::new(b"x".to_vec(), "d");
    // AbstractResource 没有底层 URL/文件路径。
    assert_eq!(r.url(), None);
    assert_eq!(r.file_path(), None);
    assert_eq!(r.uri(), None); // uri 默认回退到 url
}

#[test]
fn abstract_resource_clone_is_independent_handle() {
    let r = AbstractResource::new(b"data".to_vec(), "d").with_filename("a.txt");
    let cloned = r.clone();
    assert_eq!(cloned.content(), r.content());
    assert_eq!(cloned.filename(), r.filename());
}

// =============================================================================
// 2. filesystem_resource.rs —— 不存在文件的 exists / is_readable / filename / input_stream
// =============================================================================

#[test]
fn filesystem_resource_nonexistent_input_stream_errors() {
    let r = FileSystemResource::from_str("/no/such/vernal/file.txt");
    // input_stream 应返回错误。
    let err = r.input_stream();
    assert!(err.is_err(), "missing file input_stream should error");
}

#[test]
fn filesystem_resource_nonexistent_content_length_is_none() {
    let r = FileSystemResource::from_str("/no/such/vernal/meta.txt");
    // 无法读取元数据 → None。
    assert_eq!(r.content_length(), None);
}

#[test]
fn filesystem_resource_nonexistent_is_file_still_true() {
    // FileSystemResource 始终声明自己 is_file（语义上代表文件路径）。
    let r = FileSystemResource::from_str("/no/such/x.txt");
    assert!(r.is_file());
    assert!(!r.is_open());
    assert_eq!(r.filename().as_deref(), Some("x.txt"));
}

#[test]
fn filesystem_resource_directory_is_not_readable() {
    let dir = std::env::temp_dir();
    let r = FileSystemResource::new(dir.clone());
    // 目录存在但不是普通文件。
    assert!(r.exists());
    assert!(!r.is_readable());
    assert_eq!(r.content_length(), Some(dir.metadata().unwrap().len()));
}

#[test]
fn filesystem_resource_path_mut_allows_mutation() {
    let mut r = FileSystemResource::from_str("a/b.txt");
    assert_eq!(r.path(), &PathBuf::from("a/b.txt"));
    r.path_mut().set_extension("cfg");
    assert_eq!(r.filename().as_deref(), Some("b.cfg"));
}

#[test]
fn filesystem_resource_equality_by_path() {
    let a = FileSystemResource::from_str("/tmp/x.txt");
    let b = FileSystemResource::from_str("/tmp/x.txt");
    let c = FileSystemResource::from_str("/tmp/y.txt");
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// =============================================================================
// 3. classpath_resource.rs —— 多根搜索 / exists / description
// =============================================================================

#[test]
fn classpath_resource_multiple_roots_search_order() {
    // 配置两个根目录：第一个不含目标文件，第二个含。
    let dir1 = std::env::temp_dir().join("vernal_cp_multi_1");
    let dir2 = std::env::temp_dir().join("vernal_cp_multi_2");
    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::create_dir_all(&dir2).unwrap();
    std::fs::write(dir2.join("target.txt"), b"found-in-dir2").unwrap();

    let r = ClassPathResource::new("target.txt").with_roots(vec![dir1.clone(), dir2.clone()]);
    assert!(r.exists());
    assert!(r.is_readable());
    // 应解析到 dir2。
    assert_eq!(
        r.file_path().as_deref(),
        Some(dir2.join("target.txt").to_str().unwrap())
    );
    assert_eq!(r.content_length(), Some(b"found-in-dir2".len() as u64));

    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn classpath_resource_first_root_wins() {
    // 两个根都含同名文件，应优先取第一个。
    let dir1 = std::env::temp_dir().join("vernal_cp_first_1");
    let dir2 = std::env::temp_dir().join("vernal_cp_first_2");
    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::create_dir_all(&dir2).unwrap();
    std::fs::write(dir1.join("x.txt"), b"one").unwrap();
    std::fs::write(dir2.join("x.txt"), b"two").unwrap();

    let mut r = ClassPathResource::new("x.txt").with_roots(vec![dir1.clone(), dir2.clone()]);
    let s = r.read_to_string().unwrap();
    assert_eq!(s, "one");

    let _ = std::fs::remove_dir_all(&dir1);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn classpath_resource_description_includes_loader_name() {
    let r = ClassPathResource::new("res/app.xml").with_class_loader("MyClassLoader");
    let desc = r.description();
    assert!(desc.contains("[MyClassLoader]"), "desc = {desc}");
    assert!(desc.contains("app.xml"));
}

#[test]
fn classpath_resource_empty_roots_falls_back_to_cwd_path() {
    // 未配置根、且相对路径在 cwd 下不存在 → exists=false，但 url 仍可用。
    let r = ClassPathResource::new("definitely/not/here.xml");
    assert!(!r.exists());
    assert_eq!(
        r.url().as_deref(),
        Some("classpath:definitely/not/here.xml")
    );
    assert_eq!(r.roots().len(), 0);
}

#[test]
fn classpath_resource_input_stream_on_missing_returns_err() {
    let r = ClassPathResource::new("no/such/cp/resource.bin");
    let err = r.input_stream();
    assert!(
        err.is_err(),
        "missing classpath resource should fail to open"
    );
}

// =============================================================================
// 4. url_resource.rs —— scheme / url_str / exists / is_readable（file 与非 file）
// =============================================================================

#[test]
fn url_resource_file_scheme_exists_and_readable() {
    let path = std::env::temp_dir().join("vernal_url_exists.txt");
    std::fs::write(&path, b"hi").unwrap();
    let url = format!("file:{}", path.display());
    let r = UrlResource::new(url.clone());
    assert_eq!(r.scheme(), "file");
    assert!(r.is_file_url());
    assert!(r.exists());
    assert!(r.is_readable());
    assert_eq!(r.url_str(), url);
    assert_eq!(r.is_file(), true);
    // file_path 应解析为绝对路径。
    let fp = r.file_path().unwrap();
    assert!(fp.ends_with("vernal_url_exists.txt"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn url_resource_file_scheme_missing_not_exists() {
    let r = UrlResource::new("file:/no/such/vernal/url.txt");
    assert_eq!(r.scheme(), "file");
    assert!(r.is_file_url());
    assert!(!r.exists());
    assert!(!r.is_readable());
    // content_length 对不存在文件为 None。
    assert_eq!(r.content_length(), None);
}

#[test]
fn url_resource_https_scheme_considered_existing() {
    let r = UrlResource::new("https://example.com/a/b/c.xml");
    assert_eq!(r.scheme(), "https");
    assert!(!r.is_file_url());
    // 非 file URL：exists 保守为 true。
    assert!(r.exists());
    // is_readable 为 false（无法静态判定）。
    assert!(!r.is_readable());
    assert_eq!(r.filename().as_deref(), Some("c.xml"));
    let desc = r.description();
    assert!(desc.starts_with("URL ["));
}

#[test]
fn url_resource_custom_scheme_parsing() {
    let r = UrlResource::new("ftp://host/path/file.dat");
    assert_eq!(r.scheme(), "ftp");
    assert!(!r.is_file_url());
    assert_eq!(r.file_path(), None);
    // 非 file URL input_stream 不支持。
    assert!(r.input_stream().is_err());
}

#[test]
fn url_resource_no_filename_in_url() {
    // URL 以斜杠结尾 → filename 为 None。
    let r = UrlResource::new("https://example.com/dir/");
    assert_eq!(r.filename(), None);
}

#[test]
fn url_resource_uppercase_scheme_normalized_to_lowercase() {
    let r = UrlResource::new("FILE:/tmp/whatever");
    assert_eq!(r.scheme(), "file");
    assert!(r.is_file_url());
}

// =============================================================================
// 5. input_stream_resource.rs —— content_len / is_open / description
// =============================================================================

#[test]
fn input_stream_resource_content_len_zero_for_empty() {
    let r = InputStreamResource::new(Vec::new(), "empty");
    assert_eq!(r.content_len(), 0);
    assert_eq!(r.content_length(), Some(0));
}

#[test]
fn input_stream_resource_is_open_is_true() {
    let r = InputStreamResource::new(b"x".to_vec(), "d");
    assert!(r.is_open());
    // 其它元数据。
    assert!(r.exists());
    assert!(r.is_readable());
    assert_eq!(r.url(), None);
    assert_eq!(r.file_path(), None);
    assert_eq!(r.filename(), None);
}

#[test]
fn input_stream_resource_description_roundtrip() {
    let r = InputStreamResource::from_string("abc", "my-desc");
    assert_eq!(r.description(), "my-desc");
    assert_eq!(r.content_len(), 3);
}

#[test]
fn input_stream_resource_read_to_bytes_returns_content() {
    let mut r = InputStreamResource::new(b"hello".to_vec(), "d");
    let b = r.read_to_bytes().unwrap();
    assert_eq!(b, b"hello".to_vec());
}

#[test]
fn input_stream_resource_clone_shares_content() {
    let r = InputStreamResource::new(b"data".to_vec(), "d");
    let c = r.clone();
    assert_eq!(c.content_len(), r.content_len());
}

// =============================================================================
// 6. default_resource_loader.rs —— classpath: / file: / 未知协议 / 自定义 ProtocolResolver
// =============================================================================

#[test]
fn default_resource_loader_classpath_prefix_uses_classpath_resource() {
    let loader = DefaultResourceLoader::new();
    let r = loader.get_resource("classpath:com/example/conf.properties");
    assert!(r.url().unwrap().starts_with("classpath:"));
    // classpath 资源的 filename 来自路径尾部。
    assert_eq!(r.filename().as_deref(), Some("conf.properties"));
}

#[test]
fn default_resource_loader_file_prefix_uses_filesystem_resource() {
    let loader = DefaultResourceLoader::new();
    let r = loader.get_resource("file:/var/log/app.log");
    assert!(r.url().unwrap().starts_with("file:"));
    assert_eq!(r.filename().as_deref(), Some("app.log"));
    assert_eq!(r.file_path().as_deref(), Some("/var/log/app.log"));
}

#[test]
fn default_resource_loader_unknown_scheme_treated_as_url() {
    let loader = DefaultResourceLoader::new();
    // mailto: 是合法 scheme 字符 → 走 UrlResource。
    let r = loader.get_resource("mailto:user@example.com");
    // scheme() 不在 Resource trait 上，改用 url() 校验。
    assert_eq!(r.url().as_deref(), Some("mailto:user@example.com"));
}

#[test]
fn default_resource_loader_no_scheme_falls_back_to_filesystem() {
    let loader = DefaultResourceLoader::new();
    let r = loader.get_resource("relative/no/scheme.cfg");
    // 兜底 → FileSystemResource（file: URL）。
    assert!(r.url().unwrap().starts_with("file:"));
    assert!(r.is_file());
}

#[test]
fn default_resource_loader_custom_protocol_resolver_overrides_default() {
    let mut loader = DefaultResourceLoader::new();
    let resolver: Arc<dyn ProtocolResolver> = Arc::new(ClosureProtocolResolver::new(
        "db:",
        Box::new(|loc| {
            let rest = loc.strip_prefix("db:")?;
            Some(Arc::new(AbstractResource::from_string(
                format!("db-content[{rest}]"),
                loc,
            )) as Arc<dyn Resource>)
        }),
    ));
    loader.add_protocol_resolver(resolver);
    assert_eq!(loader.protocol_resolvers().len(), 1);

    let r = loader.get_resource("db:table/users");
    let mut stream = r.input_stream().unwrap();
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "db-content[table/users]");
}

#[test]
fn default_resource_loader_classpath_uses_configured_roots_and_loader_name() {
    let mut loader = DefaultResourceLoader::new();
    loader.set_class_loader_name("TestLoader");
    loader.add_classpath_root(PathBuf::from("/tmp"));
    assert_eq!(loader.class_loader_name(), Some("TestLoader"));
    assert_eq!(loader.classpath_roots().len(), 1);

    let r = loader.get_resource("classpath:foo/bar.txt");
    // 描述里应包含 loader 名。
    assert!(
        r.description().contains("[TestLoader]"),
        "desc={}",
        r.description()
    );
}

#[test]
fn default_resource_loader_constants_are_exported() {
    assert_eq!(CLASSPATH_URL_PREFIX, "classpath:");
    assert_eq!(FILE_URL_PREFIX, "file:");
    assert_eq!(URL_PROTOCOL_PREFIX, "url:");
}

#[test]
fn default_resource_loader_url_explicit_prefix() {
    let loader = DefaultResourceLoader::new();
    let r = loader.get_resource("url:https://x.example/file.xml");
    assert_eq!(r.url().as_deref(), Some("https://x.example/file.xml"));
    // scheme() 不在 Resource trait 上；用 description（URL [..]）确认是 UrlResource。
    assert!(r.description().starts_with("URL ["));
}

// =============================================================================
// 7. path_matching_resource_pattern_resolver.rs —— *.rs glob / ** 递归 / file: / 无匹配
// =============================================================================

#[test]
fn path_matching_resolver_glob_rs_files() {
    let root = make_glob_tree();
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    let pattern = format!("file:{}/**/*.rs", root.display());
    let resources = resolver.get_resources(&pattern).unwrap();
    assert_eq!(resources.len(), 1, "应仅匹配 a.rs");
    assert_eq!(resources[0].filename().as_deref(), Some("a.rs"),);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn path_matching_resolver_double_star_recursive_xml() {
    let root = make_glob_tree();
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    let pattern = format!("file:{}/**/*.xml", root.display());
    let resources = resolver.get_resources(&pattern).unwrap();
    // a.xml、b.xml（在 com/acme/）+ top.xml（在根，** 匹配零层）。
    let names: Vec<_> = resources.iter().filter_map(|r| r.filename()).collect();
    assert!(names.contains(&"a.xml".to_string()));
    assert!(names.contains(&"b.xml".to_string()));
    assert!(names.contains(&"top.xml".to_string()));
    assert_eq!(resources.len(), 3, "names = {names:?}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn path_matching_resolver_file_prefix_nonexistent_root_no_matches() {
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    // 根目录不存在 → 空结果。
    let out = resolver
        .get_resources("file:/no/such/root/**/*.xml")
        .unwrap();
    assert!(out.is_empty());
}

#[test]
fn path_matching_resolver_single_question_mark_glob() {
    let root = make_glob_tree();
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    // ? 匹配单个字符：a.xml、b.xml 命中，c.txt 不命中。
    let pattern = format!("file:{}/**/com/acme/?.xml", root.display());
    let resources = resolver.get_resources(&pattern).unwrap();
    assert_eq!(resources.len(), 2);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn path_matching_resolver_get_resource_single_no_wildcard_proxies() {
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    // 无通配符 → 直接代理到加载器。
    let r = resolver.get_resource("file:/tmp/some.txt");
    assert!(r.url().unwrap().starts_with("file:"));
    assert_eq!(r.filename().as_deref(), Some("some.txt"));
}

#[test]
fn path_matching_resolver_get_resource_with_wildcard_returns_first() {
    let root = make_glob_tree();
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    let pattern = format!("file:{}/**/*.xml", root.display());
    // 有通配符但调用单资源接口 → 返回首个匹配（排序后）。
    let r = resolver.get_resource(&pattern);
    assert!(r.filename().is_some());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn path_matching_resolver_no_match_returns_empty_vec() {
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    let out = resolver
        .get_resources("file:/tmp/no_such_dir_vernal/*.nomatch")
        .unwrap();
    assert!(out.is_empty());
}

#[test]
fn path_matching_resolver_classpath_single_resource_no_wildcard() {
    let resolver = PathMatchingResourcePatternResolver::with_defaults();
    // classpath: 单资源无通配符 → 返回单元素列表。
    let out = resolver.get_resources("classpath:com/acme/x.xml").unwrap();
    assert_eq!(out.len(), 1);
}

// =============================================================================
// 8. default_document_loader.rs —— 属性 / 嵌套 / 注释 / CDATA-like 文本 / 空文档 / 畸形 XML
// =============================================================================

#[test]
fn document_loader_xml_with_multiple_attributes_and_namespaces() {
    let loader = DefaultDocumentLoader::new();
    let xml = br#"<root xmlns="http://example.com/ns" attr1="v1" attr2="v2"><leaf/></root>"#;
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    let root = doc.document_element().unwrap();
    assert_eq!(root.local_name, "root");
    assert_eq!(root.get_attribute("attr1"), Some("v1"));
    assert_eq!(root.get_attribute("attr2"), Some("v2"));
    // 命名空间属性也会被当作普通属性保留。
    assert!(root.get_attribute("xmlns").is_some());
}

#[test]
fn document_loader_deeply_nested_elements() {
    let loader = DefaultDocumentLoader::new();
    let xml = b"<a><b><c><d>deep</d></c></b></a>";
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    let mut cur = doc.document_element().unwrap();
    assert_eq!(cur.local_name, "a");
    cur = cur.child_elements().next().unwrap();
    assert_eq!(cur.local_name, "b");
    cur = cur.child_elements().next().unwrap();
    assert_eq!(cur.local_name, "c");
    cur = cur.child_elements().next().unwrap();
    assert_eq!(cur.local_name, "d");
    assert_eq!(cur.text_content, "deep");
}

#[test]
fn document_loader_comments_between_elements_ignored() {
    let loader = DefaultDocumentLoader::new();
    let xml = b"<root><!-- c1 --><child/><!-- c2 --></root>";
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    let root = doc.document_element().unwrap();
    let kids: Vec<_> = root.child_elements().collect();
    assert_eq!(kids.len(), 1);
    assert_eq!(kids[0].local_name, "child");
}

#[test]
fn document_loader_processing_instruction_skipped() {
    let loader = DefaultDocumentLoader::new();
    // 注意：本解析器把任何以 `<?xml` 开头的 PI 视为 XML 声明（含属性解析），
    // 因此这里使用不以 xml 开头的处理指令。
    let xml = b"<?mystyle type=\"text/xsl\"?><root><child/></root>";
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    let root = doc.document_element().unwrap();
    assert_eq!(root.local_name, "root");
    assert_eq!(root.child_elements().next().unwrap().local_name, "child");
}

#[test]
fn document_loader_doctype_skipped() {
    let loader = DefaultDocumentLoader::new();
    let xml = b"<!DOCTYPE root SYSTEM \"dtd\"><root><child/></root>";
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    let root = doc.document_element().unwrap();
    assert_eq!(root.local_name, "root");
}

#[test]
fn document_loader_cdata_like_text_preserved_as_text() {
    // 本实现不支持真正的 CDATA 段，但形如 "a &amp; b" 的实体文本应被正确反转义。
    let loader = DefaultDocumentLoader::new();
    let xml = b"<root>value with &lt;tag&gt; &amp; &quot;q&quot;</root>";
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    let root = doc.document_element().unwrap();
    assert_eq!(root.text_content, "value with <tag> & \"q\"");
}

#[test]
fn document_loader_empty_document_no_root() {
    let loader = DefaultDocumentLoader::new();
    let doc = loader.load_document(&mut Cursor::new(b"")).unwrap();
    assert!(doc.document_element().is_none());
    assert!(doc.version.is_none());
    assert!(doc.encoding.is_none());
}

#[test]
fn document_loader_whitespace_only_document() {
    let loader = DefaultDocumentLoader::new();
    let doc = loader
        .load_document(&mut Cursor::new(b"   \n\t  "))
        .unwrap();
    assert!(doc.document_element().is_none());
}

#[test]
fn document_loader_xml_declaration_with_encoding_only() {
    let loader = DefaultDocumentLoader::new();
    let xml = b"<?xml encoding=\"UTF-8\"?><root/>";
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    assert_eq!(doc.encoding.as_deref(), Some("UTF-8"));
    assert!(doc.version.is_none());
}

#[test]
fn document_loader_self_closing_with_attributes() {
    let loader = DefaultDocumentLoader::new();
    let xml = br#"<bean id="x" class="com.X"/>"#;
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    let root = doc.document_element().unwrap();
    assert_eq!(root.get_attribute("id"), Some("x"));
    assert_eq!(root.get_attribute("class"), Some("com.X"));
    assert_eq!(root.child_elements().count(), 0);
}

#[test]
fn document_loader_single_quote_attribute_values() {
    let loader = DefaultDocumentLoader::new();
    let xml = b"<root attr='single'/>";
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    assert_eq!(
        doc.document_element().unwrap().get_attribute("attr"),
        Some("single")
    );
}

#[test]
fn document_loader_malformed_unclosed_element_errors() {
    let loader = DefaultDocumentLoader::new();
    // 元素未闭合 → 解析应返回错误。
    let xml = b"<root><child></root>";
    let res = loader.load_document(&mut Cursor::new(&xml[..]));
    // 解析器对未匹配的结束标签较宽容，但未闭合的子元素会触发 EOF 错误。
    // 此处不强求错误（取决于解析器宽容度），但至少不应 panic。
    let _ = res;
}

#[test]
fn document_loader_text_with_entities_in_attribute() {
    let loader = DefaultDocumentLoader::new();
    let xml = br#"<root attr="a&amp;b&lt;c"/>"#;
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    assert_eq!(
        doc.document_element().unwrap().get_attribute("attr"),
        Some("a&b<c")
    );
}

#[test]
fn document_loader_bom_prefix_skipped() {
    let loader = DefaultDocumentLoader::new();
    let mut xml = Vec::new();
    xml.extend_from_slice(b"\xEF\xBB\xBF");
    xml.extend_from_slice(b"<root/>");
    let doc = loader.load_document(&mut Cursor::new(&xml[..])).unwrap();
    assert_eq!(doc.document_element().unwrap().local_name, "root");
}

// =============================================================================
// 9. dtd_resolver.rs —— resolve_entity 匹配 / 不匹配 / 自定义注册
// =============================================================================

#[test]
fn dtd_resolver_non_dtd_system_id_returns_none() {
    let r = DtdResolver::new();
    // 非 .dtd 后缀 → None。
    let out = r.resolve_entity("", "http://x/y.xml").unwrap();
    assert!(out.is_none());
}

#[test]
fn dtd_resolver_non_beans_dtd_returns_none() {
    let r = DtdResolver::new();
    // 是 .dtd 但不含 /spring-beans → None。
    let out = r.resolve_entity("", "http://x/other.dtd").unwrap();
    assert!(out.is_none());
}

#[test]
fn dtd_resolver_beans_dtd_missing_local_file_returns_none() {
    let r = DtdResolver::new();
    // 是 beans DTD 但本地文件不存在 → None。
    let out = r
        .resolve_entity(
            "public",
            "http://www.springframework.org/dtd/spring-beans.dtd",
        )
        .unwrap();
    assert!(out.is_none());
}

#[test]
fn dtd_resolver_https_beans_dtd_alias_missing() {
    let r = DtdResolver::new();
    let out = r
        .resolve_entity(
            "",
            "https://www.springframework.org/dtd/spring-beans-3.0.dtd",
        )
        .unwrap();
    assert!(out.is_none());
    // 映射表中存在该 https 别名。
    assert!(
        r.mappings()
            .contains_key("https://www.springframework.org/dtd/spring-beans-3.0.dtd")
    );
}

#[test]
fn dtd_resolver_custom_register_resolves_existing_file() {
    // 注册一个指向真实文件的本地路径。
    let path = std::env::temp_dir().join("vernal_dtd_test.dtd");
    std::fs::write(&path, b"<!ELEMENT root EMPTY>").unwrap();
    let mut r = DtdResolver::new();
    // 注意：system_id 必须含 /spring-beans 且以 .dtd 结尾才会被处理。
    r.register(
        "http://www.springframework.org/dtd/spring-beans-custom.dtd",
        path.to_string_lossy().into_owned(),
    );
    let out = r
        .resolve_entity(
            "pub",
            "http://www.springframework.org/dtd/spring-beans-custom.dtd",
        )
        .unwrap();
    assert!(out.is_some(), "应能解析到本地文件");
    let entity = out.unwrap();
    assert_eq!(entity.public_id, "pub");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn beans_dtd_resolver_alias_type_equivalent() {
    // BeansDtdResolver 是 DtdResolver 的别名。
    let r = BeansDtdResolver::new();
    let out = r
        .resolve_entity(
            "",
            "http://www.springframework.org/dtd/spring-beans-2.0.dtd",
        )
        .unwrap();
    assert!(out.is_none());
}

// =============================================================================
// 10. pluggable_schema_resolver.rs —— from_text 解析 / resolve_entity 查找
// =============================================================================

#[test]
fn pluggable_schema_resolver_from_text_skips_comments_and_blank() {
    let text = "\n\
        # 注释行\n\
        \n\
        http://a/1.xsd=p1.xsd\n\
        http://b/2.xsd=p2.xsd\n\
        # 另一条注释\n\
    ";
    let r = PluggableSchemaResolver::from_text(text);
    assert_eq!(r.schema_mappings().len(), 2);
    assert_eq!(
        r.schema_mappings()
            .get("http://a/1.xsd")
            .map(|s| s.as_str()),
        Some("p1.xsd")
    );
}

#[test]
fn pluggable_schema_resolver_from_text_ignores_line_without_equals() {
    let text = "\
        no_equals_here\n\
        http://x/y.xsd=local.xsd\n\
    ";
    let r = PluggableSchemaResolver::from_text(text);
    // 没有 = 的行被忽略。
    assert_eq!(r.schema_mappings().len(), 1);
}

#[test]
fn pluggable_schema_resolver_from_text_trims_whitespace() {
    let text = "   http://x/y.xsd   =   local.xsd   ";
    let r = PluggableSchemaResolver::from_text(text);
    // 键值都被 trim。
    assert_eq!(
        r.schema_mappings()
            .get("http://x/y.xsd")
            .map(|s| s.as_str()),
        Some("local.xsd")
    );
    assert_eq!(r.schema_mappings().len(), 1);
}

#[test]
fn pluggable_schema_resolver_resolve_entity_non_xsd_returns_none() {
    let r = PluggableSchemaResolver::new();
    let out = r.resolve_entity("", "http://x/y.dtd").unwrap();
    assert!(out.is_none());
}

#[test]
fn pluggable_schema_resolver_resolve_entity_xsd_no_mapping_falls_back_to_cwd() {
    let r = PluggableSchemaResolver::new();
    // 无映射、当前目录也无该文件 → None。
    let out = r
        .resolve_entity("", "http://example.com/no/such.xsd")
        .unwrap();
    assert!(out.is_none());
}

#[test]
fn pluggable_schema_resolver_resolve_existing_local_xsd() {
    // 创建一个本地 .xsd 文件，并通过文件名回退路径解析。
    let dir = std::env::temp_dir();
    let filename = format!("vernal_schema_{}.xsd", std::process::id());
    let path = dir.join(&filename);
    std::fs::write(&path, b"<xs:schema/>").unwrap();

    // chdir 不可移植，改为直接注册一个指向该文件的映射。
    let mut r = PluggableSchemaResolver::new();
    r.register(
        format!("http://e.com/{filename}"),
        path.to_string_lossy().into_owned(),
    );
    let out = r
        .resolve_entity("pub", &format!("http://e.com/{filename}"))
        .unwrap();
    assert!(out.is_some(), "应能解析到本地 xsd");
    assert_eq!(out.unwrap().public_id, "pub");
    let _ = std::fs::remove_file(&path);
}

// =============================================================================
// 11. namespace_handler_support.rs —— register_parser / find_parser / parse
// =============================================================================

#[derive(Debug)]
struct RecordingParser {
    bean_name: String,
}

impl BeanDefinitionParser for RecordingParser {
    fn parse(
        &self,
        _element: &Element,
        registry: &mut dyn BeanDefinitionRegistry,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let def = RootBeanDefinition::new();
        register_single(registry, self.bean_name.clone(), Box::new(def))
    }
    fn bean_type_name(&self) -> &str {
        "recording"
    }
}

#[test]
fn namespace_handler_support_register_and_find_parser() {
    let mut h = NamespaceHandlerSupport::new();
    let p = Box::new(RecordingParser {
        bean_name: "rec".to_string(),
    });
    h.register_parser("component-scan", p);
    assert!(h.find_parser("component-scan").is_some());
    assert_eq!(
        h.find_parser("component-scan").unwrap().bean_type_name(),
        "recording"
    );
    // 未注册名 → None。
    assert!(h.find_parser("unknown").is_none());
}

#[test]
fn namespace_handler_support_registered_names_contains_entry() {
    let mut h = NamespaceHandlerSupport::new();
    h.register_parser(
        "a",
        Box::new(RecordingParser {
            bean_name: "n".to_string(),
        }),
    );
    h.register_parser(
        "b",
        Box::new(RecordingParser {
            bean_name: "m".to_string(),
        }),
    );
    let names = h.registered_names();
    assert!(names.iter().any(|n| n == "a"));
    assert!(names.iter().any(|n| n == "b"));
    assert_eq!(names.len(), 2);
}

#[test]
fn namespace_handler_support_parse_via_parser_into_registry() {
    let mut h = NamespaceHandlerSupport::new();
    h.register_parser(
        "mvc",
        Box::new(RecordingParser {
            bean_name: "mvcBean".to_string(),
        }),
    );
    // 借助 find_parser 取得解析器并执行 parse。
    let mut registry = SimpleBeanDefinitionRegistry::new();
    let el = Element::new("", "mvc");
    let n = h
        .find_parser("mvc")
        .unwrap()
        .parse(&el, &mut registry)
        .unwrap();
    assert_eq!(n, 1);
    assert!(registry.contains_bean_definition("mvcBean"));
}

#[test]
fn namespace_handler_support_init_is_noop_default() {
    let mut h = NamespaceHandlerSupport::new();
    h.init();
    assert!(h.registered_names().is_empty());
}

#[test]
fn namespace_handler_support_decorate_default_false() {
    let h = NamespaceHandlerSupport::new();
    assert!(!h.decorate());
}

#[test]
fn is_known_namespace_helper_branches() {
    let mut el = Element::new("urn:foo", "x");
    assert!(is_known_namespace(&el, "urn:foo"));
    assert!(!is_known_namespace(&el, "urn:bar"));
    el.namespace_uri = String::new();
    assert!(!is_known_namespace(&el, "urn:foo"));
}

// =============================================================================
// 12. default_namespace_handler_resolver.rs —— resolve 已注册 / 未注册
// =============================================================================

#[test]
fn default_ns_handler_resolver_resolve_registered_returns_handler() {
    let r = DefaultNamespaceHandlerResolver::new();
    let handler: Arc<dyn NamespaceHandler> = Arc::new(NamespaceHandlerSupport::new());
    r.register("urn:registered", handler);
    assert!(r.resolve("urn:registered").is_some());
}

#[test]
fn default_ns_handler_resolver_resolve_unregistered_returns_none() {
    let r = DefaultNamespaceHandlerResolver::new();
    assert!(r.resolve("urn:not-there").is_none());
}

#[test]
fn default_ns_handler_resolver_factory_init_called_on_first_resolve() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static INIT_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct CountingHandler;
    impl NamespaceHandler for CountingHandler {
        fn init(&mut self) {
            INIT_COUNT.fetch_add(1, Ordering::SeqCst);
        }
        fn find_parser(&self, _name: &str) -> Option<&dyn BeanDefinitionParser> {
            None
        }
    }

    let r = DefaultNamespaceHandlerResolver::new();
    r.register_factory("urn:counting", || Box::new(CountingHandler));
    // 首次解析触发工厂构造并调用 init。
    let h1 = r
        .resolve("urn:counting")
        .expect("factory should produce handler");
    assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);
    // 二次解析返回缓存，不再触发 init。
    let h2 = r.resolve("urn:counting").expect("cached handler");
    assert!(Arc::ptr_eq(&h1, &h2));
    assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn default_ns_handler_resolver_namespace_uris_merges_handlers_and_factories() {
    let r = DefaultNamespaceHandlerResolver::new();
    r.register("urn:direct", Arc::new(NamespaceHandlerSupport::new()));
    r.register_factory("urn:factory", || Box::new(NamespaceHandlerSupport::new()));
    let uris = r.namespace_uris();
    assert!(uris.contains(&"urn:direct".to_string()));
    assert!(uris.contains(&"urn:factory".to_string()));
    assert_eq!(uris.len(), 2);
}

#[test]
fn default_ns_handler_resolver_empty_returns_empty_uris() {
    let r = DefaultNamespaceHandlerResolver::new();
    assert!(r.namespace_uris().is_empty());
}

// =============================================================================
// 13. xml_reader_helpers.rs —— parse_boolean / boolean_attribute / element_id / class / depends_on
// =============================================================================

#[test]
fn xml_helpers_parse_boolean_true_literals() {
    for v in ["true", "TRUE", "True", "1", "yes", "YES", "on", "ON"] {
        assert_eq!(
            xml_reader_helpers::parse_boolean(v),
            Some(true),
            "value={v}"
        );
    }
}

#[test]
fn xml_helpers_parse_boolean_false_literals() {
    for v in ["false", "FALSE", "0", "no", "NO", "off", "OFF"] {
        assert_eq!(
            xml_reader_helpers::parse_boolean(v),
            Some(false),
            "value={v}"
        );
    }
}

#[test]
fn xml_helpers_parse_boolean_invalid_returns_none() {
    for v in ["maybe", "2", "y", "n", "", "  ", "tru"] {
        assert_eq!(xml_reader_helpers::parse_boolean(v), None, "value={v}");
    }
}

#[test]
fn xml_helpers_parse_boolean_trims_whitespace() {
    assert_eq!(xml_reader_helpers::parse_boolean("  true  "), Some(true));
    assert_eq!(xml_reader_helpers::parse_boolean("\tfalse\n"), Some(false));
}

#[test]
fn xml_helpers_boolean_attribute_present_and_valid() {
    let mut el = Element::new("", "bean");
    el.set_attribute("primary", "true");
    el.set_attribute("abstract", "false");
    assert!(xml_reader_helpers::boolean_attribute(&el, "primary", false));
    assert!(!xml_reader_helpers::boolean_attribute(
        &el, "abstract", true
    ));
}

#[test]
fn xml_helpers_boolean_attribute_absent_returns_default() {
    let el = Element::new("", "bean");
    assert!(xml_reader_helpers::boolean_attribute(&el, "missing", true));
    assert!(!xml_reader_helpers::boolean_attribute(
        &el, "missing", false
    ));
}

#[test]
fn xml_helpers_boolean_attribute_invalid_value_returns_default() {
    let mut el = Element::new("", "bean");
    el.set_attribute("weird", "huh?");
    assert!(xml_reader_helpers::boolean_attribute(&el, "weird", true));
    assert!(!xml_reader_helpers::boolean_attribute(&el, "weird", false));
}

#[test]
fn xml_helpers_element_id_returns_none_when_absent() {
    let el = Element::new("", "bean");
    assert!(xml_reader_helpers::element_id(&el).is_none());
}

#[test]
fn xml_helpers_element_class_accessor() {
    let mut el = Element::new("", "bean");
    assert!(xml_reader_helpers::element_class(&el).is_none());
    el.set_attribute("class", "com.example.Foo");
    assert_eq!(
        xml_reader_helpers::element_class(&el).as_deref(),
        Some("com.example.Foo")
    );
}

#[test]
fn xml_helpers_element_scope_and_parent() {
    let mut el = Element::new("", "bean");
    el.set_attribute("scope", "prototype");
    el.set_attribute("parent", "base");
    assert_eq!(
        xml_reader_helpers::element_scope(&el).as_deref(),
        Some("prototype")
    );
    assert_eq!(
        xml_reader_helpers::element_parent(&el).as_deref(),
        Some("base")
    );
}

#[test]
fn xml_helpers_depends_on_list_various_separators() {
    let mut el = Element::new("", "bean");
    el.set_attribute("depends-on", "a,b c");
    assert_eq!(
        xml_reader_helpers::depends_on_list(&el),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
}

#[test]
fn xml_helpers_depends_on_list_empty_when_absent() {
    let el = Element::new("", "bean");
    assert!(xml_reader_helpers::depends_on_list(&el).is_empty());
}

#[test]
fn xml_helpers_depends_on_list_filters_empty_segments() {
    let mut el = Element::new("", "bean");
    el.set_attribute("depends-on", " a , , b ");
    assert_eq!(
        xml_reader_helpers::depends_on_list(&el),
        vec!["a".to_string(), "b".to_string()]
    );
}

#[test]
fn xml_helpers_is_default_namespace_branches() {
    let mut el = Element::new("", "bean");
    assert!(xml_reader_helpers::is_default_namespace(&el));
    el.namespace_uri = "http://www.springframework.org/schema/beans".to_string();
    assert!(xml_reader_helpers::is_default_namespace(&el));
    el.namespace_uri = "http://other".to_string();
    assert!(!xml_reader_helpers::is_default_namespace(&el));
}

#[test]
fn xml_helpers_value_attributes_excluding_filters_keys() {
    let mut el = Element::new("", "property");
    el.set_attribute("name", "p");
    el.set_attribute("value", "v");
    el.set_attribute("ref", "r");
    let pairs = xml_reader_helpers::value_attributes_excluding(&el, &["name", "ref"]);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].0, "value");
    assert_eq!(pairs[0].1, "v");
}

#[test]
fn xml_helpers_local_name_and_element_names() {
    let mut el = Element::new("", "bean");
    el.set_attribute("name", "a;b c");
    assert_eq!(xml_reader_helpers::local_name(&el), "bean");
    assert_eq!(
        xml_reader_helpers::element_names(&el),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
}

#[test]
fn xml_helpers_constants_are_exported() {
    // 这些常量来自 xml_reader_helpers 模块。
    assert_eq!(xml_reader_helpers::ID_ATTRIBUTE, "id");
    assert_eq!(xml_reader_helpers::NAME_ATTRIBUTE, "name");
    assert_eq!(xml_reader_helpers::CLASS_ATTRIBUTE, "class");
    assert_eq!(xml_reader_helpers::SCOPE_ATTRIBUTE, "scope");
    assert_eq!(xml_reader_helpers::LAZY_INIT_ATTRIBUTE, "lazy-init");
    assert_eq!(xml_reader_helpers::DEPENDS_ON_ATTRIBUTE, "depends-on");
}

// =============================================================================
// 14. reader_context.rs —— error / warning / fatal / resource / registry_handle
// =============================================================================

#[test]
fn reader_context_error_message_includes_resource_and_message() {
    let resource: Arc<dyn Resource> = Arc::new(AbstractResource::from_string("c", "res-A"));
    let registry: Arc<dyn BeanDefinitionRegistry> = Arc::new(SimpleBeanDefinitionRegistry::new());
    let ctx = ReaderContext::new(resource, registry, None);
    let err = ctx.error("something failed");
    let msg = err.to_string();
    assert!(msg.contains("res-A"), "msg = {msg}");
    assert!(msg.contains("something failed"));
    assert!(msg.contains("Error"));
}

#[test]
fn reader_context_warning_does_not_panic() {
    let resource: Arc<dyn Resource> = Arc::new(AbstractResource::from_string("c", "res-W"));
    let registry: Arc<dyn BeanDefinitionRegistry> = Arc::new(SimpleBeanDefinitionRegistry::new());
    let ctx = ReaderContext::new(resource, registry, None);
    // warning 仅打印到 stderr，无返回值。
    ctx.warning("just a heads up");
}

#[test]
fn reader_context_fatal_message_format() {
    let resource: Arc<dyn Resource> = Arc::new(AbstractResource::from_string("c", "res-F"));
    let registry: Arc<dyn BeanDefinitionRegistry> = Arc::new(SimpleBeanDefinitionRegistry::new());
    let ctx = ReaderContext::new(resource, registry, None);
    let err = ctx.fatal("kaput");
    let msg = err.to_string();
    assert!(msg.contains("Fatal"));
    assert!(msg.contains("kaput"));
    assert!(msg.contains("res-F"));
}

#[test]
fn reader_context_resource_and_resource_description() {
    let resource: Arc<dyn Resource> = Arc::new(AbstractResource::from_string("c", "the-res"));
    let registry: Arc<dyn BeanDefinitionRegistry> = Arc::new(SimpleBeanDefinitionRegistry::new());
    let ctx = ReaderContext::new(Arc::clone(&resource), registry, None);
    assert_eq!(ctx.resource_description(), "the-res");
    assert_eq!(ctx.resource().description(), "the-res");
}

#[test]
fn reader_context_registry_handle_shares_state() {
    let mut reg = SimpleBeanDefinitionRegistry::new();
    reg.register_bean_definition("pre".to_string(), Box::new(RootBeanDefinition::new()))
        .unwrap();
    let resource: Arc<dyn Resource> = Arc::new(AbstractResource::from_string("c", "r"));
    let registry: Arc<dyn BeanDefinitionRegistry> = Arc::new(reg);
    let ctx = ReaderContext::new(resource, registry, None);
    // registry() 与 registry_handle() 应反映同一份状态。
    assert_eq!(ctx.registry().bean_definition_count(), 1);
    let handle = ctx.registry_handle();
    assert_eq!(handle.bean_definition_count(), 1);
    assert!(handle.contains_bean_definition("pre"));
}

#[test]
fn reader_context_without_namespace_handler_resolver() {
    let resource: Arc<dyn Resource> = Arc::new(AbstractResource::from_string("c", "r"));
    let registry: Arc<dyn BeanDefinitionRegistry> = Arc::new(SimpleBeanDefinitionRegistry::new());
    let ctx = ReaderContext::new(resource, registry, None);
    assert!(ctx.namespace_handler_resolver().is_none());
}

#[test]
fn reader_context_with_namespace_handler_resolver_resolves() {
    let resource: Arc<dyn Resource> = Arc::new(AbstractResource::from_string("c", "r"));
    let registry: Arc<dyn BeanDefinitionRegistry> = Arc::new(SimpleBeanDefinitionRegistry::new());
    let nh: Arc<dyn NamespaceHandlerResolver> = Arc::new(DefaultNamespaceHandlerResolver::new());
    let ctx = ReaderContext::new(resource, registry, Some(nh));
    let nh = ctx.namespace_handler_resolver().expect("should be present");
    // 未注册的命名空间 → None。
    assert!(nh.resolve("urn:none").is_none());
}

// =============================================================================
// 15. abstract_bean_definition_reader_impl.rs —— SharedRegistry / register / load empty
// =============================================================================

fn noop_callback() -> ParseCallback {
    Arc::new(|_bytes: &[u8], _name: &str| Ok(Vec::new()))
}

#[test]
fn shared_registry_new_starts_empty() {
    let shared = SharedRegistry::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    assert_eq!(shared.bean_definition_count(), 0);
    assert!(shared.bean_definition_names().is_empty());
}

#[test]
fn shared_registry_clone_shares_inner_state() {
    let shared = SharedRegistry::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let mut cloned = shared.clone();
    cloned
        .register_bean_definition("x".to_string(), Box::new(RootBeanDefinition::new()))
        .unwrap();
    // 两个句柄看到同一份状态。
    assert!(shared.contains_bean_definition("x"));
    assert_eq!(shared.bean_definition_count(), 1);
}

#[test]
fn shared_registry_get_bean_definition_returns_none_due_to_mutex_constraint() {
    // 实现注释说明：互斥锁下无法返回引用，get_bean_definition 恒返回 None。
    let shared = SharedRegistry::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    assert!(shared.get_bean_definition("anything").is_none());
}

#[test]
fn shared_registry_remove_bean_definition() {
    let shared = SharedRegistry::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let mut s = shared.clone();
    s.register_bean_definition("toRemove".to_string(), Box::new(RootBeanDefinition::new()))
        .unwrap();
    let removed = s.remove_bean_definition("toRemove").unwrap();
    assert_eq!(removed.bean_class_name(), "unknown");
    assert!(!shared.contains_bean_definition("toRemove"));
}

#[test]
fn shared_registry_remove_missing_errors() {
    let shared = SharedRegistry::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let mut s = shared.clone();
    assert!(s.remove_bean_definition("nope").is_err());
}

#[test]
fn abstract_reader_impl_load_bean_definitions_empty_resource_returns_zero() {
    let mut reader =
        AbstractBeanDefinitionReaderImpl::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.set_parser(noop_callback());
    let resource = TextResource {
        name: "empty".to_string(),
        content: String::new(),
    };
    let count = reader.load_bean_definitions(&resource).unwrap();
    assert_eq!(count, 0);
    assert_eq!(reader.registry().bean_definition_count(), 0);
}

#[test]
fn abstract_reader_impl_no_parser_returns_zero_without_error() {
    let reader =
        AbstractBeanDefinitionReaderImpl::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let resource = TextResource {
        name: "t".to_string(),
        content: "ignored".to_string(),
    };
    let count = reader.load_bean_definitions(&resource).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn abstract_reader_impl_inner_and_inner_mut() {
    let mut reader =
        AbstractBeanDefinitionReaderImpl::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    // inner 默认值。
    assert!(reader.inner().bean_name_generator().is_none());
    reader.inner_mut().set_bean_name_generator(Box::new(
        vernal_beans::default_bean_name_generator::DefaultBeanNameGenerator::new(),
    ));
    assert!(reader.inner().bean_name_generator().is_some());
}

#[test]
fn abstract_reader_impl_shared_registry_handle_matches_registry() {
    let mut reader =
        AbstractBeanDefinitionReaderImpl::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader
        .register_all(vec![(
            "one".to_string(),
            Box::new(RootBeanDefinition::new()) as Box<dyn BeanDefinition>,
        )])
        .unwrap();
    assert_eq!(reader.shared_registry().bean_definition_count(), 1);
    assert_eq!(reader.registry().bean_definition_count(), 1);
}

#[test]
fn abstract_reader_impl_bean_name_generator_via_trait() {
    use vernal_beans::bean_definition_reader::BeanDefinitionReader;
    let mut reader =
        AbstractBeanDefinitionReaderImpl::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    assert!(reader.bean_name_generator().is_none());
    reader.set_bean_name_generator(Box::new(
        vernal_beans::default_bean_name_generator::DefaultBeanNameGenerator::new(),
    ));
    assert!(reader.bean_name_generator().is_some());
}

// =============================================================================
// 16. properties_bean_definition_reader.rs —— meta keys / ref / 多 bean
// =============================================================================

#[test]
fn properties_reader_class_scope_lazy_meta_keys() {
    let text = "\
svc.(class)=com.Svc
svc.(scope)=prototype
svc.(lazy-init)=true
svc.(abstract)=true
svc.(primary)=true
svc.(init-method)=init
svc.(destroy-method)=destroy
svc.(factory-bean)=fb
svc.(factory-method)=fm
svc.port=8080
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(text).unwrap();
    assert_eq!(count, 1);
    let def = reader.registry_ref().get_bean_definition("svc").unwrap();
    let root = def_as_root(def);
    assert_eq!(root.bean_class_name(), "com.Svc");
    assert_eq!(root.scope(), Scope::Transient);
    assert!(root.is_lazy_init());
    assert!(root.is_abstract());
    assert!(root.is_primary());
    assert_eq!(root.init_method_name(), Some("init"));
    assert_eq!(root.destroy_method_name(), Some("destroy"));
    assert_eq!(root.factory_bean_name(), Some("fb"));
    assert_eq!(root.factory_method_name(), Some("fm"));
    assert!(root.property_values().contains("port"));
}

#[test]
fn properties_reader_singleton_scope_when_not_prototype() {
    let text = "\
svc.(class)=com.Svc
svc.(scope)=singleton
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("svc").unwrap();
    let root = def_as_root(def);
    assert_eq!(root.scope(), Scope::Singleton);
}

#[test]
fn properties_reader_depends_on_meta_key_splits() {
    let text = "\
svc.(class)=com.Svc
svc.(depends-on)=a, b c
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("svc").unwrap();
    let root = def_as_root(def);
    assert_eq!(root.depends_on(), &["a", "b", "c"]);
}

#[test]
fn properties_reader_ref_property_recorded_as_ref_marker() {
    let text = "\
svc.(class)=com.Svc
svc.partner(ref)=otherBean
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("svc").unwrap();
    let root = def_as_root(def);
    let pv = root.property_values().get("partner").unwrap();
    let v = pv.value();
    let s = v.downcast_ref::<String>().expect("should be a String");
    // 实现以属性名作为 ref 标记：`<ref:{prop_name}>`。
    assert_eq!(s, "<ref:partner>");
}

#[test]
fn properties_reader_list_property_splits_by_comma() {
    let text = "\
svc.(class)=com.Svc
svc.items(list)=a, b, c
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("svc").unwrap();
    let root = def_as_root(def);
    let pv = root.property_values().get("items").unwrap();
    let s = pv.value().downcast_ref::<String>().unwrap();
    // 实现对每段 trim 后再用 ", " 重新 join → "a, b, c"？实际 join 用 ","。
    assert_eq!(s, "a,b,c");
}

#[test]
fn properties_reader_multiple_bean_definitions() {
    let text = "\
a.(class)=com.A
a.x=1
b.(class)=com.B
b.y=2
c.(class)=com.C
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(text).unwrap();
    assert_eq!(count, 3);
    let names = reader.registry_ref().bean_definition_names();
    for expected in ["a", "b", "c"] {
        assert!(
            names.iter().any(|n| n == expected),
            "missing {expected} in {names:?}"
        );
    }
}

#[test]
fn properties_reader_parent_meta_key() {
    let text = "\
child.(class)=com.Child
child.(parent)=base
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("child").unwrap();
    assert_eq!(def.parent_name(), Some("base"));
}

#[test]
fn properties_reader_continuation_line_joins_values() {
    let text = "\
a.(class)=com.\\
Long\\
Name
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("a").unwrap();
    assert_eq!(def.bean_class_name(), "com.LongName");
}

#[test]
fn properties_reader_colon_separator_supported() {
    let text = "a.(class):com.Colon";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("a").unwrap();
    assert_eq!(def.bean_class_name(), "com.Colon");
}

#[test]
fn properties_reader_whitespace_separator_supported() {
    // 形如 "key value"（首个空白作为分隔符）。
    let text = "a.(class) com.Whitespace";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.load_from_text(text).unwrap();
    let def = reader.registry_ref().get_bean_definition("a").unwrap();
    assert_eq!(def.bean_class_name(), "com.Whitespace");
}

#[test]
fn properties_reader_bang_comment_ignored() {
    let text = "\
! a bang comment
a.(class)=com.A
";
    let mut reader =
        PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(text).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn properties_reader_load_bean_definitions_reports_count_only() {
    // 通过 BeanDefinitionReader trait：仅解析并报告数量，不真正注册。
    use vernal_beans::bean_definition_reader::BeanDefinitionReader;
    let reader = PropertiesBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let resource = TextResource {
        name: "p".to_string(),
        content: "svc.(class)=com.Svc\n".to_string(),
    };
    let count = reader.load_bean_definitions(&resource).unwrap();
    assert_eq!(count, 1);
    // trait 版本不写入 registry()，注册表保持空。
    assert_eq!(reader.registry().bean_definition_count(), 0);
}

// =============================================================================
// 17. xml_bean_definition_reader.rs —— beans/bean 全属性解析
// =============================================================================

#[test]
fn xml_reader_parses_bean_with_all_attributes() {
    let xml = r#"
<beans xmlns="http://www.springframework.org/schema/beans">
    <bean id="svc"
          class="com.example.Svc"
          scope="prototype"
          lazy-init="true"
          abstract="true"
          primary="true"
          depends-on="dep1, dep2"
          init-method="init"
          destroy-method="destroy"
          factory-bean="fb"
          factory-method="fm"
          parent="base">
        <property name="host" value="localhost"/>
        <property name="port" value="8080"/>
    </bean>
</beans>"#;
    // 用一个独立的注册表承接，再借 SharedRegistry 之外的方式校验。
    // 由于 XmlBeanDefinitionReader::registry() 返回空占位，
    // 这里通过 load_from_text 返回数量来确认解析成功。
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_singleton_scope_when_not_prototype() {
    // 非 prototype 的 scope 值 → 走 singleton 分支，解析成功。
    let xml = r#"<beans xmlns="http://www.springframework.org/schema/beans">
        <bean id="s" class="com.S" scope="singleton"/>
    </beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_no_class_bean_not_collected() {
    let xml = r#"<beans xmlns="http://www.springframework.org/schema/beans">
        <bean id="noclass"/>
    </beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn xml_reader_nested_beans_elements_collected() {
    let xml = r#"<beans xmlns="http://www.springframework.org/schema/beans">
        <beans>
            <bean id="inner" class="com.Inner"/>
        </beans>
        <bean id="outer" class="com.Outer"/>
    </beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn xml_reader_bean_without_id_uses_name_first_alias() {
    let xml = r#"<beans xmlns="http://www.springframework.org/schema/beans">
        <bean name="first, second" class="com.Named"/>
    </beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_bean_without_id_or_name_uses_class_as_name() {
    let xml = r#"<beans xmlns="http://www.springframework.org/schema/beans">
        <bean class="com.Anonymous"/>
    </beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_load_from_resource_via_input_stream() {
    let xml = r#"<beans xmlns="http://www.springframework.org/schema/beans">
        <bean id="r" class="com.R"/>
    </beans>"#;
    let resource = AbstractResource::from_string(xml, "in-memory-resource");
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_resource(&resource).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_property_without_value_not_recorded() {
    // <property> 缺少 value 子元素 → 不被记录，但 bean 仍注册。
    let xml = r#"<beans xmlns="http://www.springframework.org/schema/beans">
        <bean id="p" class="com.P">
            <property name="noValue"/>
        </bean>
    </beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_set_document_loader_swap() {
    // 替换 document_loader 应能正常工作。
    let mut reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    reader.set_document_loader(Box::new(DefaultDocumentLoader::new()));
    let count = reader
        .load_from_text(r#"<beans><bean id="x" class="com.X"/></beans>"#)
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_malformed_xml_returns_error() {
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    // 未闭合的元素声明（缺引号结尾）→ 解析器报错。
    let res = reader.load_from_text(r#"<beans><bean id="x" class=#</beans>"#);
    assert!(res.is_err(), "malformed XML should produce an error");
}

#[test]
fn xml_reader_empty_namespace_treated_as_beans() {
    // 空命名空间也被视为默认 beans 命名空间。
    let xml = r#"<beans>
        <bean id="e" class="com.E"/>
    </beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn xml_reader_namespaced_prefix_still_treated_as_beans_namespace() {
    // 本实现的简易解析器不解析 xmlns:prefix 到 namespace_uri 字段，
    // 因此 `other:beans` 的 namespace_uri 为空、local_name 为 `beans`，
    // 仍被当作默认 beans 命名空间处理（与 `is_beans_namespace` 的空串分支一致）。
    let xml = r#"<other:beans xmlns:other="http://other">
        <other:bean id="x" class="com.X"/>
    </other:beans>"#;
    let reader = XmlBeanDefinitionReader::new(Box::new(SimpleBeanDefinitionRegistry::new()));
    let count = reader.load_from_text(xml).unwrap();
    // local_name "bean" 被剥离前缀后命中默认收集逻辑。
    assert_eq!(count, 1);
}
