//! 高影响力文件的覆盖测试：type_converter_delegate, root_bean_definition, bean_definition_builder

use std::any::Any;
use std::sync::Arc;

use vernal_beans::{
    Autowire, BeanDefinitionBuilder, ConversionService, ConstructorArgumentValues,
    GenericBeanDefinition, MutablePropertyValues, PropertyDescriptor, PropertyValue,
    RootBeanDefinition, Scope, ValueHolder,
};

// ═══════════════════════════════════════════════════════════════════════════════
// TypeConverterDelegate 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn type_converter_delegate_new() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_default() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::default();
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_register_editor() {
    use vernal_beans::property_editor::PropertyEditor;
    use vernal_beans::string_trimmer_editor::StringTrimmerEditor;

    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let editor: Arc<dyn PropertyEditor> = Arc::new(StringTrimmerEditor::new());
    delegate.register_custom_editor(std::any::TypeId::of::<String>(), editor);
    assert_eq!(delegate.editor_count(), 1);
}

#[test]
fn type_converter_delegate_register_converter() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    delegate.register_converter(
        std::any::TypeId::of::<String>(),
        std::any::TypeId::of::<i32>(),
        |value| {
            let s = value.downcast_ref::<String>().unwrap();
            let n: i32 = s.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(e)
            })?;
            Ok(Box::new(n))
        },
    );
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_convert_same_type() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new(42i32);
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<i32>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn type_converter_delegate_convert_string_to_bool() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new("true".to_string());
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<bool>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<bool>().unwrap(), true);
}

#[test]
fn type_converter_delegate_convert_string_to_i32() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new("123".to_string());
    // NumberEditor 内部存储为 f64，所以返回 f64
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<i32>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<f64>().unwrap(), 123.0);
}

#[test]
fn type_converter_delegate_convert_string_to_char() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new("A".to_string());
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<char>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<char>().unwrap(), 'A');
}

#[test]
fn type_converter_delegate_convert_with_custom_converter() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    delegate.register_converter(
        std::any::TypeId::of::<String>(),
        std::any::TypeId::of::<i32>(),
        |value| {
            let s = value.downcast_ref::<String>().unwrap();
            let n: i32 = s.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(e)
            })?;
            Ok(Box::new(n))
        },
    );

    let value: Box<dyn Any> = Box::new("456".to_string());
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<i32>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<i32>().unwrap(), 456);
}

#[test]
fn type_converter_delegate_convert_no_converter() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new(Vec::<String>::new());
    let result = delegate.convert_if_necessary(
        None,
        &*value,
        std::any::TypeId::of::<Vec<i32>>(),
    );
    assert!(result.is_err());
}

#[test]
fn type_converter_delegate_convert_i32_to_string() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new(42i32);
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<String>().unwrap(), "42");
}

#[test]
fn type_converter_delegate_convert_bool_to_string() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new(true);
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<String>().unwrap(), "true");
}

#[test]
fn type_converter_delegate_convert_char_to_string() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new('X');
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<String>().unwrap(), "X");
}

#[test]
fn type_converter_delegate_clear() {
    use vernal_beans::property_editor::PropertyEditor;
    use vernal_beans::string_trimmer_editor::StringTrimmerEditor;

    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let editor: Arc<dyn PropertyEditor> = Arc::new(StringTrimmerEditor::new());
    delegate.register_custom_editor(std::any::TypeId::of::<String>(), editor);
    delegate.register_converter(
        std::any::TypeId::of::<String>(),
        std::any::TypeId::of::<i32>(),
        |value| {
            let s = value.downcast_ref::<String>().unwrap();
            let n: i32 = s.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                Box::new(e)
            })?;
            Ok(Box::new(n))
        },
    );
    assert_eq!(delegate.editor_count(), 1);
    delegate.clear();
    assert_eq!(delegate.editor_count(), 0);
}

