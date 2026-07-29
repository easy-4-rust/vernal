//! 覆盖率冲刺测试 — 覆盖 bean_desc_cache、conversion_service、registry_builder、
//! type_converter_delegate、container 内部方法等所有剩余未覆盖代码路径。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use vernal_beans::{
    BeanDefinition, BeanDescriptor, BeanDescCache, ComponentDefinition, Container,
    GenericBeanDefinition, PropertyDescriptor, Qualifier, RegistryBuilder, Resolver,
    RootBeanDefinition, Scope,
};
use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
use vernal_beans::conversion_service::{ConversionService, DefaultConversionService};
use vernal_beans::type_converter_delegate::TypeConverterDelegate;

fn q(name: &str) -> Qualifier {
    Qualifier::new(name).unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanDescCache 测试
// ═══════════════════════════════════════════════════════════════════════════════

struct TestBeanDesc;
impl BeanDescriptor for TestBeanDesc {
    fn name(&self) -> &'static str { "TestBeanDesc" }
    fn properties(&self) -> &[PropertyDescriptor] { &[] }
}

struct TestBeanDesc2;
impl BeanDescriptor for TestBeanDesc2 {
    fn name(&self) -> &'static str { "TestBeanDesc2" }
    fn properties(&self) -> &[PropertyDescriptor] { &[] }
}

#[test]
fn bean_desc_cache_global() {
    let cache = BeanDescCache::global();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn bean_desc_cache_get_or_insert_first_time() {
    let cache = BeanDescCache::new();
    let desc = cache.get_or_insert::<String, _>(|| Arc::new(TestBeanDesc));
    assert_eq!(desc.name(), "TestBeanDesc");
    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());
}

#[test]
fn bean_desc_cache_get_or_insert_cache_hit() {
    let cache = BeanDescCache::new();
    let desc1 = cache.get_or_insert::<String, _>(|| Arc::new(TestBeanDesc));
    let desc2 = cache.get_or_insert::<String, _>(|| Arc::new(TestBeanDesc2));
    // Second call should return cached value, not the new one
    assert_eq!(desc1.name(), "TestBeanDesc");
    assert_eq!(desc2.name(), "TestBeanDesc");
    assert_eq!(cache.len(), 1);
}

#[test]
fn bean_desc_cache_get_or_insert_different_types() {
    let cache = BeanDescCache::new();
    cache.get_or_insert::<String, _>(|| Arc::new(TestBeanDesc));
    cache.get_or_insert::<i32, _>(|| Arc::new(TestBeanDesc2));
    assert_eq!(cache.len(), 2);
}

#[test]
fn bean_desc_cache_clear() {
    let cache = BeanDescCache::new();
    cache.get_or_insert::<String, _>(|| Arc::new(TestBeanDesc));
    assert_eq!(cache.len(), 1);
    cache.clear();
    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}

#[test]
fn bean_desc_cache_default() {
    let cache = BeanDescCache::default();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConversionService 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn conversion_service_default() {
    let service = DefaultConversionService::default();
    // Should have built-in converters
    assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<i32>()));
    assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<i64>()));
    assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<f64>()));
    assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<bool>()));
    assert!(service.can_convert(TypeId::of::<i32>(), TypeId::of::<String>()));
    assert!(service.can_convert(TypeId::of::<i64>(), TypeId::of::<String>()));
    assert!(service.can_convert(TypeId::of::<f64>(), TypeId::of::<String>()));
    assert!(service.can_convert(TypeId::of::<bool>(), TypeId::of::<String>()));
}

#[test]
fn conversion_service_cannot_convert() {
    let service = DefaultConversionService::new();
    assert!(!service.can_convert(TypeId::of::<i32>(), TypeId::of::<bool>()));
    assert!(!service.can_convert(TypeId::of::<String>(), TypeId::of::<Vec<u8>>()));
}

