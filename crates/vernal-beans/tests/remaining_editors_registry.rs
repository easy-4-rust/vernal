//! 剩余 PropertyEditor + BeanDefinitionRegistry remove 测试。

use vernal_beans::ComponentDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::Scope;
use vernal_beans::property_editor::PropertyEditor;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug)]
struct Config {
    url: String,
}

// ── 1. FileEditor 测试 ───────────────────────────────────────────────────

/// 参照 Spring `FileEditorTests`：验证文件路径解析。
#[test]
fn file_editor_basic() {
    let mut editor = vernal_beans::file_editor::FileEditor::new();
    editor.set_as_text("/tmp/test.txt").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("/tmp/test.txt"));
}

#[test]
fn file_editor_empty() {
    let mut editor = vernal_beans::file_editor::FileEditor::new();
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none());
}

// ── 2. ClassEditor 测试 ──────────────────────────────────────────────────

/// 参照 Spring `ClassEditorTests`：验证类名解析。
#[test]
fn class_editor_basic() {
    let mut editor = vernal_beans::class_editor::ClassEditor::new();
    editor.set_as_text("java.lang.String").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("java.lang.String"));
}

#[test]
fn class_editor_invalid() {
    let mut editor = vernal_beans::class_editor::ClassEditor::new();
    let result = editor.set_as_text("invalid class name");
    assert!(result.is_err());
}

// ── 3. LocaleEditor 测试 ─────────────────────────────────────────────────

/// 参照 Spring `LocaleEditorTests`：验证 locale 解析。
#[test]
fn locale_editor_basic() {
    let mut editor = vernal_beans::locale_editor::LocaleEditor::new();
    editor.set_as_text("en_US").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("en_US"));
}

#[test]
fn locale_editor_empty() {
    let mut editor = vernal_beans::locale_editor::LocaleEditor::new();
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none());
}

// ── 4. CharsetEditor 测试 ────────────────────────────────────────────────

/// 参照 Spring `CharsetEditorTests`：验证 charset 解析。
#[test]
fn charset_editor_basic() {
    let mut editor = vernal_beans::charset_editor::CharsetEditor::new();
    editor.set_as_text("UTF-8").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("UTF-8"));
}

// ── 5. PathEditor 测试 ──────────────────────────────────────────────────

/// 参照 Spring `PathEditorTests`：验证路径解析。
#[test]
fn path_editor_basic() {
    let mut editor = vernal_beans::path_editor::PathEditor::new();
    editor.set_as_text("/usr/local/bin").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("/usr/local/bin"));
}

// ── 6. PatternEditor 测试 ────────────────────────────────────────────────

/// 参照 Spring `PatternEditorTests`：验证正则模式解析。
#[test]
fn pattern_editor_basic() {
    let mut editor = vernal_beans::pattern_editor::PatternEditor::new();
    editor.set_as_text("\\d+").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("\\d+"));
}

// ── 7. TimeZoneEditor 测试 ───────────────────────────────────────────────

/// 参照 Spring `TimeZoneEditorTests`：验证时区解析。
#[test]
fn timezone_editor_basic() {
    let mut editor = vernal_beans::timezone_editor::TimeZoneEditor::new();
    editor.set_as_text("UTC").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("UTC"));
}

// ── 8. ZoneIdEditor 测试 ─────────────────────────────────────────────────

/// 参照 Spring `ZoneIdEditorTests`：验证 zone id 解析。
#[test]
fn zone_id_editor_basic() {
    let mut editor = vernal_beans::zone_id_editor::ZoneIdEditor::new();
    editor.set_as_text("Asia/Shanghai").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("Asia/Shanghai"));
}

// ── 9. InputStreamEditor 测试 ────────────────────────────────────────────

/// 验证 InputStreamEditor。
#[test]
fn input_stream_editor_basic() {
    let mut editor = vernal_beans::input_stream_editor::InputStreamEditor::new();
    editor.set_as_text("/data/input.txt").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("/data/input.txt"));
}

// ── 10. CharArrayPropertyEditor 测试 ─────────────────────────────────────

/// 验证 CharArrayPropertyEditor。
#[test]
fn char_array_editor_basic() {
    let mut editor = vernal_beans::char_array_editor::CharArrayPropertyEditor::new();
    editor.set_as_text("hello").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("hello"));
}

// ── 11. ByteArrayPropertyEditor 测试 ─────────────────────────────────────

/// 验证 ByteArrayPropertyEditor。
#[test]
fn byte_array_editor_basic() {
    let mut editor = vernal_beans::byte_array_editor::ByteArrayPropertyEditor::new();
    editor.set_as_text("data").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("data"));
}

// ── 12. PropertiesEditor 测试 ────────────────────────────────────────────

/// 验证 PropertiesEditor。
#[test]
fn properties_editor_basic() {
    let mut editor = vernal_beans::properties_editor::PropertiesEditor::new();
    editor.set_as_text("key=value").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("key=value"));
}

// ── 13. ReaderEditor 测试 ────────────────────────────────────────────────

/// 验证 ReaderEditor。
#[test]
fn reader_editor_basic() {
    let mut editor = vernal_beans::reader_editor::ReaderEditor::new();
    editor.set_as_text("/data/reader.txt").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("/data/reader.txt"));
}

// ── 14. StringArrayPropertyEditor 测试 ───────────────────────────────────

/// 验证 StringArrayPropertyEditor。
#[test]
fn string_array_editor_basic() {
    let mut editor = vernal_beans::string_array_editor::StringArrayPropertyEditor::new();
    editor.set_as_text("a,b,c").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("a,b,c"));
}

// ── 15. ClassArrayEditor 测试 ────────────────────────────────────────────