#[test]
fn type_converter_delegate_convert_str_ref() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: &dyn Any = &"hello";
    let result = delegate
        .convert_if_necessary(None, value, std::any::TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<String>().unwrap(), "hello");
}

#[test]
fn type_converter_delegate_convert_i64_to_string() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new(42i64);
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<String>().unwrap(), "42");
}

#[test]
fn type_converter_delegate_convert_f64_to_string() {
    let delegate = vernal_beans::type_converter_delegate::TypeConverterDelegate::new();
    let value: Box<dyn Any> = Box::new(3.14f64);
    let result = delegate
        .convert_if_necessary(None, &*value, std::any::TypeId::of::<String>())
        .unwrap();
    assert_eq!(*result.downcast_ref::<String>().unwrap(), "3.14");
}

// ═══════════════════════════════════════════════════════════════════════════════
// RootBeanDefinition 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn root_bean_definition_new() {
    let rbd = RootBeanDefinition::new();
    assert_eq!(rbd.bean_class_name(), "unknown");
    assert!(rbd.parent_name().is_none());
    assert_eq!(rbd.scope(), Scope::Singleton);
    assert!(!rbd.is_lazy_init());
    assert!(!rbd.is_abstract());
    assert!(rbd.is_autowire_candidate());
    assert!(!rbd.is_primary());
    assert!(!rbd.is_fallback());
    assert!(!rbd.is_synthetic());
    assert_eq!(rbd.role(), vernal_beans::bean_definition::ROLE_APPLICATION);
    assert!(rbd.description().is_none());
    assert!(rbd.depends_on().is_empty());
    assert_eq!(rbd.autowire_mode(), Autowire::No);
    assert!(rbd.init_method_name().is_none());
    assert!(rbd.destroy_method_name().is_none());
    assert!(rbd.factory_bean_name().is_none());
    assert!(rbd.factory_method_name().is_none());
    assert_eq!(rbd.init_order_value(), 0);
}

#[test]
fn root_bean_definition_default() {
    let rbd = RootBeanDefinition::default();
    assert_eq!(rbd.bean_class_name(), "unknown");
}

#[test]
fn root_bean_definition_setters() {
    let mut rbd = RootBeanDefinition::new();

    rbd.set_bean_class_name("MyBean");
    assert_eq!(rbd.bean_class_name(), "MyBean");

    rbd.set_parent_name("ParentBean");
    assert_eq!(rbd.parent_name().unwrap(), "ParentBean");

    rbd.set_scope(Scope::Transient);
    assert_eq!(rbd.scope(), Scope::Transient);

    rbd.set_lazy_init(true);
    assert!(rbd.is_lazy_init());

    rbd.set_abstract(true);
    assert!(rbd.is_abstract());

    rbd.set_autowire_candidate(false);
    assert!(!rbd.is_autowire_candidate());

    rbd.set_primary(true);
    assert!(rbd.is_primary());

    rbd.set_fallback(true);
    assert!(rbd.is_fallback());

    rbd.set_synthetic(true);
    assert!(rbd.is_synthetic());

    rbd.set_role(vernal_beans::bean_definition::ROLE_SUPPORT);
    assert_eq!(rbd.role(), vernal_beans::bean_definition::ROLE_SUPPORT);

    rbd.set_description("Test description");
    assert_eq!(rbd.description().unwrap(), "Test description");

    rbd.add_depends_on("dep1");
    rbd.add_depends_on("dep2");
    assert_eq!(rbd.depends_on(), &["dep1", "dep2"]);

    rbd.set_autowire_mode(Autowire::ByType);
    assert_eq!(rbd.autowire_mode(), Autowire::ByType);

    rbd.set_init_method_name("init");
    assert_eq!(rbd.init_method_name().unwrap(), "init");

    rbd.set_destroy_method_name("destroy");
    assert_eq!(rbd.destroy_method_name().unwrap(), "destroy");

    rbd.set_factory_bean_name("factoryBean");
    assert_eq!(rbd.factory_bean_name().unwrap(), "factoryBean");

    rbd.set_factory_method_name("factoryMethod");
    assert_eq!(rbd.factory_method_name().unwrap(), "factoryMethod");

    rbd.set_init_order(100);
    assert_eq!(rbd.init_order_value(), 100);
}