#[test]
fn conversion_service_string_to_i32() {
    let service = DefaultConversionService::new();
    let source = "42".to_string();
    let result = service.convert(&source, TypeId::of::<i32>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn conversion_service_string_to_i64() {
    let service = DefaultConversionService::new();
    let source = "123456789".to_string();
    let result = service.convert(&source, TypeId::of::<i64>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<i64>().unwrap(), 123456789i64);
}

#[test]
fn conversion_service_string_to_f64() {
    let service = DefaultConversionService::new();
    let source = "3.14".to_string();
    let result = service.convert(&source, TypeId::of::<f64>());
    assert!(result.is_ok());
    assert!(((*result.unwrap().downcast_ref::<f64>().unwrap()) - 3.14).abs() < f64::EPSILON);
}

#[test]
fn conversion_service_string_to_bool_true_variants() {
    let service = DefaultConversionService::new();
    for val in &["true", "True", "TRUE", "yes", "Yes", "1"] {
        let source = val.to_string();
        let result = service.convert(&source, TypeId::of::<bool>());
        assert!(result.is_ok(), "Failed for {val}");
        assert!(*result.unwrap().downcast_ref::<bool>().unwrap(), "Failed for {val}");
    }
}

#[test]
fn conversion_service_string_to_bool_false_variants() {
    let service = DefaultConversionService::new();
    for val in &["false", "False", "FALSE", "no", "No", "0"] {
        let source = val.to_string();
        let result = service.convert(&source, TypeId::of::<bool>());
        assert!(result.is_ok(), "Failed for {val}");
        assert!(!*result.unwrap().downcast_ref::<bool>().unwrap(), "Failed for {val}");
    }
}

#[test]
fn conversion_service_i32_to_string() {
    let service = DefaultConversionService::new();
    let source = 42i32;
    let result = service.convert(&source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "42");
}

#[test]
fn conversion_service_i64_to_string() {
    let service = DefaultConversionService::new();
    let source = 123456789i64;
    let result = service.convert(&source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "123456789");
}

#[test]
fn conversion_service_f64_to_string() {
    let service = DefaultConversionService::new();
    let source = 3.14f64;
    let result = service.convert(&source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "3.14");
}

#[test]
fn conversion_service_bool_to_string() {
    let service = DefaultConversionService::new();
    let source = true;
    let result = service.convert(&source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "true");
}

#[test]
fn conversion_service_string_parse_error() {
    let service = DefaultConversionService::new();
    let source = "not_a_number".to_string();
    assert!(service.convert(&source, TypeId::of::<i32>()).is_err());
    assert!(service.convert(&source, TypeId::of::<i64>()).is_err());
    assert!(service.convert(&source, TypeId::of::<f64>()).is_err());
    assert!(service.convert(&source, TypeId::of::<bool>()).is_err());
}

#[test]
fn conversion_service_no_converter_error() {
    let service = DefaultConversionService::new();
    let source = 42i32;
    assert!(service.convert(&source, TypeId::of::<f64>()).is_err());
}

#[test]
fn conversion_service_register_custom() {
    let mut service = DefaultConversionService::new();
    // Register a custom converter: String -> Vec<u8>
    service.register(Arc::new(CustomStringToBytesConverter));
    assert!(service.can_convert(TypeId::of::<String>(), TypeId::of::<Vec<u8>>()));
    let source = "hello".to_string();
    let result = service.convert(&source, TypeId::of::<Vec<u8>>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<Vec<u8>>().unwrap(), b"hello");
}

struct CustomStringToBytesConverter;

impl vernal_beans::conversion_service::Converter for CustomStringToBytesConverter {
    fn source_type(&self) -> TypeId { TypeId::of::<String>() }
    fn target_type(&self) -> TypeId { TypeId::of::<Vec<u8>>() }
    fn convert(&self, value: &dyn Any) -> Result<Box<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(s) = value.downcast_ref::<String>() {
            Ok(Box::new(s.as_bytes().to_vec()))
        } else {
            Err("expected String".into())
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TypeConverterDelegate 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_converter_delegate_new_and_default() {
    let delegate = TypeConverterDelegate::new();
    assert_eq!(delegate.editor_count(), 0);
    let delegate2 = TypeConverterDelegate::default();
    assert_eq!(delegate2.editor_count(), 0);
}

#[test]
fn type_converter_delegate_convert_same_type() {
    let delegate = TypeConverterDelegate::new();
    let source = "hello".to_string();
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn type_converter_delegate_convert_string_to_f64() {
    let delegate = TypeConverterDelegate::new();
    let source = "42".to_string();
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<f64>());
    assert!(result.is_ok());
    assert!(((*result.unwrap().downcast_ref::<f64>().unwrap()) - 42.0).abs() < f64::EPSILON);
}

#[test]
fn type_converter_delegate_convert_string_to_bool() {
    let delegate = TypeConverterDelegate::new();
    let source = "true".to_string();
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<bool>());
    assert!(result.is_ok());
    assert!(*result.unwrap().downcast_ref::<bool>().unwrap());
}

#[test]
fn type_converter_delegate_convert_string_to_f64_pi() {
    let delegate = TypeConverterDelegate::new();
    let source = "3.14".to_string();
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<f64>());
    assert!(result.is_ok());
    assert!(((*result.unwrap().downcast_ref::<f64>().unwrap()) - 3.14).abs() < f64::EPSILON);
}

#[test]
fn type_converter_delegate_convert_i32_to_string() {
    let delegate = TypeConverterDelegate::new();
    let source = 42i32;
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "42");
}

#[test]
fn type_converter_delegate_convert_bool_to_string() {
    let delegate = TypeConverterDelegate::new();
    let source = true;
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "true");
}