/// 验证 ClassArrayEditor。
#[test]
fn class_array_editor_basic() {
    let mut editor = vernal_beans::class_array_editor::ClassArrayEditor::new();
    editor.set_as_text("String,Integer").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("String,Integer"));
}

// ── 16. CurrencyEditor 测试 ──────────────────────────────────────────────

/// 验证 CurrencyEditor。
#[test]
fn currency_editor_basic() {
    let mut editor = vernal_beans::currency_editor::CurrencyEditor::new();
    editor.set_as_text("USD").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("USD"));
}

// ── 17. 所有编辑器 interface 测试 ────────────────────────────────────────

/// 验证所有编辑器实现了 PropertyEditor trait。
#[test]
fn all_editors_implement_trait() {
    fn assert_editor<T: PropertyEditor>() {}
    assert_editor::<vernal_beans::string_trimmer_editor::StringTrimmerEditor>();
    assert_editor::<vernal_beans::boolean_editor::CustomBooleanEditor>();
    assert_editor::<vernal_beans::number_editor::CustomNumberEditor>();
    assert_editor::<vernal_beans::uri_editor::URIEditor>();
    assert_editor::<vernal_beans::uuid_editor::UUIDEditor>();
    assert_editor::<vernal_beans::file_editor::FileEditor>();
    assert_editor::<vernal_beans::class_editor::ClassEditor>();
    assert_editor::<vernal_beans::locale_editor::LocaleEditor>();
    assert_editor::<vernal_beans::charset_editor::CharsetEditor>();
    assert_editor::<vernal_beans::path_editor::PathEditor>();
    assert_editor::<vernal_beans::pattern_editor::PatternEditor>();
    assert_editor::<vernal_beans::timezone_editor::TimeZoneEditor>();
    assert_editor::<vernal_beans::zone_id_editor::ZoneIdEditor>();
    assert_editor::<vernal_beans::input_stream_editor::InputStreamEditor>();
    assert_editor::<vernal_beans::char_array_editor::CharArrayPropertyEditor>();
    assert_editor::<vernal_beans::byte_array_editor::ByteArrayPropertyEditor>();
    assert_editor::<vernal_beans::properties_editor::PropertiesEditor>();
    assert_editor::<vernal_beans::reader_editor::ReaderEditor>();
    assert_editor::<vernal_beans::string_array_editor::StringArrayPropertyEditor>();
    assert_editor::<vernal_beans::class_array_editor::ClassArrayEditor>();
    assert_editor::<vernal_beans::currency_editor::CurrencyEditor>();
}

// ── 18. BeanDefinitionRegistry remove 操作测试 ───────────────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.beanDefinitionRemoval`：
/// 验证删除和重新注册。
#[test]
fn registry_remove_and_reregister() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "first".to_string(),
            },
        ))
        .unwrap();

    assert_eq!(builder.len(), 1);
    assert!(builder.contains::<Config>());

    // 删除
    builder.remove::<Config>().unwrap();
    assert_eq!(builder.len(), 0);
    assert!(!builder.contains::<Config>());

    // 重新注册
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "second".to_string(),
            },
        ))
        .unwrap();
    assert_eq!(builder.len(), 1);
    assert!(builder.contains::<Config>());
}

/// 验证删除不存在的类型返回错误。
#[test]
fn registry_remove_nonexistent() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    // 删除不存在的类型
    let result = builder.remove::<String>(); // String 未注册
    assert!(result.is_err());
}

/// 验证 remove_by_key 删除。
#[test]
fn registry_remove_by_key() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let key = ComponentKey::of::<Config>();
    builder.remove_by_key(&key).unwrap();
    assert_eq!(builder.len(), 0);
}

/// 验证 remove_by_key 删除不存在的键返回错误。
#[test]
fn registry_remove_by_key_nonexistent() {
    let mut builder = RegistryBuilder::new();
    let key = ComponentKey::of::<String>();
    let result = builder.remove_by_key(&key);
    assert!(result.is_err());
}

/// 验证 contains 检查。
#[test]
fn registry_contains() {
    let mut builder = RegistryBuilder::new();
    assert!(!builder.contains::<Config>());

    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "test".to_string(),
            },
        ))
        .unwrap();
    assert!(builder.contains::<Config>());
    assert!(!builder.contains::<String>());
}

/// 验证删除后 registry 可以正常构建。
#[test]
fn registry_remove_then_build() {
    #[derive(Debug)]
    struct ServiceA;
    #[derive(Debug)]
    struct ServiceB;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(ServiceA))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value(ServiceB))
        .unwrap();

    builder.remove::<ServiceA>().unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.len(), 1);
}

/// 验证批量删除多个类型。
#[test]
fn registry_remove_multiple() {
    #[derive(Debug)]
    struct ServiceA;
    #[derive(Debug)]
    struct ServiceB;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(ServiceA))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value(ServiceB))
        .unwrap();

    assert_eq!(builder.len(), 2);

    builder.remove::<ServiceA>().unwrap();
    assert_eq!(builder.len(), 1);
    assert!(!builder.contains::<ServiceA>());
    assert!(builder.contains::<ServiceB>());

    builder.remove::<ServiceB>().unwrap();
    assert_eq!(builder.len(), 0);
    assert!(builder.is_empty());
}

/// 验证删除后重新注册不同实例。
#[test]
fn registry_remove_and_register_new_instance() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "old".to_string(),
            },
        ))
        .unwrap();

    builder.remove::<Config>().unwrap();

    // 注册为 transient
    builder
        .register(ComponentDefinition::transient::<Config, _>(
            |_resolver: &Resolver| Config {
                url: "new".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let config = container.resolve::<Config>().unwrap();
    assert_eq!(config.url, "new");
}