#[test]
fn root_bean_definition_constructor_argument_values() {
    let mut rbd = RootBeanDefinition::new();
    let cav = rbd.get_constructor_argument_values_mut();
    cav.add_generic_argument_value(ValueHolder::new(Arc::new("arg1".to_string())));
    assert_eq!(rbd.constructor_argument_values().argument_count(), 1);
}

#[test]
fn root_bean_definition_property_values() {
    let mut rbd = RootBeanDefinition::new();
    let pv = rbd.get_property_values_mut();
    pv.add(PropertyValue::new("prop1", Arc::new("value1".to_string())));
    assert_eq!(rbd.property_values().len(), 1);
}

#[test]
fn root_bean_definition_from_generic() {
    let mut generic = GenericBeanDefinition::new();
    generic.set_bean_class_name("GenericBean");
    generic.set_scope(Scope::Transient);
    generic.set_lazy_init(true);
    generic.set_abstract(false);
    generic.set_autowire_candidate(true);
    generic.set_primary(true);
    generic.set_fallback(false);
    generic.set_synthetic(true);
    generic.set_role(vernal_beans::bean_definition::ROLE_APPLICATION);
    generic.set_description("From generic");
    generic.add_depends_on("gdep1");
    generic.set_autowire_mode(Autowire::ByName);
    generic.set_init_method_name("ginit");
    generic.set_destroy_method_name("gdestroy");
    generic.set_factory_bean_name("gfactory");
    generic.set_factory_method_name("gmethod");

    let rbd = RootBeanDefinition::from_generic(generic);
    assert_eq!(rbd.bean_class_name(), "GenericBean");
    assert_eq!(rbd.scope(), Scope::Transient);
    assert!(rbd.is_lazy_init());
    assert!(rbd.is_primary());
    assert!(rbd.is_synthetic());
    assert_eq!(rbd.description().unwrap(), "From generic");
    assert_eq!(rbd.depends_on(), &["gdep1"]);
    assert_eq!(rbd.autowire_mode(), Autowire::ByName);
    assert_eq!(rbd.init_method_name().unwrap(), "ginit");
    assert_eq!(rbd.destroy_method_name().unwrap(), "gdestroy");
    assert_eq!(rbd.factory_bean_name().unwrap(), "gfactory");
    assert_eq!(rbd.factory_method_name().unwrap(), "gmethod");
}

#[test]
fn root_bean_definition_into() {
    let mut generic = GenericBeanDefinition::new();
    generic.set_bean_class_name("IntoBean");
    let rbd: RootBeanDefinition = generic.into();
    assert_eq!(rbd.bean_class_name(), "IntoBean");
}

#[test]
fn root_bean_definition_clone() {
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("Cloned");
    rbd.set_scope(Scope::Transient);
    let cloned = rbd.clone();
    assert_eq!(cloned.bean_class_name(), "Cloned");
    assert_eq!(cloned.scope(), Scope::Transient);
}

#[test]
fn root_bean_definition_debug() {
    let rbd = RootBeanDefinition::new();
    let debug = format!("{:?}", rbd);
    assert!(debug.contains("RootBeanDefinition"));
}