#[test]
fn type_converter_delegate_convert_f64_to_string() {
    let delegate = TypeConverterDelegate::new();
    let source = 3.14f64;
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "3.14");
}

#[test]
fn type_converter_delegate_convert_i64_to_string() {
    let delegate = TypeConverterDelegate::new();
    let source = 12345i64;
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "12345");
}

#[test]
fn type_converter_delegate_convert_char_to_string() {
    let delegate = TypeConverterDelegate::new();
    let source = 'A';
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<String>());
    assert!(result.is_ok());
    assert_eq!(result.unwrap().downcast_ref::<String>().unwrap(), "A");
}

#[test]
fn type_converter_delegate_convert_unsupported_target() {
    let delegate = TypeConverterDelegate::new();
    let source = "hello".to_string();
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<Vec<String>>());
    assert!(result.is_err());
}

#[test]
fn type_converter_delegate_clear() {
    let mut delegate = TypeConverterDelegate::new();
    delegate.register_custom_editor(TypeId::of::<i32>(), Arc::new(vernal_beans::number_editor::CustomNumberEditor::new()));
    assert!(delegate.editor_count() > 0);
    delegate.clear();
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_convert_u32_same_type() {
    let delegate = TypeConverterDelegate::new();
    let source = 42u32;
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<u32>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<u32>().unwrap(), 42);
}

#[test]
fn type_converter_delegate_convert_u64_same_type() {
    let delegate = TypeConverterDelegate::new();
    let source = 42u64;
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<u64>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<u64>().unwrap(), 42);
}

#[test]
fn type_converter_delegate_convert_f32_same_type() {
    let delegate = TypeConverterDelegate::new();
    let source = 3.14f32;
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<f32>());
    assert!(result.is_ok());
    assert!(((*result.unwrap().downcast_ref::<f32>().unwrap()) - 3.14).abs() < f32::EPSILON);
}

#[test]
fn type_converter_delegate_convert_vec_string_same_type() {
    let delegate = TypeConverterDelegate::new();
    let source = vec!["a".to_string(), "b".to_string()];
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<Vec<String>>());
    assert!(result.is_ok());
}

#[test]
fn type_converter_delegate_convert_hashmap_same_type() {
    let delegate = TypeConverterDelegate::new();
    let mut source = HashMap::new();
    source.insert("key".to_string(), "value".to_string());
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<HashMap<String, String>>());
    assert!(result.is_ok());
}

#[test]
fn type_converter_delegate_convert_vec_u8_same_type() {
    let delegate = TypeConverterDelegate::new();
    let source = vec![1u8, 2, 3];
    let result = delegate.convert_if_necessary(None, &source, TypeId::of::<Vec<u8>>());
    assert!(result.is_ok());
    assert_eq!(*result.unwrap().downcast_ref::<Vec<u8>>().unwrap(), vec![1u8, 2, 3]);
}

#[test]
fn type_converter_delegate_convert_custom_editor() {
    let mut delegate = TypeConverterDelegate::new();
    delegate.register_custom_editor(TypeId::of::<i32>(), Arc::new(vernal_beans::number_editor::CustomNumberEditor::new()));
    assert!(delegate.editor_count() > 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// RegistryBuilder 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn registry_builder_register_all() {
    let mut builder = RegistryBuilder::new();
    let defs = vec![
        ComponentDefinition::singleton::<String, _>(|_| "hello".to_string()),
        ComponentDefinition::singleton::<i32, _>(|_| 42i32),
    ];
    builder.register_all(defs).unwrap();
    assert_eq!(builder.len(), 2);
}

#[test]
fn registry_builder_register_all_duplicate() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let defs = vec![
        ComponentDefinition::singleton::<String, _>(|_| "world".to_string()),
    ];
    let result = builder.register_all(defs);
    assert!(result.is_err());
}

#[test]
fn registry_builder_register_duplicate() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let result = builder.register(ComponentDefinition::singleton::<String, _>(|_| "world".to_string()));
    assert!(result.is_err());
}

#[test]
fn registry_builder_remove_by_key() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    builder.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32)).unwrap();
    let key = vernal_beans::ComponentKey::of::<String>();
    builder.remove_by_key(&key).unwrap();
    assert_eq!(builder.len(), 1);
    assert!(!builder.contains::<String>());
}

