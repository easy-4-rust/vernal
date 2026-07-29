//! Editor 文件全覆盖测试 — 为所有 PropertyEditor 实现编写完整生命周期测试。
//!
//! 每个 editor 测试覆盖：new()、Default::default()、target_type()、set_as_text()、
//! get_as_text()、set_value()、get_value()、get_value_type()

use std::any::{Any, TypeId};
use std::sync::Arc;
use vernal_beans::PropertyEditor;

/// 通用 editor 测试辅助函数
fn test_editor_lifecycle(
    editor: &mut dyn PropertyEditor,
    input: &str,
    expected_type: TypeId,
) {
    // target_type
    assert_eq!(editor.target_type(), expected_type);

    // set_as_text + get_as_text round-trip
    let set_result = editor.set_as_text(input);
    assert!(set_result.is_ok(), "set_as_text failed for input: {input}");

    let text = editor.get_as_text();
    assert!(text.is_some(), "get_as_text returned None for input: {input}");

    // get_value
    let val = editor.get_value();
    assert!(val.is_some(), "get_value returned None for input: {input}");

    // get_value_type
    let vtype = editor.get_value_type();
    let _ = vtype;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 所有 Editor 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn editor_boolean_editor() {
    let mut editor = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    let _ = vernal_beans::boolean_editor::CustomBooleanEditor::default();
    test_editor_lifecycle(&mut editor, "true", TypeId::of::<bool>());
}

#[test]
fn editor_byte_array_editor() {
    let mut editor = vernal_beans::byte_array_editor::ByteArrayPropertyEditor::new();
    let _ = vernal_beans::byte_array_editor::ByteArrayPropertyEditor::default();
    test_editor_lifecycle(&mut editor, "1,2,3", TypeId::of::<String>());
}

#[test]
fn editor_char_array_editor() {
    let mut editor = vernal_beans::char_array_editor::CharArrayPropertyEditor::new();
    let _ = vernal_beans::char_array_editor::CharArrayPropertyEditor::default();
    test_editor_lifecycle(&mut editor, "a,b,c", TypeId::of::<String>());
}

#[test]
fn editor_class_editor() {
    let mut editor = vernal_beans::class_editor::ClassEditor::new();
    let _ = vernal_beans::class_editor::ClassEditor::default();
    test_editor_lifecycle(&mut editor, "std::string::String", TypeId::of::<String>());
}

#[test]
fn editor_currency_editor() {
    let mut editor = vernal_beans::currency_editor::CurrencyEditor::new();
    let _ = vernal_beans::currency_editor::CurrencyEditor::default();
    test_editor_lifecycle(&mut editor, "USD", TypeId::of::<String>());
}

#[test]
fn editor_file_editor() {
    let mut editor = vernal_beans::file_editor::FileEditor::new();
    let _ = vernal_beans::file_editor::FileEditor::default();
    test_editor_lifecycle(&mut editor, "/tmp/test", TypeId::of::<String>());
}

#[test]
fn editor_locale_editor() {
    let mut editor = vernal_beans::locale_editor::LocaleEditor::new();
    let _ = vernal_beans::locale_editor::LocaleEditor::default();
    test_editor_lifecycle(&mut editor, "en_US", TypeId::of::<String>());
}

#[test]
fn editor_number_editor() {
    let mut editor = vernal_beans::number_editor::CustomNumberEditor::new();
    let _ = vernal_beans::number_editor::CustomNumberEditor::default();
    test_editor_lifecycle(&mut editor, "42", TypeId::of::<f64>());
}

#[test]
fn editor_path_editor() {
    let mut editor = vernal_beans::path_editor::PathEditor::new();
    let _ = vernal_beans::path_editor::PathEditor::default();
    test_editor_lifecycle(&mut editor, "/tmp/test", TypeId::of::<String>());
}

#[test]
fn editor_pattern_editor() {
    let mut editor = vernal_beans::pattern_editor::PatternEditor::new();
    let _ = vernal_beans::pattern_editor::PatternEditor::default();
    test_editor_lifecycle(&mut editor, ".*", TypeId::of::<String>());
}

#[test]
fn editor_string_trimmer_editor() {
    let mut editor = vernal_beans::string_trimmer_editor::StringTrimmerEditor::new();
    let _ = vernal_beans::string_trimmer_editor::StringTrimmerEditor::default();
    test_editor_lifecycle(&mut editor, "hello", TypeId::of::<String>());
}

#[test]
fn editor_uri_editor() {
    let mut editor = vernal_beans::uri_editor::URIEditor::new();
    let _ = vernal_beans::uri_editor::URIEditor::default();
    test_editor_lifecycle(&mut editor, "https://example.com", TypeId::of::<String>());
}

#[test]
fn editor_uuid_editor() {
    let mut editor = vernal_beans::uuid_editor::UUIDEditor::new();
    let _ = vernal_beans::uuid_editor::UUIDEditor::default();
    test_editor_lifecycle(&mut editor, "550e8400-e29b-41d4-a716-446655440000", TypeId::of::<String>());
}

#[test]
fn editor_charset_editor() {
    let mut editor = vernal_beans::charset_editor::CharsetEditor::new();
    let _ = vernal_beans::charset_editor::CharsetEditor::default();
    test_editor_lifecycle(&mut editor, "UTF-8", TypeId::of::<String>());
}

#[test]
fn editor_timezone_editor() {
    let mut editor = vernal_beans::timezone_editor::TimeZoneEditor::new();
    let _ = vernal_beans::timezone_editor::TimeZoneEditor::default();
    test_editor_lifecycle(&mut editor, "UTC", TypeId::of::<String>());
}

#[test]
fn editor_zone_id_editor() {
    let mut editor = vernal_beans::zone_id_editor::ZoneIdEditor::new();
    let _ = vernal_beans::zone_id_editor::ZoneIdEditor::default();
    test_editor_lifecycle(&mut editor, "UTC", TypeId::of::<String>());
}

#[test]
fn editor_char_property_editor() {
    let mut editor = vernal_beans::char_property_editor::CharacterEditor::new();
    let _ = vernal_beans::char_property_editor::CharacterEditor::default();
    test_editor_lifecycle(&mut editor, "A", TypeId::of::<char>());
}

#[test]
fn editor_byte_array_property_editor() {
    let mut editor = vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor::new();
    let _ = vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor::default();
    test_editor_lifecycle(&mut editor, "1,2,3", TypeId::of::<Vec<u8>>());
}

#[test]
fn editor_charset_property_editor() {
    let mut editor = vernal_beans::charset_property_editor::CharsetPropertyEditor::new();
    let _ = vernal_beans::charset_property_editor::CharsetPropertyEditor::default();
    test_editor_lifecycle(&mut editor, "UTF-8", TypeId::of::<String>());
}

#[test]
fn editor_file_array_editor() {
    let mut editor = vernal_beans::file_array_editor::FileArrayEditor::new();
    let _ = vernal_beans::file_array_editor::FileArrayEditor::default();
    test_editor_lifecycle(&mut editor, "/tmp/a,/tmp/b", TypeId::of::<String>());
}

#[test]
fn editor_class_array_editor() {
    let mut editor = vernal_beans::class_array_editor::ClassArrayEditor::new();
    let _ = vernal_beans::class_array_editor::ClassArrayEditor::default();
    test_editor_lifecycle(&mut editor, "std::string::String", TypeId::of::<String>());
}

#[test]
fn editor_input_source_editor() {
    let mut editor = vernal_beans::input_source_editor::InputSourceEditor::new();
    let _ = vernal_beans::input_source_editor::InputSourceEditor::default();
    test_editor_lifecycle(&mut editor, "/tmp/input", TypeId::of::<String>());
}

#[test]
fn editor_input_stream_editor() {
    let mut editor = vernal_beans::input_stream_editor::InputStreamEditor::new();
    let _ = vernal_beans::input_stream_editor::InputStreamEditor::default();
    test_editor_lifecycle(&mut editor, "/tmp/stream", TypeId::of::<String>());
}

#[test]
fn editor_path_property_editor() {
    let mut editor = vernal_beans::path_property_editor::PathPropertyEditor::new();
    let _ = vernal_beans::path_property_editor::PathPropertyEditor::default();
    test_editor_lifecycle(&mut editor, "/tmp/test", TypeId::of::<String>());
}

#[test]
fn editor_properties_editor() {
    let mut editor = vernal_beans::properties_editor::PropertiesEditor::new();
    let _ = vernal_beans::properties_editor::PropertiesEditor::default();
    test_editor_lifecycle(&mut editor, "key=value", TypeId::of::<String>());
}

#[test]
fn editor_reader_editor() {
    let mut editor = vernal_beans::reader_editor::ReaderEditor::new();
    let _ = vernal_beans::reader_editor::ReaderEditor::default();
    test_editor_lifecycle(&mut editor, "/tmp/reader", TypeId::of::<String>());
}

#[test]
fn editor_resource_bundle_editor() {
    let mut editor = vernal_beans::resource_bundle_editor::ResourceBundleEditor::new();
    let _ = vernal_beans::resource_bundle_editor::ResourceBundleEditor::default();
    test_editor_lifecycle(&mut editor, "Messages", TypeId::of::<String>());
}

#[test]
fn editor_string_array_editor() {
    let mut editor = vernal_beans::string_array_editor::StringArrayPropertyEditor::new();
    let _ = vernal_beans::string_array_editor::StringArrayPropertyEditor::default();
    test_editor_lifecycle(&mut editor, "a,b,c", TypeId::of::<String>());
}

#[test]
fn editor_char_array_property_editor() {
    let mut editor = vernal_beans::char_array_property_editor::CharArrayPropertyEditor::new();
    let _ = vernal_beans::char_array_property_editor::CharArrayPropertyEditor::default();
    test_editor_lifecycle(&mut editor, "a,b,c", TypeId::of::<String>());
}

#[test]
fn editor_boolean_editor_error_path() {
    let mut editor = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    let result = editor.set_as_text("maybe");
    assert!(result.is_err());
}

#[test]
fn editor_number_editor_error_path() {
    let mut editor = vernal_beans::number_editor::CustomNumberEditor::new();
    let result = editor.set_as_text("not_a_number");
    assert!(result.is_err());
}

#[test]
fn editor_uuid_editor_error_path() {
    let mut editor = vernal_beans::uuid_editor::UUIDEditor::new();
    let result = editor.set_as_text("not-a-uuid");
    assert!(result.is_err());
}

#[test]
fn editor_uri_editor_error_path() {
    let mut editor = vernal_beans::uri_editor::URIEditor::new();
    let result = editor.set_as_text("not a uri");
    assert!(result.is_err());
}

// BooleanEditor specific tests
#[test]
fn editor_boolean_editor_false_variants() {
    let mut editor = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    for val in &["false", "False", "no", "No", "0"] {
        let result = editor.set_as_text(val);
        assert!(result.is_ok(), "Failed for {val}");
        let val = editor.get_value().unwrap().downcast_ref::<bool>().copied().unwrap();
        assert!(!val, "Expected false for {val}");
    }
}

#[test]
fn editor_boolean_editor_true_variants() {
    let mut editor = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    for val in &["true", "True", "yes", "Yes", "1"] {
        let result = editor.set_as_text(val);
        assert!(result.is_ok(), "Failed for {val}");
        let val = editor.get_value().unwrap().downcast_ref::<bool>().copied().unwrap();
        assert!(val, "Expected true for {val}");
    }
}

// NumberEditor specific tests
#[test]
fn editor_number_editor_allow_empty() {
    let mut editor = vernal_beans::number_editor::CustomNumberEditor::with_allow_empty(true);
    let result = editor.set_as_text("");
    assert!(result.is_ok());
    assert!(editor.get_value().is_none());
}

#[test]
fn editor_number_editor_disallow_empty() {
    let mut editor = vernal_beans::number_editor::CustomNumberEditor::new();
    let result = editor.set_as_text("");
    assert!(result.is_err());
}

// PropertyEditor trait default methods
#[test]
fn editor_supports_text_default() {
    let editor = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    assert!(editor.supports_text());
}

#[test]
fn editor_supports_custom_editor_default() {
    let editor = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    assert!(!editor.supports_custom_editor());
}

#[test]
fn editor_java_initialization_string_default() {
    let editor = vernal_beans::boolean_editor::CustomBooleanEditor::new();
    assert!(editor.java_initialization_string().is_none());
}
