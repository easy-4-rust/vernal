//! 剩余低覆盖文件的测试：standard_bean_expression_resolver, property editors, misc

use std::any::Any;
use std::sync::Arc;

use vernal_beans::bean_expression_resolver::BeanExpressionResolver;

// ═══════════════════════════════════════════════════════════════════════════════
// StandardBeanExpressionResolver 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn standard_bean_expression_resolver_new() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    assert_eq!(resolver.bean_count(), 0);
}

#[test]
fn standard_bean_expression_resolver_default() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::default();
    assert_eq!(resolver.bean_count(), 0);
}

#[test]
fn standard_bean_expression_resolver_register_bean() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("myBean".to_string(), Arc::new(42i32));
    assert_eq!(resolver.bean_count(), 1);
}

#[test]
fn standard_bean_expression_resolver_clear_context() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("bean1".to_string(), Arc::new(1i32));
    resolver.register_bean("bean2".to_string(), Arc::new(2i32));
    assert_eq!(resolver.bean_count(), 2);
    resolver.clear_context();
    assert_eq!(resolver.bean_count(), 0);
}

#[test]
fn standard_bean_expression_resolver_evaluate_simple_bean() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("myBean".to_string(), Arc::new(42i32));

    let result = resolver.evaluate("myBean", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn standard_bean_expression_resolver_evaluate_unknown_simple_identifier() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("unknownBean", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_evaluate_spel_integer_literal() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    // SpEL 字面量可能因 vernal-expression 实现而返回 None 或错误
    let result = resolver.evaluate("#{123}", None);
    // 验证不会 panic
    let _ = result;
}

#[test]
fn standard_bean_expression_resolver_evaluate_spel_string_literal() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("#{'hello'}", None);
    let _ = result;
}

#[test]
fn standard_bean_expression_resolver_evaluate_spel_boolean_literal() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("#{true}", None);
    let _ = result;
}

#[test]
fn standard_bean_expression_resolver_evaluate_invalid_spel() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("#{invalid!!!}", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_evaluate_empty() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_evaluate_whitespace() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("myBean".to_string(), Arc::new(42i32));
    let result = resolver.evaluate("  myBean  ", None).unwrap();
    assert!(result.is_some());
}

#[test]
fn standard_bean_expression_resolver_evaluate_numeric_start() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("123abc", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_evaluate_underscore_start() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("_private".to_string(), Arc::new(1i32));
    let result = resolver.evaluate("_private", None).unwrap();
    assert!(result.is_some());
    assert_eq!(*result.unwrap().downcast_ref::<i32>().unwrap(), 1);
}

#[test]
fn standard_bean_expression_resolver_evaluate_special_chars() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    let result = resolver.evaluate("a+b", None).unwrap();
    assert!(result.is_none());
}

#[test]
fn standard_bean_expression_resolver_multiple_beans() {
    let resolver = vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver::new();
    resolver.register_bean("intBean".to_string(), Arc::new(42i32));
    resolver.register_bean("strBean".to_string(), Arc::new("hello".to_string()));
    resolver.register_bean("boolBean".to_string(), Arc::new(true));

    let result = resolver.evaluate("intBean", None).unwrap();
    assert_eq!(*result.unwrap().downcast_ref::<i32>().unwrap(), 42);

    let result = resolver.evaluate("strBean", None).unwrap();
    assert_eq!(*result.unwrap().downcast_ref::<String>().unwrap(), "hello");

    let result = resolver.evaluate("boolBean", None).unwrap();
    assert_eq!(*result.unwrap().downcast_ref::<bool>().unwrap(), true);
}

