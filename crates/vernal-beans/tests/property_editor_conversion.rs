//! PropertyEditor + ConversionService 集成测试。
//!
//! 参照 Spring Framework 7.0.8 的 `propertyeditors` 包测试场景。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::boolean_editor::CustomBooleanEditor;
use vernal_beans::conversion_service::{ConversionService, Converter, DefaultConversionService};
use vernal_beans::number_editor::CustomNumberEditor;
use vernal_beans::property_editor::PropertyEditor;
use vernal_beans::StringTrimmerEditor;
use vernal_beans::URIEditor;
use vernal_beans::UUIDEditor;

// ── StringTrimmerEditor 测试 ─────────────────────────────────────────────

/// 参照 Spring `StringTrimmerEditorTests`：验证修剪行为。
#[test]
fn string_trimmer_editor_basic() {
    let mut editor = StringTrimmerEditor::new();
    editor.set_as_text("  hello  ").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("hello"));
}

/// 验证空字符串转 None。
#[test]
fn string_trimmer_editor_empty_to_null() {
    let mut editor = StringTrimmerEditor::new();
    editor.set_as_text("   ").unwrap();
    assert_eq!(editor.get_as_text(), None);
}

/// 验证空字符串不转 None（allow_empty=false 但空字符串仍为 None）。
#[test]
fn string_trimmer_editor_empty_preserved() {
    let mut editor = StringTrimmerEditor::with_empty_as_null(false);
    editor.set_as_text("").unwrap();
    // 空字符串修剪后为空，但 empty_as_null=false 时仍为 Some("")
    assert_eq!(editor.get_as_text(), Some("".to_string()));
}

/// 验证 set_value / get_value。
#[test]
fn string_trimmer_editor_set_get_value() {
    let mut editor = StringTrimmerEditor::new();
    editor.set_value(Arc::new("  world  ".to_string()));
    let value = editor
        .get_value()
        .unwrap()
        .downcast_ref::<String>()
        .unwrap();
    assert_eq!(value, "world");
}

/// 验证 supports_text 返回 true。
#[test]
fn string_trimmer_editor_supports_text() {
    let editor = StringTrimmerEditor::new();
    assert!(editor.supports_text());
}

/// 验证 target_type 是 String。
#[test]
fn string_trimmer_editor_target_type() {
    let editor = StringTrimmerEditor::new();
    assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
}

// ── CustomBooleanEditor 测试 ─────────────────────────────────────────────

/// 参照 Spring `CustomBooleanEditorTests`：验证 "true"/"false"。
#[test]
fn boolean_editor_true_false() {
    let mut editor = CustomBooleanEditor::new();
    editor.set_as_text("true").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(*value);

    editor.set_as_text("false").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(!*value);
}

/// 验证 "yes"/"no"。
#[test]
fn boolean_editor_yes_no() {
    let mut editor = CustomBooleanEditor::new();
    editor.set_as_text("yes").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(*value);

    editor.set_as_text("no").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(!*value);
}

/// 验证 "1"/"0"。
#[test]
fn boolean_editor_one_zero() {
    let mut editor = CustomBooleanEditor::new();
    editor.set_as_text("1").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(*value);

    editor.set_as_text("0").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(!*value);
}

/// 验证大小写不敏感。
#[test]
fn boolean_editor_case_insensitive() {
    let mut editor = CustomBooleanEditor::new();
    editor.set_as_text("TRUE").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(*value);

    editor.set_as_text("Yes").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(*value);
}

/// 验证无效输入返回错误。
#[test]
fn boolean_editor_invalid_input() {
    let mut editor = CustomBooleanEditor::new();
    let result = editor.set_as_text("maybe");
    assert!(result.is_err());
}

/// 验证空字符串处理。
#[test]
fn boolean_editor_empty_string() {
    let mut editor = CustomBooleanEditor::new();
    let result = editor.set_as_text("");
    assert!(result.is_err()); // 不允许空值

    let mut editor = CustomBooleanEditor::with_allow_empty(true);
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none()); // 空值转为 None
}

/// 验证 get_as_text。
#[test]
fn boolean_editor_get_as_text() {
    let mut editor = CustomBooleanEditor::new();
    editor.set_as_text("true").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("true"));

    editor.set_as_text("false").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("false"));
}

/// 验证 set_value。
#[test]
fn boolean_editor_set_value() {
    let mut editor = CustomBooleanEditor::new();
    editor.set_value(Arc::new(true));
    let value = editor.get_value().unwrap().downcast_ref::<bool>().unwrap();
    assert!(*value);
}

// ── CustomNumberEditor 测试 ──────────────────────────────────────────────

/// 参照 Spring `CustomNumberEditorTests`：验证整数解析。
#[test]
fn number_editor_integer() {
    let mut editor = CustomNumberEditor::new();
    editor.set_as_text("42").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
    assert_eq!(*value, 42.0);
}

