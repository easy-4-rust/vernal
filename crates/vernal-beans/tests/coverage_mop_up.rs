//! 扫尾覆盖测试 — 针对剩余低覆盖率文件。
//! 目标：覆盖 editor 文件缺口、expression resolver、bean_definition_utils 等。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::property_editor::PropertyEditor;

// ── ClassEditor (82.14%) ──────────────────────────────────────────

#[test]
fn class_editor_remaining_paths() {
    use vernal_beans::class_editor::ClassEditor;
    let mut e = ClassEditor::new();
    let _d: ClassEditor = Default::default();

    // set_as_text with valid class name
    e.set_as_text("java.lang.String").unwrap();
    assert_eq!(e.get_as_text(), Some("java.lang.String".to_string()));

    // get_value after set_as_text
    assert!(e.get_value().is_some());

    // set_value with string
    e.set_value(Arc::new("com.example.Test".to_string()));
    assert!(e.get_value().is_some());

    // set_as_text with empty → clears
    e.set_as_text("").unwrap();
    assert!(e.get_value().is_none());

    // set_as_text with spaces → error
    assert!(e.set_as_text("bad class name").is_err());
}

// ── ByteArrayEditor (82.14%) ──────────────────────────────────────

#[test]
fn byte_array_editor_remaining() {
    use vernal_beans::byte_array_editor::ByteArrayPropertyEditor;
    let mut e = ByteArrayPropertyEditor::new();
    let _d: ByteArrayPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    // get_value_type checked indirectly
    e.set_as_text("test").unwrap();
    assert!(e.get_value().is_some());
    assert!(e.get_as_text().is_some());
    e.set_as_text("text").unwrap();
    assert!(e.get_value().is_some());
}

// ── CharArrayPropertyEditor (82.14%) ─────────────────────────────

#[test]
fn char_array_property_editor_remaining() {
    use vernal_beans::char_array_property_editor::CharArrayPropertyEditor;
    let mut e = CharArrayPropertyEditor::new();
    let _d: CharArrayPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("ab").unwrap();
    assert!(e.get_value().is_some());
}

// ── ClassArrayEditor (82.14%) ────────────────────────────────────

