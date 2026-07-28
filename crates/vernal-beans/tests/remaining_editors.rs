//! 剩余 PropertyEditor 测试。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::property_editor::PropertyEditor;

// ── 1. InputSourceEditor 测试 ────────────────────────────────────────────

/// 验证 InputSourceEditor。
#[test]
fn input_source_editor_basic() {
    let mut editor = vernal_beans::input_source_editor::InputSourceEditor::new();
    editor.set_as_text("https://example.com/data.xml").unwrap();
    assert_eq!(
        editor.get_as_text().as_deref(),
        Some("https://example.com/data.xml")
    );
}

// ── 2. ByteArrayPropertyEditor 测试 ─────────────────────────────────────

/// 验证 ByteArrayPropertyEditor 字节数组转换。
#[test]
fn byte_array_property_editor_basic() {
    let mut editor = vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor::new();
    editor.set_as_text("hello world").unwrap();
    let value = editor
        .get_value()
        .unwrap()
        .downcast_ref::<Vec<u8>>()
        .unwrap();
    assert_eq!(value, b"hello world");
}

/// 验证 get_as_text 返回字符串。
#[test]
fn byte_array_property_editor_get_as_text() {
    let mut editor = vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor::new();
    editor.set_as_text("test").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("test"));
}

// ── 3. CharacterEditor 测试 ──────────────────────────────────────────────

/// 验证 CharacterEditor 字符解析。
#[test]
fn character_editor_basic() {
    let mut editor = vernal_beans::char_property_editor::CharacterEditor::new();
    editor.set_as_text("A").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<char>().unwrap();
    assert_eq!(*value, 'A');
}

/// 验证 CharacterEditor 多字符取第一个。
#[test]
fn character_editor_multiple_chars() {
    let mut editor = vernal_beans::char_property_editor::CharacterEditor::new();
    editor.set_as_text("Hello").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<char>().unwrap();
    assert_eq!(*value, 'H');
}

/// 验证 CharacterEditor 空字符串。
#[test]
fn character_editor_empty() {
    let mut editor = vernal_beans::char_property_editor::CharacterEditor::new();
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none());
}

// ── 4. CharArrayPropertyEditor 测试 ──────────────────────────────────────

/// 验证 CharArrayPropertyEditor。
#[test]
fn char_array_property_editor_basic() {
    let mut editor = vernal_beans::char_array_property_editor::CharArrayPropertyEditor::new();
    editor.set_as_text("hello").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("hello"));
}

// ── 5. CharsetPropertyEditor 测试 ────────────────────────────────────────

/// 验证 CharsetPropertyEditor。
#[test]
fn charset_property_editor_basic() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    editor.set_as_text("UTF-8").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("UTF-8"));
}

/// 验证 CharsetPropertyEditor 空字符串。
#[test]
fn charset_property_editor_empty() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none());
}

// ── 6. PathPropertyEditor 测试 ───────────────────────────────────────────

/// 验证 PathPropertyEditor。
#[test]
fn path_property_editor_basic() {
    let mut editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    editor.set_as_text("/usr/local/bin").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("/usr/local/bin"));
}

// ── 7. 所有新编辑器 trait 验证 ───────────────────────────────────────────

/// 验证所有新编辑器实现了 PropertyEditor trait。
#[test]
fn all_new_editors_implement_trait() {
    fn assert_editor<T: PropertyEditor>() {}
    assert_editor::<vernal_beans::input_source_editor::InputSourceEditor>();
    assert_editor::<vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor>();
    assert_editor::<vernal_beans::char_property_editor::CharacterEditor>();
    assert_editor::<vernal_beans::char_array_property_editor::CharArrayPropertyEditor>();
    assert_editor::<vernal_beans::charset_property_editor::CharsetPropertyEditor>();
    assert_editor::<vernal_beans::path_property_editor::PathPropertyEditor>();
}

// ── 8. 所有 PropertyEditor 完整列表验证 ──────────────────────────────────

/// 验证所有 26 个 PropertyEditor 都实现了 trait。
#[test]
fn all_26_property_editors_implement_trait() {
    fn assert_editor<T: PropertyEditor>() {}
    // 原有 20 个
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
    // 新增 6 个
    assert_editor::<vernal_beans::input_source_editor::InputSourceEditor>();
    assert_editor::<vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor>();
    assert_editor::<vernal_beans::char_property_editor::CharacterEditor>();
    assert_editor::<vernal_beans::char_array_property_editor::CharArrayPropertyEditor>();
    assert_editor::<vernal_beans::charset_property_editor::CharsetPropertyEditor>();
    assert_editor::<vernal_beans::path_property_editor::PathPropertyEditor>();
}

// ── 9. CharacterEditor set_value 测试 ────────────────────────────────────

/// 验证 CharacterEditor set_value。
#[test]
fn character_editor_set_value() {
    let mut editor = vernal_beans::char_property_editor::CharacterEditor::new();
    editor.set_value(Arc::new('Z'));
    let value = editor.get_value().unwrap().downcast_ref::<char>().unwrap();
    assert_eq!(*value, 'Z');
}

/// 验证 CharacterEditor get_as_text。
#[test]
fn character_editor_get_as_text() {
    let mut editor = vernal_beans::char_property_editor::CharacterEditor::new();
    editor.set_as_text("X").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("X"));
}

// ── 10. ByteArrayPropertyEditor set_value 测试 ───────────────────────────

/// 验证 ByteArrayPropertyEditor set_value。
#[test]
fn byte_array_property_editor_set_value() {
    let mut editor = vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor::new();
    let data = vec![72, 101, 108, 108, 111]; // "Hello"
    editor.set_value(Arc::new(data.clone()));
    let value = editor
        .get_value()
        .unwrap()
        .downcast_ref::<Vec<u8>>()
        .unwrap();
    assert_eq!(*value, data);
}

/// 验证 ByteArrayPropertyEditor target_type。
#[test]
fn byte_array_property_editor_target_type() {
    let editor = vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor::new();
    assert_eq!(editor.target_type(), std::any::TypeId::of::<Vec<u8>>());
}