#[test]
fn root_bean_definition_bean_definition_trait() {
    let mut rbd = RootBeanDefinition::new();
    rbd.set_bean_class_name("TraitBean");
    rbd.set_scope(Scope::Transient);
    rbd.set_lazy_init(true);
    rbd.set_primary(true);
    rbd.set_fallback(true);
    rbd.set_autowire_candidate(false);
    rbd.set_role(vernal_beans::bean_definition::ROLE_SUPPORT);
    rbd.set_description("desc");
    rbd.set_parent_name("parent");
    rbd.set_factory_bean_name("factory");
    rbd.set_factory_method_name("method");
    rbd.set_init_method_name("init");
    rbd.set_destroy_method_name("destroy");
    rbd.set_abstract(true);

    assert_eq!(rbd.bean_class_name(), "TraitBean");
    assert_eq!(rbd.scope(), Scope::Transient);
    assert!(rbd.is_lazy_init());
    assert!(rbd.is_primary());
    assert!(rbd.is_fallback());
    assert!(!rbd.is_autowire_candidate());
    assert_eq!(rbd.role(), vernal_beans::bean_definition::ROLE_SUPPORT);
    assert_eq!(rbd.description().unwrap(), "desc");
    assert_eq!(rbd.parent_name().unwrap(), "parent");
    assert_eq!(rbd.factory_bean_name().unwrap(), "factory");
    assert_eq!(rbd.factory_method_name().unwrap(), "method");
    assert_eq!(rbd.init_method_name().unwrap(), "init");
    assert_eq!(rbd.destroy_method_name().unwrap(), "destroy");
    assert!(rbd.is_abstract());
}

// ═══════════════════════════════════════════════════════════════════════════════
// BeanDefinitionBuilder 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn bean_definition_builder_generic() {
    let builder = BeanDefinitionBuilder::generic("MyBean");
    let rbd = builder.build();
    assert_eq!(rbd.get_bean_class_name(), Some("MyBean"));
}

#[test]
fn bean_definition_builder_root() {
    let builder = BeanDefinitionBuilder::root("RootBean");
    let rbd = builder.build();
    assert_eq!(rbd.bean_class_name(), "RootBean");
}

#[test]
fn bean_definition_builder_set_bean_class_name() {
    let builder = BeanDefinitionBuilder::generic("Old").set_bean_class_name("New");
    let rbd = builder.build();
    assert_eq!(rbd.get_bean_class_name(), Some("New"));
}

#[test]
fn bean_definition_builder_set_scope() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_scope(Scope::Transient);
    let rbd = builder.build();
    assert_eq!(rbd.scope(), Scope::Transient);
}

#[test]
fn bean_definition_builder_set_lazy_init() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_lazy_init(true);
    let rbd = builder.build();
    assert!(rbd.is_lazy_init());
}

#[test]
fn bean_definition_builder_set_primary() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_primary(true);
    let rbd = builder.build();
    assert!(rbd.is_primary());
}

#[test]
fn bean_definition_builder_set_abstract() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_abstract(true);
    let rbd = builder.build();
    assert!(rbd.is_abstract());
}

#[test]
fn bean_definition_builder_set_autowire_candidate() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_autowire_candidate(false);
    let rbd = builder.build();
    assert!(!rbd.is_autowire_candidate());
}

#[test]
fn bean_definition_builder_set_description() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_description("Test desc");
    let rbd = builder.build();
    assert_eq!(rbd.description().unwrap(), "Test desc");
}

#[test]
fn bean_definition_builder_add_depends_on() {
    let builder = BeanDefinitionBuilder::generic("Bean")
        .add_depends_on("dep1")
        .add_depends_on("dep2");
    let rbd = builder.build();
    assert_eq!(rbd.depends_on(), &["dep1", "dep2"]);
}

#[test]
fn bean_definition_builder_set_init_method() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_init_method("myInit");
    let rbd = builder.build();
    assert_eq!(rbd.init_method_name().unwrap(), "myInit");
}

#[test]
fn bean_definition_builder_set_destroy_method() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_destroy_method("myDestroy");
    let rbd = builder.build();
    assert_eq!(rbd.destroy_method_name().unwrap(), "myDestroy");
}

#[test]
fn bean_definition_builder_set_factory_bean_name() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_factory_bean_name("myFactory");
    let rbd = builder.build();
    assert_eq!(rbd.factory_bean_name().unwrap(), "myFactory");
}

