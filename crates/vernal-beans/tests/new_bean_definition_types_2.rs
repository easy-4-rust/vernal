//! 新创建的 BeanDefinition 层次结构类型（第二批）的覆盖测试。

use vernal_beans::{
    AbstractBeanDefinition, AnnotatedBeanDefinition, AnnotatedGenericBeanDefinition, Autowire,
    BeanDefinition, BeanDefinitionReader, BeanDefinitionResource, BeanNameGenerator,
    DefaultBeanNameGenerator, Scope,
};

// ── AbstractBeanDefinition ──────────────────────────────────────────

#[test]
fn abstract_bean_definition_new() {
    let def = AbstractBeanDefinition::new();
    assert_eq!(def.get_scope(), Scope::Singleton);
    assert!(!def.is_lazy_init());
    assert!(!def.is_primary());
    assert!(!def.is_fallback());
    assert!(def.is_autowire_candidate());
    assert!(!def.is_abstract());
    assert!(!def.is_synthetic());
    assert_eq!(def.get_role(), 0);
    assert!(def.get_description().is_none());
}

#[test]
fn abstract_bean_definition_setters() {
    let mut def = AbstractBeanDefinition::new();
    def.set_bean_class_name("com.example.Test");
    def.set_parent_name("parent");
    def.set_scope(Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    def.set_fallback(true);
    def.set_autowire_candidate(false);
    def.set_abstract_flag(true);
    def.set_synthetic(true);
    def.set_role(2);
    def.set_description("desc");
    def.set_autowire_mode(Autowire::ByType);
    def.set_init_method_name("init");
    def.set_destroy_method_name("destroy");
    def.set_factory_bean_name("fb");
    def.set_factory_method_name("fm");
    def.add_depends_on("dep1");

    assert_eq!(def.get_bean_class_name(), Some("com.example.Test"));
    assert_eq!(def.get_parent_name(), Some("parent"));
    assert_eq!(def.get_scope(), Scope::Transient);
    assert!(def.is_lazy_init());
    assert!(def.is_primary());
    assert!(def.is_fallback());
    assert!(!def.is_autowire_candidate());
    assert!(def.is_abstract());
    assert!(def.is_synthetic());
    assert_eq!(def.get_role(), 2);
    assert_eq!(def.get_description(), Some("desc"));
    assert_eq!(def.autowire_mode(), Autowire::ByType);
    assert_eq!(def.get_init_method_name(), Some("init"));
    assert_eq!(def.get_destroy_method_name(), Some("destroy"));
    assert_eq!(def.get_factory_bean_name(), Some("fb"));
    assert_eq!(def.get_factory_method_name(), Some("fm"));
    assert_eq!(def.get_depends_on(), &["dep1"]);
}

#[test]
fn abstract_bean_definition_default() {
    let def: AbstractBeanDefinition = Default::default();
    assert_eq!(def.get_scope(), Scope::Singleton);
}

#[test]
fn abstract_bean_definition_bean_definition_trait() {
    let mut def = AbstractBeanDefinition::new();
    def.set_bean_class_name("com.example.Test");
    def.set_scope(Scope::Transient);
    def.set_lazy_init(true);
    def.set_primary(true);
    def.set_fallback(true);
    def.set_autowire_candidate(false);
    def.set_abstract_flag(true);
    def.set_role(1);
    def.set_description("desc");
    def.set_parent_name("p");
    def.set_factory_bean_name("fb");
    def.set_factory_method_name("fm");
    def.set_init_method_name("init");
    def.set_destroy_method_name("destroy");

    let bd: &dyn BeanDefinition = &def;
    assert_eq!(bd.bean_class_name(), "com.example.Test");
    assert_eq!(bd.scope(), Scope::Transient);
    assert!(bd.is_lazy_init());
    assert!(bd.is_primary());
    assert!(bd.is_fallback());
    assert!(!bd.is_autowire_candidate());
    assert_eq!(bd.role(), 1);
    assert_eq!(bd.description(), Some("desc"));
    assert_eq!(bd.parent_name(), Some("p"));
    assert_eq!(bd.factory_bean_name(), Some("fb"));
    assert_eq!(bd.factory_method_name(), Some("fm"));
    assert_eq!(bd.init_method_name(), Some("init"));
    assert_eq!(bd.destroy_method_name(), Some("destroy"));
    assert!(bd.is_abstract());
}

// ── AnnotatedGenericBeanDefinition ─────────────────────────────────

#[test]
fn annotated_generic_definition_new() {
    let def = AnnotatedGenericBeanDefinition::new("com.example.MyConfiguration");
    assert_eq!(def.annotation_type_name(), "com.example.MyConfiguration");
    assert_eq!(def.bean_class_name(), "unknown");
    assert_eq!(def.scope(), Scope::Singleton);
}

#[test]
fn annotated_generic_definition_set_bean_class() {
    let mut def = AnnotatedGenericBeanDefinition::new("com.example.Config");
    def.inner_mut().set_bean_class_name("com.example.MyBean");
    assert_eq!(def.bean_class_name(), "com.example.MyBean");
}

#[test]
fn annotated_generic_definition_inner() {
    let def = AnnotatedGenericBeanDefinition::new("test.Config");
    assert_eq!(def.inner().get_bean_class_name(), None);
}

// ── DefaultBeanNameGenerator ──────────────────────────────────────

#[test]
fn default_name_generator_new() {
    let g = DefaultBeanNameGenerator::new();
    let def = AbstractBeanDefinition::new();
    let name = g.generate_bean_name(&def);
    assert_eq!(name, "unknown");
}

#[test]
fn default_name_generator_with_class() {
    let g = DefaultBeanNameGenerator::new();
    let mut def = AbstractBeanDefinition::new();
    def.set_bean_class_name("com.example.MyService");
    let name = g.generate_bean_name(&def);
    assert_eq!(name, "com.example.MyService");
}

#[test]
fn default_name_generator_default() {
    let g: DefaultBeanNameGenerator = Default::default();
    let mut def = AbstractBeanDefinition::new();
    def.set_bean_class_name("TestBean");
    assert_eq!(g.generate_bean_name(&def), "TestBean");
}

// ── BeanNameGenerator trait ────────────────────────────────────────

#[test]
fn bean_name_generator_trait_object() {
    let g: Box<dyn BeanNameGenerator> = Box::new(DefaultBeanNameGenerator::new());
    let def = AbstractBeanDefinition::new();
    let name = g.generate_bean_name(&def);
    assert!(!name.is_empty());
}

// ── BeanDefinitionReader trait ────────────────────────────────────

#[test]
fn bean_definition_reader_trait_object() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    struct MockReader;
    impl BeanDefinitionReader for MockReader {
        fn registry(&self) -> &dyn BeanDefinitionRegistry {
            unimplemented!("mock")
        }
        fn load_bean_definitions(
            &self,
            _resource: &dyn BeanDefinitionResource,
        ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
            Ok(0)
        }
        fn bean_name_generator(&self) -> Option<&dyn BeanNameGenerator> {
            None
        }
        fn set_bean_name_generator(&mut self, _generator: Box<dyn BeanNameGenerator>) {}
    }

    let reader = MockReader;
    let _: &dyn BeanDefinitionReader = &reader;
}

// ── BeanDefinitionResource trait ──────────────────────────────────

#[test]
fn bean_definition_resource_trait_object() {
    struct MockResource;
    impl BeanDefinitionResource for MockResource {
        fn description(&self) -> &str {
            "mock"
        }
        fn name(&self) -> &str {
            "mock.xml"
        }
        fn read_to_string(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok("<beans></beans>".to_string())
        }
    }

    let resource = MockResource;
    assert_eq!(resource.description(), "mock");
    assert_eq!(resource.name(), "mock.xml");
    assert!(resource.read_to_string().is_ok());
}

#[test]
fn bean_definition_resource_dynamic() {
    struct DynamicResource(String);
    impl BeanDefinitionResource for DynamicResource {
        fn description(&self) -> &str {
            &self.0
        }
        fn name(&self) -> &str {
            &self.0
        }
        fn read_to_string(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.0.clone())
        }
    }

    let r = DynamicResource("test.xml".to_string());
    assert_eq!(r.read_to_string().unwrap(), "test.xml");
}