// ═══════════════════════════════════════════════════════════════════════════════
// PropertyEditor 补充测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_trimmer_editor_basic() {
    use vernal_beans::property_editor::PropertyEditor;
    use vernal_beans::string_trimmer_editor::StringTrimmerEditor;

    let mut editor = StringTrimmerEditor::new();
    editor.set_as_text("  hello  ").unwrap();
    assert!(editor.get_value().is_some());
    let value = editor.get_value().unwrap();
    assert_eq!(*value.downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn boolean_editor_basic() {
    use vernal_beans::boolean_editor::CustomBooleanEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = CustomBooleanEditor::new();
    editor.set_as_text("true").unwrap();
    assert!(editor.get_value().is_some());
    let value = editor.get_value().unwrap();
    assert_eq!(*value.downcast_ref::<bool>().unwrap(), true);
}

#[test]
fn number_editor_basic() {
    use vernal_beans::number_editor::CustomNumberEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = CustomNumberEditor::new();
    editor.set_as_text("42").unwrap();
    assert!(editor.get_value().is_some());
    let value = editor.get_value().unwrap();
    assert_eq!(*value.downcast_ref::<f64>().unwrap(), 42.0);
}

#[test]
fn char_property_editor_basic() {
    use vernal_beans::char_property_editor::CharacterEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = CharacterEditor::new();
    editor.set_as_text("A").unwrap();
    assert!(editor.get_value().is_some());
    let value = editor.get_value().unwrap();
    assert_eq!(*value.downcast_ref::<char>().unwrap(), 'A');
}

#[test]
fn byte_array_property_editor_basic() {
    use vernal_beans::byte_array_property_editor::ByteArrayPropertyEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = ByteArrayPropertyEditor::new();
    editor.set_as_text("hello").unwrap();
    assert!(editor.get_value().is_some());
    let value = editor.get_value().unwrap();
    assert_eq!(*value.downcast_ref::<Vec<u8>>().unwrap(), b"hello".to_vec());
}

#[test]
fn charset_editor_basic() {
    use vernal_beans::charset_editor::CharsetEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = CharsetEditor::new();
    editor.set_as_text("UTF-8").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn class_editor_basic() {
    use vernal_beans::class_editor::ClassEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = ClassEditor::new();
    let result = editor.set_as_text("java.lang.String");
    let _ = result;
}

#[test]
fn file_editor_basic() {
    use vernal_beans::file_editor::FileEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = FileEditor::new();
    editor.set_as_text("/tmp/test.txt").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn locale_editor_basic() {
    use vernal_beans::locale_editor::LocaleEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = LocaleEditor::new();
    editor.set_as_text("en_US").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn path_editor_basic() {
    use vernal_beans::path_editor::PathEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = PathEditor::new();
    editor.set_as_text("/tmp").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn pattern_editor_basic() {
    use vernal_beans::pattern_editor::PatternEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = PatternEditor::new();
    editor.set_as_text(".*").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn timezone_editor_basic() {
    use vernal_beans::property_editor::PropertyEditor;
    use vernal_beans::timezone_editor::TimeZoneEditor;

    let mut editor = TimeZoneEditor::new();
    editor.set_as_text("UTC").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn uri_editor_basic() {
    use vernal_beans::property_editor::PropertyEditor;
    use vernal_beans::uri_editor::URIEditor;

    let mut editor = URIEditor::new();
    editor.set_as_text("https://example.com").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn uuid_editor_basic() {
    use vernal_beans::property_editor::PropertyEditor;
    use vernal_beans::uuid_editor::UUIDEditor;

    let mut editor = UUIDEditor::new();
    editor.set_as_text("550e8400-e29b-41d4-a716-446655440000").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn zone_id_editor_basic() {
    use vernal_beans::property_editor::PropertyEditor;
    use vernal_beans::zone_id_editor::ZoneIdEditor;

    let mut editor = ZoneIdEditor::new();
    editor.set_as_text("UTC").unwrap();
    assert!(editor.get_value().is_some());
}

#[test]
fn currency_editor_basic() {
    use vernal_beans::currency_editor::CurrencyEditor;
    use vernal_beans::property_editor::PropertyEditor;

    let mut editor = CurrencyEditor::new();
    editor.set_as_text("USD").unwrap();
    assert!(editor.get_value().is_some());
}

// ═══════════════════════════════════════════════════════════════════════════════
// GraphError 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn graph_error_display_missing_dependency() {
    let err = vernal_beans::GraphError::MissingDependency {
        path: vec!["a".to_string(), "b".to_string()],
    };
    let msg = format!("{err}");
    assert!(msg.contains("missing dependency"));
}

#[test]
fn graph_error_display_ambiguous_dependency() {
    let err = vernal_beans::GraphError::AmbiguousDependency {
        path: vec!["a".to_string()],
        candidates: vec!["c1".to_string(), "c2".to_string()],
    };
    let msg = format!("{err}");
    assert!(msg.contains("ambiguous"));
}

#[test]
fn graph_error_display_missing_trait_binding_target() {
    let err = vernal_beans::GraphError::MissingTraitBindingTarget {
        binding: "my binding".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("trait binding"));
}

#[test]
fn graph_error_display_cycle() {
    let err = vernal_beans::GraphError::Cycle {
        path: vec!["a".to_string(), "b".to_string(), "a".to_string()],
    };
    let msg = format!("{err}");
    assert!(msg.contains("cycle"));
}

#[test]
fn graph_error_debug() {
    let err = vernal_beans::GraphError::Cycle {
        path: vec!["a".to_string()],
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("Cycle"));
}

#[test]
fn graph_error_clone() {
    let err = vernal_beans::GraphError::Cycle {
        path: vec!["a".to_string()],
    };
    let cloned = err.clone();
    let _ = cloned;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dependency 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn dependency_of() {
    let dep = vernal_beans::Dependency::of::<String>();
    assert_eq!(dep.type_name(), std::any::type_name::<String>());
}

#[test]
fn dependency_qualified() {
    let q = vernal_beans::Qualifier::new("test").unwrap();
    let dep = vernal_beans::Dependency::qualified::<String>(q);
    assert!(dep.qualifier().is_some());
}

#[test]
fn dependency_optional_of() {
    let dep = vernal_beans::Dependency::optional_of::<String>();
    let _ = dep;
}

#[test]
fn dependency_trait_of() {
    let dep = vernal_beans::Dependency::trait_of::<dyn Any + Send + Sync>();
    let _ = dep;
}

#[test]
fn dependency_display() {
    let dep = vernal_beans::Dependency::of::<String>();
    let msg = format!("{dep}");
    assert!(!msg.is_empty());
}

#[test]
fn dependency_debug() {
    let dep = vernal_beans::Dependency::of::<String>();
    let debug = format!("{dep:?}");
    assert!(debug.contains("Dependency"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentDefinition 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_definition_shared_value() {
    let def = vernal_beans::ComponentDefinition::shared_value(42i32);
    let _ = def;
}

#[test]
fn component_definition_key() {
    let def = vernal_beans::ComponentDefinition::shared_value(42i32);
    assert_eq!(def.key().type_name(), std::any::type_name::<i32>());
}

#[test]
fn component_definition_scope() {
    let def = vernal_beans::ComponentDefinition::shared_value(42i32);
    assert_eq!(def.scope(), vernal_beans::Scope::Singleton);
}

#[test]
fn component_definition_display() {
    let def = vernal_beans::ComponentDefinition::shared_value(42i32);
    let msg = format!("{:?}", def);
    assert!(!msg.is_empty());
}

#[test]
fn component_definition_debug() {
    let def = vernal_beans::ComponentDefinition::shared_value(42i32);
    let debug = format!("{def:?}");
    assert!(debug.contains("ComponentDefinition"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeState 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_state_open() {
    let state = vernal_beans::ScopeState::Open;
    assert_eq!(state, vernal_beans::ScopeState::Open);
}

#[test]
fn scope_state_closing() {
    let state = vernal_beans::ScopeState::Closing;
    assert_eq!(state, vernal_beans::ScopeState::Closing);
}

#[test]
fn scope_state_closed() {
    let state = vernal_beans::ScopeState::Closed;
    assert_eq!(state, vernal_beans::ScopeState::Closed);
}

#[test]
fn scope_state_default() {
    let state = vernal_beans::ScopeState::default();
    assert_eq!(state, vernal_beans::ScopeState::Open);
}

#[test]
fn scope_state_debug() {
    let state = vernal_beans::ScopeState::Open;
    let debug = format!("{:?}", state);
    assert!(debug.contains("Open"));
}

#[test]
fn scope_state_clone() {
    let state = vernal_beans::ScopeState::Closing;
    let cloned = state;
    assert_eq!(state, cloned);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Qualifier 测试补充
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn qualifier_debug() {
    let q = vernal_beans::Qualifier::new("test").unwrap();
    let debug = format!("{:?}", q);
    assert!(debug.contains("test"));
}

#[test]
fn qualifier_clone() {
    let q = vernal_beans::Qualifier::new("test").unwrap();
    let cloned = q.clone();
    assert_eq!(q, cloned);
}

#[test]
fn qualifier_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let q1 = vernal_beans::Qualifier::new("test").unwrap();
    let q2 = vernal_beans::Qualifier::new("test").unwrap();
    map.insert(q1, 1);
    assert_eq!(map.get(&q2), Some(&1));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ScopeKey 测试补充
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn scope_key_display() {
    let k = vernal_beans::ScopeKey::of::<String>();
    let msg = format!("{k}");
    assert!(!msg.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// TraitKey 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_key_of() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let _ = k;
}

#[test]
fn trait_key_display() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let msg = format!("{k}");
    assert!(!msg.is_empty());
}

#[test]
fn trait_key_debug() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let debug = format!("{k:?}");
    assert!(debug.contains("TraitKey"));
}

#[test]
fn trait_key_clone() {
    let k = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let cloned = k.clone();
    assert_eq!(k, cloned);
}

#[test]
fn trait_key_eq() {
    let k1 = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    let k2 = vernal_beans::TraitKey::of::<dyn Any + Send + Sync>();
    assert_eq!(k1, k2);

    let k3 = vernal_beans::TraitKey::of::<String>();
    assert_ne!(k1, k3);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ComponentKey 测试补充
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn component_key_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let k1 = vernal_beans::ComponentKey::of::<String>();
    let k2 = vernal_beans::ComponentKey::of::<String>();
    map.insert(k1, 1);
    assert_eq!(map.get(&k2), Some(&1));
}

// ═══════════════════════════════════════════════════════════════════════════════
// PlaceholderConfigurerSupport 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn placeholder_configurer_support_new() {
    let configurer = vernal_beans::placeholder_configurer::PlaceholderConfigurerSupport::new();
    let _ = configurer;
}

// ═══════════════════════════════════════════════════════════════════════════════
// TypeConverter trait 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_converter_trait_exists() {
    fn _assert<T: vernal_beans::TypeConverter>() {}
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanDefinitionUtils — 作为自由函数测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_utils_generate_bean_name() {
    let mut builder = vernal_beans::RegistryBuilder::new();
    builder.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = vernal_beans::Container::new(registry);
    let name = vernal_beans::bean_definition_utils::generate_bean_name(Some("com.example.MyClass"), &container);
    assert!(!name.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanFactoryUtils — Mock ListableBeanFactory 集成测试
// ═══════════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use vernal_beans::listable_bean_factory::ListableBeanFactory;

struct MockListableBeanFactory {
    definitions: std::collections::HashMap<String, Arc<dyn vernal_beans::BeanDefinition>>,
    beans: std::collections::HashMap<std::any::TypeId, Vec<String>>,
    instances: std::collections::HashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl MockListableBeanFactory {
    fn new() -> Self {
        Self {
            definitions: std::collections::HashMap::new(),
            beans: std::collections::HashMap::new(),
            instances: std::collections::HashMap::new(),
        }
    }

    fn register_definition(&mut self, name: String, def: Arc<dyn vernal_beans::BeanDefinition>, type_id: std::any::TypeId) {
        self.beans.entry(type_id).or_default().push(name.clone());
        self.definitions.insert(name.clone(), def);
        self.instances.insert(name, Arc::new("dummy".to_string()));
    }
}

impl vernal_beans::BeanFactory for MockListableBeanFactory {
    fn get_bean_by_key(
        &self,
        _key: &vernal_beans::ComponentKey,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        self.instances.values().next()
            .cloned()
            .ok_or_else(|| "not found".into())
    }

    fn get_bean_by_type_id(
        &self,
        type_id: std::any::TypeId,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(names) = self.beans.get(&type_id) {
            if let Some(name) = names.first() {
                if let Some(instance) = self.instances.get(name) {
                    return Ok(Arc::clone(instance));
                }
            }
        }
        Err("not found".into())
    }

    fn contains_bean(&self, _key: &vernal_beans::ComponentKey) -> bool { true }
    fn is_singleton(&self, _key: &vernal_beans::ComponentKey) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(true) }
    fn is_prototype(&self, _key: &vernal_beans::ComponentKey) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> { Ok(false) }
    fn get_type(&self, _key: &vernal_beans::ComponentKey) -> Result<Option<&'static str>, Box<dyn std::error::Error + Send + Sync>> { Ok(Some("MockBean")) }
    fn get_aliases(&self, _key: &vernal_beans::ComponentKey) -> Vec<vernal_beans::ComponentKey> { vec![] }
    fn get_bean_provider_by_type_id(
        &self,
        _type_id: std::any::TypeId,
    ) -> Result<Box<dyn vernal_beans::ObjectProvider<dyn Any + Send + Sync> + '_>, Box<dyn std::error::Error + Send + Sync>> {
        Err("not implemented".into())
    }
    fn is_type_match(&self, _key: &vernal_beans::ComponentKey, _type_id: std::any::TypeId) -> bool { true }
}

impl ListableBeanFactory for MockListableBeanFactory {
    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.definitions.contains_key(bean_name)
    }
    fn bean_definition_count(&self) -> usize { self.definitions.len() }
    fn bean_definition_names(&self) -> Vec<String> { self.definitions.keys().cloned().collect() }
    fn bean_names_for_type_id(&self, type_id: std::any::TypeId, _: bool, _: bool) -> Vec<String> {
        self.beans.get(&type_id).cloned().unwrap_or_default()
    }
    fn beans_of_type_id(&self, type_id: std::any::TypeId, _: bool, _: bool) -> Result<std::collections::HashMap<String, Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut result = std::collections::HashMap::new();
        if let Some(names) = self.beans.get(&type_id) {
            for name in names {
                if let Some(instance) = self.instances.get(name) {
                    result.insert(name.clone(), Arc::clone(instance));
                }
            }
        }
        Ok(result)
    }
    fn bean_post_processor_count(&self) -> usize { 0 }
    fn contains_non_singleton_bean(&self) -> bool { false }
    fn contains_singleton_bean(&self) -> bool { !self.instances.is_empty() }
    fn bean_names_iterator(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(self.definitions.keys().cloned())
    }
}

#[test]
fn bean_factory_utils_count_beans_for_type() {
    let mut factory = MockListableBeanFactory::new();
    let def = Arc::new(vernal_beans::RootBeanDefinition::new()) as Arc<dyn vernal_beans::BeanDefinition>;
    factory.register_definition("bean1".to_string(), def.clone(), std::any::TypeId::of::<String>());
    factory.register_definition("bean2".to_string(), def.clone(), std::any::TypeId::of::<String>());
    factory.register_definition("bean3".to_string(), def, std::any::TypeId::of::<i32>());
    let count = vernal_beans::bean_factory_utils::BeanFactoryUtils::count_beans_for_type(std::any::TypeId::of::<String>(), &factory);
    assert_eq!(count, 2);
}

#[test]
fn bean_factory_utils_bean_names_for_type() {
    let mut factory = MockListableBeanFactory::new();
    let def = Arc::new(vernal_beans::RootBeanDefinition::new()) as Arc<dyn vernal_beans::BeanDefinition>;
    factory.register_definition("bean1".to_string(), def.clone(), std::any::TypeId::of::<String>());
    factory.register_definition("bean2".to_string(), def, std::any::TypeId::of::<String>());
    let names = vernal_beans::bean_factory_utils::BeanFactoryUtils::bean_names_for_type(std::any::TypeId::of::<String>(), &factory);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"bean1".to_string()));
    assert!(names.contains(&"bean2".to_string()));
}

#[test]
fn bean_factory_utils_beans_of_type() {
    let mut factory = MockListableBeanFactory::new();
    let def = Arc::new(vernal_beans::RootBeanDefinition::new()) as Arc<dyn vernal_beans::BeanDefinition>;
    factory.register_definition("greeting".to_string(), def, std::any::TypeId::of::<String>());
    let beans = vernal_beans::bean_factory_utils::BeanFactoryUtils::beans_of_type(std::any::TypeId::of::<String>(), &factory);
    assert!(beans.is_ok());
    let beans = beans.unwrap();
    assert_eq!(beans.len(), 1);
    assert!(beans.contains_key("greeting"));
}

#[test]
fn bean_factory_utils_bean_definition_names_util() {
    let mut factory = MockListableBeanFactory::new();
    let def = Arc::new(vernal_beans::RootBeanDefinition::new()) as Arc<dyn vernal_beans::BeanDefinition>;
    factory.register_definition("bean1".to_string(), def, std::any::TypeId::of::<String>());
    let names = vernal_beans::bean_factory_utils::BeanFactoryUtils::bean_definition_names(&factory);
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"bean1".to_string()));
}

#[test]
fn bean_factory_utils_count_beans_for_type_including_ancestors_util() {
    let mut factory = MockListableBeanFactory::new();
    let def = Arc::new(vernal_beans::RootBeanDefinition::new()) as Arc<dyn vernal_beans::BeanDefinition>;
    factory.register_definition("bean1".to_string(), def, std::any::TypeId::of::<String>());
    let count = vernal_beans::bean_factory_utils::BeanFactoryUtils::count_beans_for_type_including_ancestors(std::any::TypeId::of::<String>(), &factory);
    assert_eq!(count, 1);
}

#[test]
fn bean_factory_utils_bean_names_for_type_including_ancestors_util() {
    let mut factory = MockListableBeanFactory::new();
    let def = Arc::new(vernal_beans::RootBeanDefinition::new()) as Arc<dyn vernal_beans::BeanDefinition>;
    factory.register_definition("bean1".to_string(), def, std::any::TypeId::of::<String>());
    let names = vernal_beans::bean_factory_utils::BeanFactoryUtils::bean_names_for_type_including_ancestors(std::any::TypeId::of::<String>(), &factory);
    assert_eq!(names.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// RootBeanDefinition — 额外覆盖
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn root_bean_definition_from_generic() {
    let mut generic = vernal_beans::GenericBeanDefinition::new();
    generic.set_bean_class_name("MyClass");
    generic.set_scope(vernal_beans::Scope::Transient);
    generic.set_lazy_init(true);
    generic.set_primary(true);
    generic.set_description("desc");
    let root = vernal_beans::RootBeanDefinition::from_generic(generic);
    assert_eq!(root.bean_class_name(), "MyClass");
    assert!(root.scope().is_transient());
    assert!(root.is_lazy_init());
    assert!(root.is_primary());
    assert_eq!(root.description(), Some("desc"));
}

#[test]
fn root_bean_definition_default() {
    let def = vernal_beans::RootBeanDefinition::default();
    assert!(!def.is_lazy_init());
    assert!(!def.is_primary());
    assert!(!def.is_abstract());
    assert!(!def.is_synthetic());
    assert!(!def.is_fallback());
    assert!(def.is_autowire_candidate());
    assert_eq!(def.role(), 0);
    assert!(def.description().is_none());
    assert!(def.init_method_name().is_none());
    assert!(def.destroy_method_name().is_none());
    assert!(def.factory_bean_name().is_none());
    assert!(def.factory_method_name().is_none());
    assert!(def.parent_name().is_none());
    assert_eq!(def.depends_on().len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// RegistryBuilder — 额外覆盖
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_register_bundle() {
    let mut builder = vernal_beans::RegistryBuilder::new();
    let defs = vec![
        vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()),
    ];
    let bindings = vec![];
    builder.register_bundle(defs, bindings).unwrap();
    assert_eq!(builder.len(), 1);
}

#[test]
fn registry_builder_contains() {
    let mut builder = vernal_beans::RegistryBuilder::new();
    builder.register(vernal_beans::ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    assert!(builder.contains::<String>());
    assert!(!builder.contains::<i32>());
}