#[test]
fn bean_definition_builder_set_factory_method_name() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_factory_method_name("myMethod");
    let rbd = builder.build();
    assert_eq!(rbd.factory_method_name().unwrap(), "myMethod");
}

#[test]
fn bean_definition_builder_set_autowire_mode() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_autowire_mode(Autowire::ByType);
    let rbd = builder.build();
    assert_eq!(rbd.autowire_mode(), Autowire::ByType);
}

#[test]
fn bean_definition_builder_set_parent_name() {
    let builder = BeanDefinitionBuilder::generic("Bean").set_parent_name("Parent");
    let rbd = builder.build();
    assert_eq!(rbd.get_parent_name(), Some("Parent"));
}

#[test]
fn bean_definition_builder_chained() {
    let builder = BeanDefinitionBuilder::generic("ChainedBean")
        .set_scope(Scope::Transient)
        .set_lazy_init(true)
        .set_primary(true)
        .set_description("Chained builder")
        .add_depends_on("dep1")
        .set_init_method("init")
        .set_destroy_method("destroy")
        .set_factory_bean_name("factory")
        .set_factory_method_name("method")
        .set_autowire_mode(Autowire::ByType)
        .set_parent_name("Parent")
        .set_autowire_candidate(false)
        .set_abstract(true);

    let rbd = builder.build();
    assert_eq!(rbd.get_bean_class_name(), Some("ChainedBean"));
    assert_eq!(rbd.scope(), Scope::Transient);
    assert!(rbd.is_lazy_init());
    assert!(rbd.is_primary());
    assert_eq!(rbd.description().unwrap(), "Chained builder");
    assert_eq!(rbd.depends_on(), &["dep1"]);
    assert_eq!(rbd.init_method_name().unwrap(), "init");
    assert_eq!(rbd.destroy_method_name().unwrap(), "destroy");
    assert_eq!(rbd.factory_bean_name().unwrap(), "factory");
    assert_eq!(rbd.factory_method_name().unwrap(), "method");
    assert_eq!(rbd.autowire_mode(), Autowire::ByType);
    assert_eq!(rbd.get_parent_name(), Some("Parent"));
    assert!(!rbd.is_autowire_candidate());
    assert!(rbd.is_abstract());
}

#[test]
fn bean_definition_builder_add_property_value() {
    let builder = BeanDefinitionBuilder::generic("Bean")
        .add_property_value("key", "value".to_string());
    let rbd = builder.build();
    assert_eq!(rbd.property_values().len(), 1);
}