/// 验证浮点数解析。
#[test]
fn number_editor_float() {
    let mut editor = CustomNumberEditor::new();
    editor.set_as_text("3.14").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
    assert!((value - 3.14).abs() < 0.001);
}

/// 验证负数解析。
#[test]
fn number_editor_negative() {
    let mut editor = CustomNumberEditor::new();
    editor.set_as_text("-100").unwrap();
    let value = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
    assert_eq!(*value, -100.0);
}

/// 验证无效输入。
#[test]
fn number_editor_invalid_input() {
    let mut editor = CustomNumberEditor::new();
    let result = editor.set_as_text("abc");
    assert!(result.is_err());
}

/// 验证空字符串处理。
#[test]
fn number_editor_empty_string() {
    let mut editor = CustomNumberEditor::new();
    let result = editor.set_as_text("");
    assert!(result.is_err());

    let mut editor = CustomNumberEditor::with_allow_empty(true);
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none());
}

/// 验证 get_as_text 整数格式。
#[test]
fn number_editor_get_as_text_integer() {
    let mut editor = CustomNumberEditor::new();
    editor.set_as_text("42").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("42"));
}

/// 验证 get_as_text 浮点格式。
#[test]
fn number_editor_get_as_text_float() {
    let mut editor = CustomNumberEditor::new();
    editor.set_as_text("3.14").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("3.14"));
}

/// 验证 set_value 从 i32。
#[test]
fn number_editor_set_value_i32() {
    let mut editor = CustomNumberEditor::new();
    editor.set_value(Arc::new(42i32));
    let value = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
    assert_eq!(*value, 42.0);
}

/// 验证 set_value 从 i64。
#[test]
fn number_editor_set_value_i64() {
    let mut editor = CustomNumberEditor::new();
    editor.set_value(Arc::new(100i64));
    let value = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
    assert_eq!(*value, 100.0);
}

/// 验证 set_value 从 f64。
#[test]
fn number_editor_set_value_f64() {
    let mut editor = CustomNumberEditor::new();
    editor.set_value(Arc::new(2.718f64));
    let value = editor.get_value().unwrap().downcast_ref::<f64>().unwrap();
    assert!((value - 2.718).abs() < 0.001);
}

// ── URIEditor 测试 ───────────────────────────────────────────────────────

/// 参照 Spring `URIEditorTests`：验证 URI 解析。
#[test]
fn uri_editor_basic() {
    let mut editor = URIEditor::new();
    editor.set_as_text("https://example.com").unwrap();
    assert_eq!(editor.get_as_text().as_deref(), Some("https://example.com"));
}

/// 验证文件 URI。
#[test]
fn uri_editor_file_uri() {
    let mut editor = URIEditor::new();
    editor.set_as_text("file:///tmp/test.txt").unwrap();
    assert_eq!(
        editor.get_as_text().as_deref(),
        Some("file:///tmp/test.txt")
    );
}

/// 验证无效 URI。
#[test]
fn uri_editor_invalid() {
    let mut editor = URIEditor::new();
    let result = editor.set_as_text("not a uri");
    assert!(result.is_err());
}

/// 验证空字符串。
#[test]
fn uri_editor_empty() {
    let mut editor = URIEditor::new();
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none());
}

// ── UUIDEditor 测试 ──────────────────────────────────────────────────────

/// 参照 Spring `UUIDEditorTests`：验证 UUID 解析。
#[test]
fn uuid_editor_basic() {
    let mut editor = UUIDEditor::new();
    editor
        .set_as_text("550e8400-e29b-41d4-a716-446655440000")
        .unwrap();
    assert_eq!(
        editor.get_as_text().as_deref(),
        Some("550e8400-e29b-41d4-a716-446655440000")
    );
}

/// 验证无效 UUID。
#[test]
fn uuid_editor_invalid() {
    let mut editor = UUIDEditor::new();
    let result = editor.set_as_text("not-a-uuid");
    assert!(result.is_err());
}

/// 验证格式错误的 UUID。
#[test]
fn uuid_editor_wrong_format() {
    let mut editor = UUIDEditor::new();
    let result = editor.set_as_text("550e8400-e29b-41d4-a716"); // 太短
    assert!(result.is_err());
}

/// 验证空字符串。
#[test]
fn uuid_editor_empty() {
    let mut editor = UUIDEditor::new();
    editor.set_as_text("").unwrap();
    assert!(editor.get_value().is_none());
}

// ── DefaultConversionService 测试 ────────────────────────────────────────

/// 参照 Spring `ConversionServiceTests`：验证 String → i32。
#[test]
fn conversion_service_string_to_i32() {
    let service = DefaultConversionService::new();
    let source = "42".to_string();
    let result = service
        .convert(&source, std::any::TypeId::of::<i32>())
        .unwrap();
    let value = result.downcast_ref::<i32>().unwrap();
    assert_eq!(*value, 42);
}