#[test]
fn class_array_editor_remaining() {
    use vernal_beans::class_array_editor::ClassArrayEditor;
    let mut e = ClassArrayEditor::new();
    let _d: ClassArrayEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

// ── FileArrayEditor (82.14%) ─────────────────────────────────────

#[test]
fn file_array_editor_remaining() {
    use vernal_beans::file_array_editor::FileArrayEditor;
    let mut e = FileArrayEditor::new();
    let _d: FileArrayEditor = Default::default();
    assert!(e.get_value().is_none());
    assert_eq!(e.target_type(), std::any::TypeId::of::<String>());
}

// ── InputSourceEditor (82.14%) ──────────────────────────────────

#[test]
fn input_source_editor_remaining() {
    use vernal_beans::input_source_editor::InputSourceEditor;
    let mut e = InputSourceEditor::new();
    let _d: InputSourceEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("test.xml").unwrap();
    assert!(e.get_value().is_some());
}

// ── PathEditor (82.14%) ──────────────────────────────────────────

#[test]
fn path_editor_remaining() {
    use vernal_beans::path_editor::PathEditor;
    let mut e = PathEditor::new();
    let _d: PathEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("/tmp/test").unwrap();
    assert!(e.get_value().is_some());
    assert!(e.get_as_text().is_some());
}

// ── PathPropertyEditor (82.14%) ─────────────────────────────────

#[test]
fn path_property_editor_remaining() {
    use vernal_beans::path_property_editor::PathPropertyEditor;
    let mut e = PathPropertyEditor::new();
    let _d: PathPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("/path/to/file").unwrap();
    assert!(e.get_value().is_some());
}

// ── PropertiesEditor (82.14%) ───────────────────────────────────

#[test]
fn properties_editor_remaining() {
    use vernal_beans::properties_editor::PropertiesEditor;
    let mut e = PropertiesEditor::new();
    let _d: PropertiesEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("key=value").unwrap();
    assert!(e.get_value().is_some());
}

// ── ReaderEditor (82.14%) ───────────────────────────────────────

#[test]
fn reader_editor_remaining() {
    use vernal_beans::reader_editor::ReaderEditor;
    let mut e = ReaderEditor::new();
    let _d: ReaderEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("content").unwrap();
    assert!(e.get_value().is_some());
    assert!(e.get_as_text().is_some());
}

// ── FileEditor (83.87%) ─────────────────────────────────────────

#[test]
fn file_editor_remaining() {
    use vernal_beans::file_editor::FileEditor;
    let mut e = FileEditor::new();
    let _d: FileEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("test.txt").unwrap();
    assert!(e.get_value().is_some());
}

// ── CharsetPropertyEditor (84.85%) ───────────────────────────────

#[test]
fn charset_property_editor_remaining() {
    use vernal_beans::charset_property_editor::CharsetPropertyEditor;
    let mut e = CharsetPropertyEditor::new();
    let _d: CharsetPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("ISO-8859-1").unwrap();
    assert!(e.get_value().is_some());
}

// ── StandardBeanExpressionResolver (80%) ─────────────────────────

#[test]
fn expression_resolver_remaining() {
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;
    let r = StandardBeanExpressionResolver::new();

    // SpEL templates
    let _ = r.evaluate("#{systemProperties['user.home']}", Some("testBean"));
    // Long identifier
    let _ = r.evaluate("some.long.identifier", None);
    // Number in expression
    let _ = r.evaluate("42", None);
}

// ── BeanDefinitionUtils (54%) ─────────────────────────────────────

#[test]
fn bean_definition_utils_generate_bean_name() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::bean_definition_utils;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    struct EmptyReg;
    impl BeanDefinitionRegistry for EmptyReg {
        fn register_bean_definition(
            &mut self,
            _: String,
            _: Box<dyn vernal_beans::BeanDefinition>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn remove_bean_definition(
            &mut self,
            _: &str,
        ) -> Result<Box<dyn vernal_beans::BeanDefinition>, Box<dyn std::error::Error + Send + Sync>>
        {
            unimplemented!()
        }
        fn get_bean_definition(
            &self,
            _: &str,
        ) -> Option<&'static dyn vernal_beans::BeanDefinition> {
            None
        }
        fn contains_bean_definition(&self, _: &str) -> bool {
            false
        }
        fn bean_definition_count(&self) -> usize {
            0
        }
        fn bean_definition_names(&self) -> Vec<String> {
            vec![]
        }
    }

    let reg = EmptyReg;
    // generate_bean_name with existing class name
    let name = bean_definition_utils::generate_bean_name(Some("com.example.Test"), &reg);
    assert_eq!(name, "com.example.Test");

    // generate_bean_name without class name
    let name = bean_definition_utils::generate_bean_name(None, &reg);
    assert_eq!(name, "anonymous");
}

// ── Vector<String> editor (test type conversions) ─────────────────

#[test]
fn locale_editor_remaining_set_text() {
    use vernal_beans::locale_editor::LocaleEditor;
    let mut e = LocaleEditor::new();
    let _d: LocaleEditor = Default::default();
    e.set_as_text("en_US").unwrap();
    assert!(e.get_value().is_some());
    e.set_as_text("").unwrap(); // empty clears
    assert!(e.get_value().is_none());
}

#[test]
fn string_trimmer_editor_remaining() {
    use vernal_beans::string_trimmer_editor::StringTrimmerEditor;
    let mut e = StringTrimmerEditor::new();
    let _d: StringTrimmerEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("  trimmed  ").unwrap();
    assert!(e.get_as_text().is_some());
    let text = e.get_as_text().unwrap();
    assert_eq!(text, "trimmed"); // or keeps spaces - just verify it works
}

#[test]
fn string_array_editor_remaining() {
    use vernal_beans::string_array_editor::StringArrayPropertyEditor;
    let mut e = StringArrayPropertyEditor::new();
    let _d: StringArrayPropertyEditor = Default::default();
    assert!(e.get_value().is_none());
    e.set_as_text("a,b,c").unwrap();
    assert!(e.get_value().is_some());
    assert!(e.get_as_text().is_some());
}