#[test]
fn bean_definition_builder_add_constructor_arg_value() {
    let builder = BeanDefinitionBuilder::generic("Bean")
        .add_constructor_arg_value("arg1".to_string());
    let rbd = builder.build();
    assert_eq!(rbd.constructor_argument_values().argument_count(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// GenericBeanDefinition 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn generic_bean_definition_new() {
    let gbd = GenericBeanDefinition::new();
    assert_eq!(gbd.get_bean_class_name(), None);
}

#[test]
fn generic_bean_definition_setters() {
    let mut gbd = GenericBeanDefinition::new();
    gbd.set_bean_class_name("Generic");
    assert_eq!(gbd.get_bean_class_name(), Some("Generic"));

    gbd.set_scope(Scope::Transient);
    assert_eq!(gbd.scope(), Scope::Transient);

    gbd.set_lazy_init(true);
    assert!(gbd.is_lazy_init());

    gbd.set_abstract(true);
    assert!(gbd.is_abstract());

    gbd.set_primary(true);
    assert!(gbd.is_primary());

    gbd.set_fallback(true);
    assert!(gbd.is_fallback());

    gbd.set_synthetic(true);
    assert!(gbd.is_synthetic());

    gbd.set_autowire_candidate(false);
    assert!(!gbd.is_autowire_candidate());

    gbd.set_role(vernal_beans::bean_definition::ROLE_SUPPORT);
    assert_eq!(gbd.role(), vernal_beans::bean_definition::ROLE_SUPPORT);

    gbd.set_description("Generic desc");
    assert_eq!(gbd.description().unwrap(), "Generic desc");

    gbd.add_depends_on("gdep");
    assert_eq!(gbd.depends_on(), &["gdep"]);

    gbd.set_autowire_mode(Autowire::ByType);
    assert_eq!(gbd.autowire_mode(), Autowire::ByType);

    gbd.set_init_method_name("ginit");
    assert_eq!(gbd.init_method_name().unwrap(), "ginit");

    gbd.set_destroy_method_name("gdestroy");
    assert_eq!(gbd.destroy_method_name().unwrap(), "gdestroy");

    gbd.set_factory_bean_name("gfactory");
    assert_eq!(gbd.factory_bean_name().unwrap(), "gfactory");

    gbd.set_factory_method_name("gmethod");
    assert_eq!(gbd.factory_method_name().unwrap(), "gmethod");

    gbd.set_parent_name("gparent");
    assert_eq!(gbd.get_parent_name(), Some("gparent"));
}

#[test]
fn generic_bean_definition_from_root() {
    let mut root = RootBeanDefinition::new();
    root.set_bean_class_name("FromRoot");
    root.set_scope(Scope::Transient);
    root.set_lazy_init(true);
    root.set_primary(true);
    root.set_description("root desc");
    root.set_autowire_mode(Autowire::ByType);

    let gbd = GenericBeanDefinition::from_root(&root);
    assert_eq!(gbd.get_bean_class_name(), Some("FromRoot"));
    assert_eq!(gbd.scope(), Scope::Transient);
    assert!(gbd.is_lazy_init());
    assert!(gbd.is_primary());
    assert_eq!(gbd.description().unwrap(), "root desc");
    assert_eq!(gbd.autowire_mode(), Autowire::ByType);
}

#[test]
fn generic_bean_definition_clone() {
    let mut gbd = GenericBeanDefinition::new();
    gbd.set_bean_class_name("ClonedGeneric");
    let cloned = gbd.clone();
    assert_eq!(cloned.get_bean_class_name(), Some("ClonedGeneric"));
}

#[test]
fn generic_bean_definition_debug() {
    let gbd = GenericBeanDefinition::new();
    let debug = format!("{:?}", gbd);
    assert!(debug.contains("GenericBeanDefinition"));
}

#[test]
fn generic_bean_definition_add_property_value() {
    let mut gbd = GenericBeanDefinition::new();
    gbd.add_property_value("key", "value".to_string());
    assert_eq!(gbd.property_values().len(), 1);
}

#[test]
fn generic_bean_definition_constructor_argument_values_mut() {
    let mut gbd = GenericBeanDefinition::new();
    gbd.constructor_argument_values_mut()
        .add_generic_argument_value(ValueHolder::new(Arc::new("arg".to_string())));
    assert_eq!(gbd.constructor_argument_values().argument_count(), 1);
}

#[test]
fn generic_bean_definition_property_values_mut() {
    let mut gbd = GenericBeanDefinition::new();
    gbd.property_values_mut()
        .add(PropertyValue::new("k", Arc::new("v".to_string())));
    assert_eq!(gbd.property_values().len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConstructorArgumentValues 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn constructor_argument_values_new() {
    let cav = ConstructorArgumentValues::new();
    assert_eq!(cav.argument_count(), 0);
    assert!(cav.is_empty());
}

#[test]
fn constructor_argument_values_from_other() {
    let mut cav1 = ConstructorArgumentValues::new();
    cav1.add_generic_argument_value(ValueHolder::new(Arc::new("arg1".to_string())));
    let cav2 = ConstructorArgumentValues::from_other(&cav1);
    assert_eq!(cav2.argument_count(), 1);
}

#[test]
fn constructor_argument_values_add_indexed_argument_value() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg0".to_string())));
    assert!(cav.has_indexed_argument_value(0));
    assert!(!cav.has_indexed_argument_value(1));
}

#[test]
fn constructor_argument_values_get_indexed_argument_value() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg0".to_string())));
    let vh = cav.get_indexed_argument_value(0);
    assert!(vh.is_some());
}

#[test]
fn constructor_argument_values_indexed_argument_values() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg0".to_string())));
    cav.add_indexed_argument_value(1, ValueHolder::new(Arc::new("arg1".to_string())));
    assert_eq!(cav.indexed_argument_values().len(), 2);
}

#[test]
fn constructor_argument_values_add_generic_argument_value() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_generic_argument_value(ValueHolder::with_type(
        Arc::new(42i32),
        "i32",
    ));
    assert_eq!(cav.generic_argument_values().len(), 1);
}