/// 验证 String → i64。
#[test]
fn conversion_service_string_to_i64() {
    let service = DefaultConversionService::new();
    let source = "1000000".to_string();
    let result = service
        .convert(&source, std::any::TypeId::of::<i64>())
        .unwrap();
    let value = result.downcast_ref::<i64>().unwrap();
    assert_eq!(*value, 1000000);
}

/// 验证 String → f64。
#[test]
fn conversion_service_string_to_f64() {
    let service = DefaultConversionService::new();
    let source = "3.14".to_string();
    let result = service
        .convert(&source, std::any::TypeId::of::<f64>())
        .unwrap();
    let value = result.downcast_ref::<f64>().unwrap();
    assert!((value - 3.14).abs() < 0.001);
}

/// 验证 String → bool。
#[test]
fn conversion_service_string_to_bool() {
    let service = DefaultConversionService::new();
    let source = "true".to_string();
    let result = service
        .convert(&source, std::any::TypeId::of::<bool>())
        .unwrap();
    let value = result.downcast_ref::<bool>().unwrap();
    assert!(*value);
}

/// 验证 i32 → String。
#[test]
fn conversion_service_i32_to_string() {
    let service = DefaultConversionService::new();
    let source = 42i32;
    let result = service
        .convert(&source, std::any::TypeId::of::<String>())
        .unwrap();
    let value = result.downcast_ref::<String>().unwrap();
    assert_eq!(value, "42");
}

/// 验证 bool → String。
#[test]
fn conversion_service_bool_to_string() {
    let service = DefaultConversionService::new();
    let source = true;
    let result = service
        .convert(&source, std::any::TypeId::of::<String>())
        .unwrap();
    let value = result.downcast_ref::<String>().unwrap();
    assert_eq!(value, "true");
}

/// 验证 can_convert。
#[test]
fn conversion_service_can_convert() {
    let service = DefaultConversionService::new();
    assert!(service.can_convert(
        std::any::TypeId::of::<String>(),
        std::any::TypeId::of::<i32>()
    ));
    assert!(service.can_convert(
        std::any::TypeId::of::<String>(),
        std::any::TypeId::of::<bool>()
    ));
    // i32 → String 有内置转换器
    assert!(service.can_convert(
        std::any::TypeId::of::<i32>(),
        std::any::TypeId::of::<String>()
    ));
    assert!(!service.can_convert(
        std::any::TypeId::of::<Vec<u8>>(),
        std::any::TypeId::of::<i32>()
    ));
}

/// 验证转换失败时返回错误。
#[test]
fn conversion_service_invalid_conversion() {
    let service = DefaultConversionService::new();
    let source = "abc".to_string();
    let result = service.convert(&source, std::any::TypeId::of::<i32>());
    assert!(result.is_err());
}

/// 验证自定义转换器注册。
#[test]
fn conversion_service_custom_converter() {
    struct StringToUpperConverter;
    impl Converter for StringToUpperConverter {
        fn source_type(&self) -> std::any::TypeId {
            std::any::TypeId::of::<String>()
        }
        fn target_type(&self) -> std::any::TypeId {
            std::any::TypeId::of::<String>()
        }
        fn convert(
            &self,
            source: &dyn Any,
        ) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
            let s = source.downcast_ref::<String>().ok_or("Not a String")?;
            Ok(Box::new(s.to_uppercase()))
        }
    }

    let mut service = DefaultConversionService::new();
    service.register(Arc::new(StringToUpperConverter));

    let source = "hello".to_string();
    let result = service
        .convert(&source, std::any::TypeId::of::<String>())
        .unwrap();
    let value = result.downcast_ref::<String>().unwrap();
    assert_eq!(value, "HELLO");
}

// ── PropertyEditor Registry 测试 ─────────────────────────────────────────

/// 验证 PropertyEditor 的基本接口。
#[test]
fn property_editor_interface() {
    let editor = StringTrimmerEditor::new();
    assert!(editor.supports_text());
    assert!(!editor.supports_custom_editor());
    assert_eq!(editor.target_type(), std::any::TypeId::of::<String>());
    assert!(editor.java_initialization_string().is_none());
}

/// 验证 CustomBooleanEditor 接口。
#[test]
fn boolean_editor_interface() {
    let editor = CustomBooleanEditor::new();
    assert!(editor.supports_text());
    assert!(!editor.supports_custom_editor());
    assert_eq!(editor.target_type(), std::any::TypeId::of::<bool>());
}

/// 验证 CustomNumberEditor 接口。
#[test]
fn number_editor_interface() {
    let editor = CustomNumberEditor::new();
    assert!(editor.supports_text());
    assert!(!editor.supports_custom_editor());
    assert_eq!(editor.target_type(), std::any::TypeId::of::<f64>());
}
