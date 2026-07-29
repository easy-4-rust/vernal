//! 全覆盖 PropertyEditor 的边缘情况测试。
//! 测试每个 Editor 的 new()、Default、set_as_text/set_value 错误路径等。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::property_editor::PropertyEditor;

// ── ClassEditor ──────────────────────────────────────────────────────────

#[test]
fn class_editor_edge_cases() {
    use vernal_beans::class_editor::ClassEditor;
    let mut e = ClassEditor::new();
    // Default
    let _e2: ClassEditor = Default::default();
    // get_value on empty
    assert!(e.get_value().is_none());
    // get_value_type
    assert_eq!(e.get_value_type(), std::any::TypeId::of::<String>());
    // set_as_text with space → error
    assert!(e.set_as_text("bad class name").is_err());
    // set_as_text empty → None value
    assert!(e.set_as_text("").is_ok());
    assert!(e.get_value().is_none());
    // set_value with wrong type (i32 instead of String)
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_value with correct type
    e.set_value(Arc::new("com.example.Foo".to_string()));
    assert!(e.get_value().is_some());
    // set_as_text valid
    e.set_as_text("com.example.Bar").unwrap();
    assert_eq!(e.get_as_text(), Some("com.example.Bar".to_string()));
    // target_type
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

// ── CustomBooleanEditor ──────────────────────────────────────────────────

#[test]
fn boolean_editor_edge_cases() {
    use vernal_beans::boolean_editor::CustomBooleanEditor;
    let mut e = CustomBooleanEditor::new();
    let _e2: CustomBooleanEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.get_value_type(), std::any::TypeId::of::<bool>());
    // set_value with wrong type
    e.set_value(Arc::new("not_bool".to_string()));
    assert_eq!(e.get_as_text(), None);
    // set_value with correct type
    e.set_value(Arc::new(true));
    assert!(e.get_as_text().is_some());
    // set_as_text "true"
    e.set_as_text("true").unwrap();
    assert_eq!(e.get_as_text(), Some("true".to_string()));
    // set_as_text "false"
    e.set_as_text("false").unwrap();
    assert_eq!(e.get_as_text(), Some("false".to_string()));
    // set_as_text empty string returns error for boolean editor
    assert!(e.set_as_text("").is_err());
}

// ── CharacterEditor ─────────────────────────────────────────────────────

#[test]
fn character_editor_edge_cases() {
    use vernal_beans::char_property_editor::CharacterEditor;
    let mut e = CharacterEditor::new();
    let _e2: CharacterEditor = Default::default();
    assert!(e.get_value().is_none());
    // CharacterEditor's get_value_type returns char's TypeId
    assert_eq!(e.get_value_type(), std::any::TypeId::of::<char>());
    // set_value with wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_value with correct type
    e.set_value(Arc::new('A'));
    assert_eq!(e.get_as_text(), Some("A".to_string()));
    // set_as_text single char
    e.set_as_text("X").unwrap();
    // target_type returns char's TypeId
    assert_eq!(e.target_type(), std::any::TypeId::of::<char>());
}

// ── CustomNumberEditor ──────────────────────────────────────────────────

#[test]
fn number_editor_edge_cases() {
    use vernal_beans::number_editor::CustomNumberEditor;
    let mut e = CustomNumberEditor::new();
    let _e2: CustomNumberEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.get_value_type(), std::any::TypeId::of::<f64>());
    // set_value with wrong type
    e.set_value(Arc::new("not_a_number".to_string()));
    assert!(e.get_value().is_none());
    // set_value with correct type
    e.set_value(Arc::new(42.5f64));
    assert_eq!(e.get_as_text(), Some("42.5".to_string()));
    // set_as_text valid
    e.set_as_text("123.456").unwrap();
    assert!(e.get_value().is_some());
    // set_as_text invalid
    assert!(e.set_as_text("not_a_number").is_err());
    // set_as_text empty returns error
    assert!(e.set_as_text("").is_err());
}

// ── ByteArrayPropertyEditor ─────────────────────────────────────────────

#[test]
fn byte_array_editor_edge_cases() {
    use vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor;
    let mut e = ByteArrayPropertyEditor::new();
    let _e2: ByteArrayPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.get_value_type(), std::any::TypeId::of::<Vec<u8>>());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("hello").unwrap();
    assert!(e.get_value().is_some());
}

// ── CharArrayPropertyEditor ─────────────────────────────────────────────

#[test]
fn char_array_editor_edge_cases() {
    use vernal_beans::char_array_editor::CharArrayPropertyEditor;
    let mut e = CharArrayPropertyEditor::new();
    let _e2: CharArrayPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("chars").unwrap();
    assert!(e.get_value().is_some());
}