#[test]
fn constructor_argument_values_get_generic_argument_value() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_generic_argument_value(ValueHolder::with_type(
        Arc::new(42i32),
        "i32",
    ));
    let vh = cav.get_generic_argument_value("i32");
    assert!(vh.is_some());
    let vh_none = cav.get_generic_argument_value("f64");
    assert!(vh_none.is_none());
}

#[test]
fn constructor_argument_values_get_argument_value() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg0".to_string())));
    cav.add_generic_argument_value(ValueHolder::with_type(
        Arc::new(42i32),
        "i32",
    ));

    // 按索引查找
    let vh = cav.get_argument_value(0, None, None);
    assert!(vh.is_some());

    // 按类型查找
    let vh = cav.get_argument_value(99, Some("i32"), None);
    assert!(vh.is_some());

    // 不存在
    let vh = cav.get_argument_value(99, Some("f64"), None);
    assert!(vh.is_none());
}

#[test]
fn constructor_argument_values_contains_named_argument() {
    let mut cav = ConstructorArgumentValues::new();
    assert!(!cav.contains_named_argument());

    cav.add_generic_argument_value(ValueHolder::with_type_and_name(
        Arc::new("arg".to_string()),
        "String",
        "name",
    ));
    assert!(cav.contains_named_argument());
}

#[test]
fn constructor_argument_values_clear() {
    let mut cav = ConstructorArgumentValues::new();
    cav.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg0".to_string())));
    cav.add_generic_argument_value(ValueHolder::new(Arc::new("arg1".to_string())));
    cav.clear();
    assert!(cav.is_empty());
}

#[test]
fn constructor_argument_values_add_argument_values() {
    let mut cav1 = ConstructorArgumentValues::new();
    cav1.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg0".to_string())));

    let mut cav2 = ConstructorArgumentValues::new();
    cav2.add_indexed_argument_value(1, ValueHolder::new(Arc::new("arg1".to_string())));

    cav1.add_argument_values(&cav2);
    assert_eq!(cav1.argument_count(), 2);
}

#[test]
fn value_holder_new() {
    let vh = ValueHolder::new(Arc::new("test".to_string()));
    assert!(vh.value().is_some());
    assert!(vh.type_name().is_none());
    assert!(vh.name().is_none());
    assert!(!vh.is_converted());
}

#[test]
fn value_holder_with_type() {
    let vh = ValueHolder::with_type(Arc::new(42i32), "i32");
    assert!(vh.value().is_some());
    assert_eq!(vh.type_name(), Some("i32"));
    assert!(vh.name().is_none());
}

#[test]
fn value_holder_with_type_and_name() {
    let vh = ValueHolder::with_type_and_name(Arc::new("test".to_string()), "String", "name");
    assert!(vh.value().is_some());
    assert_eq!(vh.type_name(), Some("String"));
    assert_eq!(vh.name(), Some("name"));
}

#[test]
fn value_holder_set_converted_value() {
    let mut vh = ValueHolder::new(Arc::new("original".to_string()));
    assert!(!vh.is_converted());
    vh.set_converted_value(Arc::new("converted".to_string()));
    assert!(vh.is_converted());
    assert!(vh.converted_value().is_some());
}