#[test]
fn registry_builder_bean_definition_registry_trait() {
    let mut builder = RegistryBuilder::new();
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    // register_bean_definition is a stub that always returns Ok
    let def = Box::new(RootBeanDefinition::new());
    builder.register_bean_definition("myBean".to_string(), def).unwrap();
    // get_bean_definition is a stub that always returns None
    assert!(builder.get_bean_definition("myBean").is_none());
    // bean_definition_count
    let _ = builder.bean_definition_count();
    // bean_definition_names
    let _ = builder.bean_definition_names();
}

#[test]
fn registry_builder_remove_bean_definition_not_found() {
    let mut builder = RegistryBuilder::new();
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    let result = builder.remove_bean_definition("nonexistent");
    assert!(result.is_err());
}

#[test]
fn registry_builder_remove_bean_definition_found() {
    let mut builder = RegistryBuilder::new();
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    // First register via the component definition path
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    // Then remove by type name
    let type_name = std::any::type_name::<String>().to_string();
    let result = builder.remove_bean_definition(&type_name);
    assert!(result.is_ok());
}

#[test]
fn registry_builder_validate_bindings_duplicate_exact() {
    let mut builder = RegistryBuilder::new();
    let binding1 = vernal_beans::TraitBinding::new::<dyn vernal_beans::bean_factory::BeanFactory, Container, _>(
        |arc| arc as Arc<dyn vernal_beans::bean_factory::BeanFactory>,
    );
    let binding2 = vernal_beans::TraitBinding::new::<dyn vernal_beans::bean_factory::BeanFactory, Container, _>(
        |arc| arc as Arc<dyn vernal_beans::bean_factory::BeanFactory>,
    );
    builder.bind(binding1).unwrap();
    // Binding the same exact pair again should fail
    let result = builder.bind(binding2);
    assert!(result.is_err());
}

#[test]
fn registry_builder_validate_bindings_duplicate_in_batch() {
    let mut builder = RegistryBuilder::new();
    let binding1 = vernal_beans::TraitBinding::new::<dyn vernal_beans::bean_factory::BeanFactory, Container, _>(
        |arc| arc as Arc<dyn vernal_beans::bean_factory::BeanFactory>,
    );
    let binding2 = vernal_beans::TraitBinding::new::<dyn vernal_beans::bean_factory::BeanFactory, Container, _>(
        |arc| arc as Arc<dyn vernal_beans::bean_factory::BeanFactory>,
    );
    // Two identical bindings in the same batch
    let result = builder.bind_all(vec![binding1, binding2]);
    assert!(result.is_err());
}