// ── StringArrayPropertyEditor ───────────────────────────────────────────

#[test]
fn string_array_editor_edge_cases() {
    use vernal_beans::string_array_editor::StringArrayPropertyEditor;
    let mut e = StringArrayPropertyEditor::new();
    let _e2: StringArrayPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("a,b,c").unwrap();
    assert!(e.get_value().is_some());
}

// ── URIEditor ───────────────────────────────────────────────────────────

#[test]
fn uri_editor_edge_cases() {
    use vernal_beans::uri_editor::URIEditor;
    let mut e = URIEditor::new();
    let _e2: URIEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_value correct type
    e.set_value(Arc::new("https://example.com".to_string()));
    // set_as_text valid URI
    e.set_as_text("https://rust-lang.org").unwrap();
    assert!(e.get_value().is_some());
}

// ── UUIDEditor ──────────────────────────────────────────────────────────

#[test]
fn uuid_editor_edge_cases() {
    use vernal_beans::uuid_editor::UUIDEditor;
    let mut e = UUIDEditor::new();
    let _e2: UUIDEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text invalid UUID
    assert!(e.set_as_text("not-a-uuid").is_err());
    // set_as_text valid UUID
    let uuid = "550e8400-e29b-41d4-a716-446655440000";
    e.set_as_text(uuid).unwrap();
    assert_eq!(e.get_as_text(), Some(uuid.to_string()));
}

// ── TimeZoneEditor ──────────────────────────────────────────────────────

#[test]
fn timezone_editor_edge_cases() {
    use vernal_beans::timezone_editor::TimeZoneEditor;
    let mut e = TimeZoneEditor::new();
    let _e2: TimeZoneEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("UTC").unwrap();
    assert!(e.get_value().is_some());
    e.set_as_text("America/New_York").unwrap();
    assert!(e.get_value().is_some());
}

// ── LocaleEditor ────────────────────────────────────────────────────────

#[test]
fn locale_editor_edge_cases() {
    use vernal_beans::locale_editor::LocaleEditor;
    let mut e = LocaleEditor::new();
    let _e2: LocaleEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("en_US").unwrap();
    assert!(e.get_value().is_some());
}

// ── PatternEditor ───────────────────────────────────────────────────────

#[test]
fn pattern_editor_edge_cases() {
    use vernal_beans::pattern_editor::PatternEditor;
    let mut e = PatternEditor::new();
    let _e2: PatternEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text valid regex
    e.set_as_text("[a-z]+").unwrap();
    assert!(e.get_value().is_some());
}

// ── CurrencyEditor ──────────────────────────────────────────────────────

#[test]
fn currency_editor_edge_cases() {
    use vernal_beans::currency_editor::CurrencyEditor;
    let mut e = CurrencyEditor::new();
    let _e2: CurrencyEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("USD").unwrap();
    assert!(e.get_value().is_some());
}

// ── ClassEditor ───────────────

#[test]
fn class_editor_more_edges() {
    use vernal_beans::class_editor::ClassEditor;
    let mut e = ClassEditor::new();
    // set_as_text valid
    e.set_as_text("java.lang.String").unwrap();
    assert_eq!(e.get_as_text(), Some("java.lang.String".to_string()));
    // get_value returns Some
    assert!(e.get_value().is_some());
}

#[test]
fn charset_editor_edge_cases() {
    use vernal_beans::charset_editor::CharsetEditor;
    let mut e = CharsetEditor::new();
    let _e2: CharsetEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("UTF-8").unwrap();
    assert!(e.get_value().is_some());
}

#[test]
fn input_stream_editor_edge_cases() {
    use vernal_beans::input_stream_editor::InputStreamEditor;
    let mut e = InputStreamEditor::new();
    let _e2: InputStreamEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("/path/to/file").unwrap();
    assert!(e.get_value().is_some());
}

#[test]
fn resource_bundle_editor_edge_cases() {
    use vernal_beans::resource_bundle_editor::ResourceBundleEditor;
    let mut e = ResourceBundleEditor::new();
    let _e2: ResourceBundleEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("messages").unwrap();
    assert!(e.get_value().is_some());
}

#[test]
fn zone_id_editor_edge_cases() {
    use vernal_beans::zone_id_editor::ZoneIdEditor;
    let mut e = ZoneIdEditor::new();
    let _e2: ZoneIdEditor = Default::default();
    assert!(e.get_value().is_none());
    // set_value wrong type
    e.set_value(Arc::new(42i32));
    assert!(e.get_value().is_none());
    // set_as_text
    e.set_as_text("America/New_York").unwrap();
    assert!(e.get_value().is_some());
}