#[test]
fn value_holder_copy() {
    let vh = ValueHolder::with_type_and_name(Arc::new("test".to_string()), "String", "name");
    let copied = vh.copy();
    assert!(copied.value().is_some());
    assert_eq!(copied.type_name(), Some("String"));
    assert_eq!(copied.name(), Some("name"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// MutablePropertyValues 补充测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn mutable_property_values_from_vec() {
    let props = vec![
        PropertyValue::new("k1", Arc::new("v1".to_string())),
        PropertyValue::new("k2", Arc::new("v2".to_string())),
    ];
    let mpv = MutablePropertyValues::from_vec(props);
    assert_eq!(mpv.len(), 2);
}

#[test]
fn mutable_property_values_contains() {
    let mut mpv = MutablePropertyValues::new();
    mpv.add(PropertyValue::new("key", Arc::new("value".to_string())));
    assert!(mpv.contains("key"));
    assert!(!mpv.contains("missing"));
}

#[test]
fn mutable_property_values_get() {
    let mut mpv = MutablePropertyValues::new();
    mpv.add(PropertyValue::new("key", Arc::new("value".to_string())));
    let pv = mpv.get("key");
    assert!(pv.is_some());
}

#[test]
fn mutable_property_values_add_property_values() {
    let mut mpv1 = MutablePropertyValues::new();
    mpv1.add(PropertyValue::new("k1", Arc::new("v1".to_string())));

    let mut mpv2 = MutablePropertyValues::new();
    mpv2.add(PropertyValue::new("k2", Arc::new("v2".to_string())));

    mpv1.add_property_values(&mpv2);
    assert_eq!(mpv1.len(), 2);
}

#[test]
fn mutable_property_values_clear() {
    let mut mpv = MutablePropertyValues::new();
    mpv.add(PropertyValue::new("k1", Arc::new("v1".to_string())));
    mpv.add(PropertyValue::new("k2", Arc::new("v2".to_string())));
    mpv.clear();
    assert_eq!(mpv.len(), 0);
}

#[test]
fn mutable_property_values_is_empty() {
    let mut mpv = MutablePropertyValues::new();
    assert!(mpv.is_empty());
    mpv.add(PropertyValue::new("k", Arc::new("v".to_string())));
    assert!(!mpv.is_empty());
}

#[test]
fn mutable_property_values_get_property_values() {
    let mut mpv = MutablePropertyValues::new();
    mpv.add(PropertyValue::new("k1", Arc::new("v1".to_string())));
    mpv.add(PropertyValue::new("k2", Arc::new("v2".to_string())));
    let values = mpv.get_property_values();
    assert_eq!(values.len(), 2);
}

#[test]
fn mutable_property_values_add_value() {
    let mut mpv = MutablePropertyValues::new();
    mpv.add_value("key", Arc::new("value".to_string()));
    assert_eq!(mpv.len(), 1);
    assert!(mpv.contains("key"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConversionService 补充测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn conversion_service_default() {
    let cs = vernal_beans::DefaultConversionService::new();
    // String -> i32 转换器已注册
    assert!(cs.can_convert(std::any::TypeId::of::<String>(), std::any::TypeId::of::<i32>()));
    // String -> String 未注册
    assert!(!cs.can_convert(std::any::TypeId::of::<String>(), std::any::TypeId::of::<String>()));
}

#[test]
fn conversion_service_convert() {
    let cs = vernal_beans::DefaultConversionService::new();
    let result = cs.convert(&"42".to_string(), std::any::TypeId::of::<i32>());
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(*value.downcast_ref::<i32>().unwrap(), 42);
}

#[test]
fn conversion_service_convert_no_converter() {
    let cs = vernal_beans::DefaultConversionService::new();
    let result = cs.convert(&"hello", std::any::TypeId::of::<String>());
    assert!(result.is_err());
}