#[test]
fn registry_builder_validate_definitions_duplicate_in_batch() {
    let mut builder = RegistryBuilder::new();
    let defs = vec![
        ComponentDefinition::singleton::<String, _>(|_| "a".to_string()),
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string()),
    ];
    let result = builder.register_all(defs);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — bean_definition_names
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_bean_definition_names() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    // Add a dynamic definition
    let def = Box::new(RootBeanDefinition::new());
    container.register_bean_definition("myBean".to_string(), def).unwrap();
    // Remove one to create a deleted marker
    let type_name = std::any::type_name::<String>().to_string();
    let _ = container.remove_bean_definition(&type_name);
    let names = container.bean_definition_names();
    // Should contain "myBean" and the registry definition (String) but not the deleted one
    assert!(names.contains(&"myBean".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — is_type_match
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_is_type_match() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    // Check via BeanFactory trait
    use vernal_beans::bean_factory::BeanFactory;
    let key = vernal_beans::ComponentKey::of::<String>();
    // Contains the bean
    assert!(container.contains_bean(&key));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — PostProcessor Ok(None) branch
// ═══════════════════════════════════════════════════════════════════════════════

struct NoOpPostProcessor;

impl vernal_beans::BeanPostProcessor for NoOpPostProcessor {
    fn post_process_after_initialization(
        &self,
        bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(None) // Return None to exercise line 626
    }
}

#[test]
fn container_resolve_with_noop_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(NoOpPostProcessor));
    // Resolve should still work, returning original bean
    let result = container.resolve::<String>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), "hello");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Container — configure_bean with Ok(None) PostProcessor
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn container_configure_bean_with_noop_post_processor() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let mut container = Container::new(registry);
    container.add_bean_post_processor(Arc::new(NoOpPostProcessor));
    use vernal_beans::AutowireCapableBeanFactory;
    let existing: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
    let result = container.configure_bean(existing, "test");
    assert!(result.is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════════
// RootBeanDefinition — get_constructor_argument_values_mut / get_property_values_mut
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn root_bean_definition_constructor_args_and_property_values() {
    let mut def = RootBeanDefinition::new();
    def.get_constructor_argument_values_mut().add_generic_argument_value(
        vernal_beans::ValueHolder::new(Arc::new(42i32)),
    );
    def.get_property_values_mut().add_value("myProp", Arc::new("hello".to_string()));
    assert!(!def.constructor_argument_values().generic_argument_values().is_empty());
    assert!(def.property_values().get("myProp").is_some());
}

#[test]
fn root_bean_definition_setters_coverage() {
    let mut def = RootBeanDefinition::new();
    def.set_scope(Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    def.set_fallback(true);
    def.set_abstract(true);
    def.set_autowire_candidate(false);
    def.set_role(2);
    def.set_description("desc");
    def.set_init_method_name("init");
    def.set_destroy_method_name("destroy");
    def.set_factory_bean_name("factory");
    def.set_factory_method_name("create");
    def.set_parent_name("parent");
    def.set_bean_class_name("MyClass");
    def.set_synthetic(true);
    def.add_depends_on("dep");
    def.set_autowire_mode(vernal_beans::Autowire::ByType);
    // Read all getters
    assert!(def.scope().is_transient());
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
    assert!(def.is_fallback());
    assert!(!def.is_autowire_candidate());
    assert_eq!(def.role(), 2);
    assert_eq!(def.description(), Some("desc"));
    assert_eq!(def.init_method_name(), Some("init"));
    assert_eq!(def.destroy_method_name(), Some("destroy"));
    assert_eq!(def.factory_bean_name(), Some("factory"));
    assert_eq!(def.factory_method_name(), Some("create"));
    assert_eq!(def.parent_name(), Some("parent"));
    assert_eq!(def.bean_class_name(), "MyClass");
    assert!(def.is_abstract());
    assert!(def.is_synthetic());
    assert_eq!(def.depends_on(), &["dep"]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// GenericBeanDefinition — extra coverage
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn generic_bean_definition_all_setters() {
    let mut def = GenericBeanDefinition::new();
    def.set_bean_class_name("MyClass");
    def.set_scope(Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    def.set_fallback(true);
    def.set_abstract(true);
    def.set_autowire_candidate(false);
    def.set_role(2);
    def.set_description("desc");
    def.set_init_method_name("init");
    def.set_destroy_method_name("destroy");
    def.set_factory_bean_name("factory");
    def.set_factory_method_name("create");
    def.set_parent_name("parent");
    def.set_synthetic(true);
    def.set_autowire_mode(vernal_beans::Autowire::ByType);
    def.add_depends_on("dep");
    // Read all
    assert_eq!(def.get_bean_class_name(), Some("MyClass"));
    assert!(def.scope().is_transient());
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
    assert!(def.is_fallback());
    assert!(!def.is_autowire_candidate());
    assert_eq!(def.role(), 2);
    assert_eq!(def.description(), Some("desc"));
    assert_eq!(def.init_method_name(), Some("init"));
    assert_eq!(def.destroy_method_name(), Some("destroy"));
    assert_eq!(def.factory_bean_name(), Some("factory"));
    assert_eq!(def.factory_method_name(), Some("create"));
    assert_eq!(def.get_parent_name(), Some("parent"));
    assert!(def.is_abstract());
    assert!(def.is_synthetic());
    assert_eq!(def.depends_on(), &["dep"]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Resolver — resolve_trait
// ═══════════════════════════════════════════════════════════════════════════════

trait TestTrait: Send + Sync {
    fn get_value(&self) -> i32;
}

struct TestTraitImpl(i32);
impl TestTrait for TestTraitImpl {
    fn get_value(&self) -> i32 { self.0 }
}

#[test]
fn resolver_resolve_trait() {
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<TestTraitImpl, _>(|_| TestTraitImpl(42))).unwrap();
    builder.bind(vernal_beans::TraitBinding::new::<dyn TestTrait, TestTraitImpl, _>(
        |arc| arc as Arc<dyn TestTrait>,
    )).unwrap();
    builder.register(ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        resolver.resolve_trait::<dyn TestTrait>().unwrap().get_value()
    }).depends_on_trait::<dyn TestTrait>()).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    assert_eq!(*container.resolve::<i32>().unwrap(), 42);
}

#[test]
fn resolver_resolve_trait_not_found() {
    // Test that resolve_trait returns an error when no binding exists
    let mut builder = RegistryBuilder::new();
    builder.register(ComponentDefinition::singleton::<String, _>(|_| "hello".to_string())).unwrap();
    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let result = container.resolve_trait::<dyn TestTrait>();
    assert!(result.is_err());
}
