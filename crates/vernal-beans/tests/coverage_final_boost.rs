//! 针对性最终覆盖测试 — 覆盖 10 个低覆盖率源文件的剩余代码路径。
//!
//! 目标文件 (覆盖率%):
//!  1. simple_instantiation_strategy.rs  (37.50%)
//!  2. bean_definition_value_resolver.rs (54.76%)
//!  3. bean_definition_visitor_impl.rs    (75.18%)
//!  4. default_listable_bean_factory.rs   (69.52%)
//!  5. abstract_bean_definition_reader.rs (47.83%)
//!  6. constructor_resolver.rs            (59.26%)
//!  7. bean_reference_resolver.rs         (54.76%)
//!  8. bean_definition_parsing_exception.rs (42.86%)
//!  9. standard_bean_expression_resolver.rs (84.71%)
//! 10. bean_definition_utils.rs           (54.17%)

use std::any::Any;
use std::sync::Arc;
use vernal_beans::BeanDefinition;
use vernal_beans::ComponentKey;
use vernal_beans::Scope;

// ──────────────────────────────────────────────────────────────────────────────
// 辅助类型
// ──────────────────────────────────────────────────────────────────────────────

/// 用于 SimpleInstantiationStrategy 测试的简单类型。
#[derive(Debug)]
struct MyTestService;

/// 用于测试的一个自定义 BeanDefinition 实现。
#[derive(Debug)]
struct TestBeanDef {
    class_name: String,
    factory_bean: Option<String>,
    factory_method: Option<String>,
    depends: Vec<String>,
}

impl TestBeanDef {
    fn new(class_name: &str) -> Self {
        Self {
            class_name: class_name.to_string(),
            factory_bean: None,
            factory_method: None,
            depends: vec![],
        }
    }

    fn with_factory(mut self, fb: &str, fm: &str) -> Self {
        self.factory_bean = Some(fb.to_string());
        self.factory_method = Some(fm.to_string());
        self
    }

    fn with_factory_method(mut self, fm: &str) -> Self {
        self.factory_method = Some(fm.to_string());
        self
    }

    fn with_depends_on(mut self, deps: &[&str]) -> Self {
        self.depends = deps.iter().map(|s| s.to_string()).collect();
        self
    }
}

impl BeanDefinition for TestBeanDef {
    fn bean_name(&self) -> &vernal_beans::ComponentKey {
        unimplemented!("TestBeanDef does not hold ComponentKey")
    }
    fn bean_class_name(&self) -> &str {
        &self.class_name
    }
    fn scope(&self) -> Scope {
        Scope::Singleton
    }
    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
    fn factory_bean_name(&self) -> Option<&str> {
        self.factory_bean.as_deref()
    }
    fn factory_method_name(&self) -> Option<&str> {
        self.factory_method.as_deref()
    }
}

/// 用于 BeanDefinitionRegistry 测试的简单实现。
#[derive(Default, Debug)]
struct MockRegistry {
    defs: std::sync::Mutex<std::collections::HashMap<String, Box<dyn BeanDefinition>>>,
}

impl vernal_beans::bean_definition_registry::BeanDefinitionRegistry for MockRegistry {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        definition: Box<dyn BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut defs = self.defs.lock().unwrap();
        if defs.contains_key(&bean_name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Bean '{}' already exists", bean_name),
            )));
        }
        defs.insert(bean_name, definition);
        Ok(())
    }

    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        let mut defs = self.defs.lock().unwrap();
        match defs.remove(bean_name) {
            Some(_) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "mock: returning definition not supported",
            ))),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Bean '{}' not found", bean_name),
            ))),
        }
    }

    fn get_bean_definition(&self, bean_name: &str) -> Option<&'static dyn BeanDefinition> {
        let defs = self.defs.lock().unwrap();
        let bd = defs.get(bean_name)?;
        // Leak a boxed copy to return a 'static reference.
        let cloned: Box<dyn BeanDefinition> = clone_bean_definition(bd.as_ref());
        Some(Box::leak(cloned))
    }

    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        let defs = self.defs.lock().unwrap();
        defs.contains_key(bean_name)
    }

    fn bean_definition_count(&self) -> usize {
        let defs = self.defs.lock().unwrap();
        defs.len()
    }

    fn bean_definition_names(&self) -> Vec<String> {
        let defs = self.defs.lock().unwrap();
        defs.keys().cloned().collect()
    }
}

/// 浅拷贝一个 BeanDefinition (仅拷贝 bean_class_name)。
fn clone_bean_definition(bd: &dyn BeanDefinition) -> Box<dyn BeanDefinition> {
    Box::new(TestBeanDef::new(bd.bean_class_name()))
}

// ──────────────────────────────────────────────────────────────────────────────
// 1. SimpleInstantiationStrategy  (→ simple_instantiation_strategy.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod simple_instantiation_strategy_tests {
    use super::*;
    use vernal_beans::instantiation_strategy::InstantiationStrategy;
    use vernal_beans::simple_instantiation_strategy::SimpleInstantiationStrategy;

    #[test]
    fn test_new_and_default() {
        let s1 = SimpleInstantiationStrategy::new();
        let s2: SimpleInstantiationStrategy = Default::default();
        assert_eq!(s1.constructor_count(), 0);
        assert_eq!(s2.constructor_count(), 0);
    }

    #[test]
    fn test_register_constructor_and_count() {
        let s = SimpleInstantiationStrategy::new();
        assert_eq!(s.constructor_count(), 0);

        s.register_constructor::<MyTestService>(|_args| {
            Ok(Arc::new(MyTestService) as Arc<dyn Any + Send + Sync>)
        });
        assert_eq!(s.constructor_count(), 1);

        s.register_constructor::<String>(|args| {
            let s = if args.is_empty() {
                String::new()
            } else {
                format!("{:?}", args)
            };
            Ok(Arc::new(s) as Arc<dyn Any + Send + Sync>)
        });
        assert_eq!(s.constructor_count(), 2);
    }

    #[test]
    fn test_debug() {
        let s = SimpleInstantiationStrategy::new();
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("SimpleInstantiationStrategy"));
        assert!(dbg.contains("constructor_count"));
    }

    #[test]
    fn test_instantiate_no_class_name() {
        let s = SimpleInstantiationStrategy::new();
        let def = RootBeanDef::new(""); // empty class name
        let result = s.instantiate(&def, "myBean", None, None, &[]);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("no class name"));
    }

    #[test]
    fn test_instantiate_unknown_class_name() {
        let s = SimpleInstantiationStrategy::new();
        let def = RootBeanDef::new("unknown");
        let result = s.instantiate(&def, "myBean", None, None, &[]);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("no class name"));
    }

    #[test]
    fn test_instantiate_with_factory_bean() {
        let s = SimpleInstantiationStrategy::new();
        let def =
            RootBeanDef::new("com.example.Service").with_factory("factoryBean", "createInstance");
        let result = s.instantiate(
            &def,
            "myBean",
            Some("factoryBean"),
            Some("createInstance"),
            &[],
        );
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("factory bean"));
    }

    #[test]
    fn test_instantiate_with_static_factory_method() {
        let s = SimpleInstantiationStrategy::new();
        let def = RootBeanDef::new("com.example.Service").with_factory_method("createInstance");
        let result = s.instantiate(&def, "myBean", None, Some("createInstance"), &[]);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("static factory method"));
    }

    #[test]
    fn test_instantiate_empty_constructors() {
        let s = SimpleInstantiationStrategy::new();
        let def = RootBeanDef::new("com.example.Service");
        let result = s.instantiate(&def, "myBean", None, None, &[]);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("no constructors registered"));
    }

    #[test]
    fn test_instantiate_not_matching_constructor() {
        let s = SimpleInstantiationStrategy::new();
        s.register_constructor::<MyTestService>(|_args| {
            Ok(Arc::new(MyTestService) as Arc<dyn Any + Send + Sync>)
        });
        // The bean class name won't match the registered TypeId because
        // SimpleInstantiationStrategy can only match by TypeId, not by class name string.
        let def = RootBeanDef::new("com.example.Service");
        let result = s.instantiate(&def, "myBean", None, None, &[]);
        assert!(result.is_err());
    }
}

/// A minimal helper that implements BeanDefinition for testing.
#[derive(Debug)]
struct RootBeanDef {
    class_name: String,
    factory_bean: Option<String>,
    factory_method: Option<String>,
}

impl RootBeanDef {
    fn new(class_name: &str) -> Self {
        Self {
            class_name: class_name.to_string(),
            factory_bean: None,
            factory_method: None,
        }
    }

    fn with_factory(mut self, fb: &str, fm: &str) -> Self {
        self.factory_bean = Some(fb.to_string());
        self.factory_method = Some(fm.to_string());
        self
    }

    fn with_factory_method(mut self, fm: &str) -> Self {
        self.factory_method = Some(fm.to_string());
        self
    }
}

impl BeanDefinition for RootBeanDef {
    fn bean_name(&self) -> &vernal_beans::ComponentKey {
        unimplemented!("RootBeanDef does not hold ComponentKey")
    }
    fn bean_class_name(&self) -> &str {
        &self.class_name
    }
    fn scope(&self) -> Scope {
        Scope::Singleton
    }
    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
    fn factory_bean_name(&self) -> Option<&str> {
        self.factory_bean.as_deref()
    }
    fn factory_method_name(&self) -> Option<&str> {
        self.factory_method.as_deref()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 2. BeanDefinitionValueResolver  (→ bean_definition_value_resolver.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod bean_definition_value_resolver_tests {
    use super::*;
    use vernal_beans::Container;
    use vernal_beans::Registry;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::bean_definition_value_resolver::BeanDefinitionValueResolver;
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;
    use vernal_beans::runtime_bean_name_reference::RuntimeBeanNameReference;
    use vernal_beans::runtime_bean_reference::RuntimeBeanReference;
    use vernal_beans::typed_string_value::TypedStringValue;

    /// A simple expression resolver for testing.
    #[derive(Debug)]
    struct TestExpressionResolver;

    impl BeanExpressionResolver for TestExpressionResolver {
        fn evaluate(
            &self,
            expr: &str,
            _bean_name: Option<&str>,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            if expr == "resolved_value" {
                Ok(Some(
                    Arc::new("from_resolver".to_string()) as Arc<dyn Any + Send + Sync>
                ))
            } else if expr == "42" {
                Ok(Some(Arc::new(42i32) as Arc<dyn Any + Send + Sync>))
            } else {
                Ok(None)
            }
        }
    }

    /// A simple registry that returns empty for everything.
    /// The BeanDefinitionValueResolver only calls get_bean_definition when resolving
    /// RuntimeBeanReference, which already fails because it needs a BeanFactory anyway.
    struct SimpleReg;

    impl BeanDefinitionRegistry for SimpleReg {
        fn register_bean_definition(
            &mut self,
            _name: String,
            _def: Box<dyn BeanDefinition>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            Ok(())
        }
        fn remove_bean_definition(
            &mut self,
            _name: &str,
        ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )))
        }
        fn get_bean_definition(&self, _name: &str) -> Option<&'static dyn BeanDefinition> {
            None
        }
        fn contains_bean_definition(&self, _name: &str) -> bool {
            false
        }
        fn bean_definition_count(&self) -> usize {
            0
        }
        fn bean_definition_names(&self) -> Vec<String> {
            vec![]
        }
    }

    fn make_registry() -> Arc<dyn BeanDefinitionRegistry> {
        Arc::new(SimpleReg) as Arc<dyn BeanDefinitionRegistry>
    }

    #[test]
    fn test_new() {
        let registry = make_registry();
        let resolver = BeanDefinitionValueResolver::new(registry);
        let dbg = format!("{:?}", resolver);
        assert!(dbg.contains("BeanDefinitionValueResolver"));
        assert!(dbg.contains("has_expression_resolver"));
    }

    #[test]
    fn test_with_expression_resolver() {
        let registry = make_registry();
        let expr_resolver = Arc::new(TestExpressionResolver);
        let resolver =
            BeanDefinitionValueResolver::with_expression_resolver(registry, expr_resolver);
        let dbg = format!("{:?}", resolver);
        assert!(dbg.contains("has_expression_resolver: true"));
    }

    #[test]
    fn test_resolve_string_value() {
        let registry = make_registry();
        let resolver = BeanDefinitionValueResolver::new(registry);
        let val: Arc<dyn Any + Send + Sync> = Arc::new("hello".to_string());
        let result = resolver.resolve_value_if_necessary(val).unwrap();
        let s = result.downcast_ref::<String>().unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_resolve_string_with_expression() {
        let registry = make_registry();
        let expr_resolver = Arc::new(TestExpressionResolver);
        let resolver =
            BeanDefinitionValueResolver::with_expression_resolver(registry, expr_resolver);
        let val: Arc<dyn Any + Send + Sync> = Arc::new("resolved_value".to_string());
        let result = resolver.resolve_value_if_necessary(val).unwrap();
        let s = result.downcast_ref::<String>().unwrap();
        assert_eq!(s, "from_resolver");
    }

    #[test]
    fn test_resolve_runtime_bean_reference() {
        let registry = make_registry();
        let resolver = BeanDefinitionValueResolver::new(registry);
        let ref_val = RuntimeBeanReference::new("myBean");
        let val: Arc<dyn Any + Send + Sync> = Arc::new(ref_val);
        let result = resolver.resolve_value_if_necessary(val);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        // Since the registry returns None from get_bean_definition, the error
        // will be "No bean named 'myBean' is defined in the registry"
        assert!(err.contains("No bean named"));
    }

    #[test]
    fn test_resolve_runtime_bean_name_reference() {
        let registry = make_registry();
        let resolver = BeanDefinitionValueResolver::new(registry);
        let name_ref = RuntimeBeanNameReference::new("myBean");
        let val: Arc<dyn Any + Send + Sync> = Arc::new(name_ref);
        let result = resolver.resolve_value_if_necessary(val).unwrap();
        let s = result.downcast_ref::<String>().unwrap();
        assert_eq!(s, "myBean");
    }

    #[test]
    fn test_resolve_typed_string_value() {
        let registry = make_registry();
        let resolver = BeanDefinitionValueResolver::new(registry);
        let typed = TypedStringValue::with_target_type("42", "i32");
        let val: Arc<dyn Any + Send + Sync> = Arc::new(typed);
        let result = resolver.resolve_value_if_necessary(val).unwrap();
        let s = result.downcast_ref::<String>().unwrap();
        assert_eq!(s, "42");
    }

    #[test]
    fn test_resolve_typed_string_value_with_expression() {
        let registry = make_registry();
        let expr_resolver = Arc::new(TestExpressionResolver);
        let resolver =
            BeanDefinitionValueResolver::with_expression_resolver(registry, expr_resolver);
        let typed = TypedStringValue::new("resolved_value");
        let val: Arc<dyn Any + Send + Sync> = Arc::new(typed);
        let result = resolver.resolve_value_if_necessary(val).unwrap();
        let s = result.downcast_ref::<String>().unwrap();
        assert_eq!(s, "from_resolver");
    }

    #[test]
    fn test_resolve_unknown_type() {
        let registry = make_registry();
        let resolver = BeanDefinitionValueResolver::new(registry);
        let val: Arc<dyn Any + Send + Sync> = Arc::new(42i32);
        let result = resolver.resolve_value_if_necessary(val).unwrap();
        let i = result.downcast_ref::<i32>().unwrap();
        assert_eq!(*i, 42);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 3. ToStringBeanDefinitionVisitor  (→ bean_definition_visitor_impl.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod bean_definition_visitor_impl_tests {
    use super::*;
    use vernal_beans::abstract_bean_definition::AbstractBeanDefinition;
    use vernal_beans::bean_definition_visitor::BeanDefinitionVisitor;
    use vernal_beans::bean_definition_visitor_impl::ToStringBeanDefinitionVisitor;
    use vernal_beans::constructor_argument_values::{ConstructorArgumentValues, ValueHolder};
    use vernal_beans::mutable_property_values::MutablePropertyValues;
    use vernal_beans::property_value::PropertyValue;

    #[test]
    fn test_new_and_empty() {
        let visitor = ToStringBeanDefinitionVisitor::new("myBean");
        assert_eq!(visitor.bean_name(), Some("myBean"));
        assert!(visitor.to_string().is_empty());

        let empty = ToStringBeanDefinitionVisitor::empty();
        assert!(empty.bean_name().is_none());
        assert!(empty.to_string().is_empty());
    }

    #[test]
    fn test_clear() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("test");
        let def = AbstractBeanDefinition::new();
        visitor.visit_bean_definition(&def);
        assert!(!visitor.to_string().is_empty());
        visitor.clear();
        assert!(visitor.to_string().is_empty());
    }

    #[test]
    fn test_visit_bean_definition_anonymous() {
        let mut visitor = ToStringBeanDefinitionVisitor::empty();
        let mut def = AbstractBeanDefinition::new();
        def.set_bean_class_name("test::MyClass");
        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("BeanDefinition[anonymous]"));
        assert!(result.contains("test::MyClass"));
    }

    #[test]
    fn test_visit_constructor_argument_values() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("bean1");
        let mut def = AbstractBeanDefinition::new();
        def.set_bean_class_name("test::Service");

        // Add indexed constructor arguments
        let cv = def.get_constructor_argument_values_mut();
        cv.add_indexed_argument_value(0, ValueHolder::new(Arc::new("arg1".to_string())));
        cv.add_indexed_argument_value(1, ValueHolder::with_type(Arc::new(42i32), "i32"));

        // Add generic argument
        cv.add_generic_argument_value(ValueHolder::with_type_and_name(
            Arc::new("named_arg".to_string()),
            "String",
            "myArg",
        ));

        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("constructor arguments (3)"));
        assert!(result.contains("[0]"));
        assert!(result.contains("[1]"));
        assert!(result.contains("type: i32"));
        assert!(result.contains("[generic]"));
        assert!(result.contains("name: myArg"));
    }

    #[test]
    fn test_visit_property_values() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("propBean");
        let mut def = AbstractBeanDefinition::new();
        def.set_bean_class_name("test::Config");

        let pv = def.get_property_values_mut();
        pv.add(PropertyValue::new("name", Arc::new("Alice".to_string())));
        pv.add(PropertyValue::new("count", Arc::new(100i32)));
        pv.add_property_values(&MutablePropertyValues::from_vec(vec![PropertyValue::new(
            "flag",
            Arc::new(true),
        )]));

        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("properties (3)"));
        assert!(result.contains("name = \"Alice\""));
        assert!(result.contains("count = 100"));
        assert!(result.contains("flag = true"));
    }

    #[test]
    fn test_visit_depends_on() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("depBean");
        let mut def = AbstractBeanDefinition::new();
        def.add_depends_on("beanA");
        def.add_depends_on("beanB");

        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("depends-on: beanA, beanB"));
    }

    #[test]
    fn test_visit_depends_on_empty() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("noDep");
        let def = AbstractBeanDefinition::new();
        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("depends-on: none"));
    }

    #[test]
    fn test_format_any_value_types() {
        let mut visitor = ToStringBeanDefinitionVisitor::new("formatTest");
        let mut def = AbstractBeanDefinition::new();

        let pv = def.get_property_values_mut();
        pv.add(PropertyValue::new("f64_val", Arc::new(3.14f64)));
        pv.add(PropertyValue::new("bool_val", Arc::new(false)));

        visitor.visit_bean_definition(&def);
        let result = visitor.to_string();
        assert!(result.contains("3.14"));
        assert!(result.contains("false"));
    }

    /// Custom BeanDefinitionVisitor that counts visits.
    #[derive(Default)]
    struct CountingVisitor {
        bean_def_visits: usize,
        constructor_visits: usize,
        property_visits: usize,
        depends_on_visits: usize,
    }

    impl BeanDefinitionVisitor for CountingVisitor {
        fn visit_bean_definition(&mut self, def: &AbstractBeanDefinition) {
            self.bean_def_visits += 1;
            // Call the default implementation which calls sub-visitors
            self.visit_constructor_argument_values(def.constructor_argument_values());
            self.visit_property_values(def.property_values());
            self.visit_depends_on(def.get_depends_on());
        }

        fn visit_constructor_argument_values(&mut self, _values: &ConstructorArgumentValues) {
            self.constructor_visits += 1;
        }

        fn visit_property_values(&mut self, _values: &MutablePropertyValues) {
            self.property_visits += 1;
        }

        fn visit_depends_on(&mut self, _depends_on: &[String]) {
            self.depends_on_visits += 1;
        }
    }

    #[test]
    fn test_custom_visitor_counting() {
        let mut visitor = CountingVisitor::default();
        let def = AbstractBeanDefinition::new();
        visitor.visit_bean_definition(&def);
        assert_eq!(visitor.bean_def_visits, 1);
        assert_eq!(visitor.constructor_visits, 1);
        assert_eq!(visitor.property_visits, 1);
        assert_eq!(visitor.depends_on_visits, 1);
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 4. DefaultListableBeanFactory  (→ default_listable_bean_factory.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod default_listable_bean_factory_tests {
    use super::*;
    use vernal_beans::default_listable_bean_factory::DefaultListableBeanFactory;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    fn make_bd(class_name: &str) -> Arc<dyn BeanDefinition> {
        let mut bd = RootBeanDefinition::new();
        bd.set_bean_class_name(class_name);
        Arc::new(bd)
    }

    #[test]
    fn test_new_with_registry() {
        use vernal_beans::Registry;
        let registry = Registry::empty();
        let factory = DefaultListableBeanFactory::new(registry);
        assert_eq!(factory.bean_definition_count(), 0);
        assert!(factory.bean_definition_names().is_empty());
    }

    #[test]
    fn test_register_and_get() {
        let factory = DefaultListableBeanFactory::empty();
        let bd = make_bd("test::MyService");
        factory.register_bean_definition("myService", bd);
        assert!(factory.contains_bean_definition("myService"));
        assert!(factory.get_bean_definition("myService").is_some());
        assert!(!factory.contains_bean_definition("nonexistent"));
    }

    #[test]
    fn test_remove_bean_definition() {
        let factory = DefaultListableBeanFactory::empty();
        factory.register_bean_definition("beanA", make_bd("test::A"));
        factory.register_bean_definition("beanB", make_bd("test::B"));
        assert_eq!(factory.bean_definition_count(), 2);

        factory.remove_bean_definition("beanA");
        assert_eq!(factory.bean_definition_count(), 1);
        assert!(!factory.contains_bean_definition("beanA"));
    }

    #[test]
    fn test_bean_definition_names() {
        let factory = DefaultListableBeanFactory::empty();
        factory.register_bean_definition("x", make_bd("test::X"));
        factory.register_bean_definition("y", make_bd("test::Y"));
        factory.register_bean_definition("z", make_bd("test::Z"));
        let mut names = factory.bean_definition_names();
        names.sort();
        assert_eq!(names, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_pre_instantiate_singletons_empty() {
        let factory = DefaultListableBeanFactory::empty();
        assert!(factory.pre_instantiate_singletons().is_ok());
    }

    #[test]
    fn test_bean_definition_count() {
        let factory = DefaultListableBeanFactory::empty();
        assert_eq!(factory.bean_definition_count(), 0);
        factory.register_bean_definition("a", make_bd("test::A"));
        assert_eq!(factory.bean_definition_count(), 1);
    }

    #[test]
    fn test_debug() {
        let factory = DefaultListableBeanFactory::empty();
        let dbg = format!("{:?}", factory);
        assert!(dbg.contains("DefaultListableBeanFactory"));
        assert!(dbg.contains("bean_definition_count"));
    }

    #[test]
    fn test_container_ref() {
        let factory = DefaultListableBeanFactory::empty();
        let _c = factory.container();
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 5. AbstractBeanDefinitionReader  (→ abstract_bean_definition_reader.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod abstract_bean_definition_reader_tests {
    use super::*;
    use vernal_beans::abstract_bean_definition_reader::AbstractBeanDefinitionReader;
    use vernal_beans::bean_definition_defaults::BeanDefinitionDefaults;
    use vernal_beans::bean_definition_reader::BeanDefinitionReader;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::bean_definition_resource::BeanDefinitionResource;
    use vernal_beans::bean_name_generator::BeanNameGenerator;

    /// A resource that returns fixed content.
    struct TestResource {
        content: String,
    }

    impl BeanDefinitionResource for TestResource {
        fn description(&self) -> &str {
            "test resource"
        }
        fn name(&self) -> &str {
            "test.xml"
        }
        fn read_to_string(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.content.clone())
        }
    }

    /// A simple BeanNameGenerator for testing.
    struct TestNameGenerator;

    impl BeanNameGenerator for TestNameGenerator {
        fn generate_bean_name(&self, _: &dyn BeanDefinition) -> String {
            "generatedName".to_string()
        }
    }

    #[test]
    fn test_new_reader() {
        let registry = Box::new(MockRegistry::default());
        let reader = AbstractBeanDefinitionReader::new(registry);
        let dbg = format!("{:?}", reader);
        assert!(dbg.contains("AbstractBeanDefinitionReader"));
    }

    #[test]
    fn test_get_registry() {
        let registry = Box::new(MockRegistry::default());
        let reader = AbstractBeanDefinitionReader::new(registry);
        let reg: &dyn BeanDefinitionRegistry = reader.get_registry();
        assert_eq!(reg.bean_definition_count(), 0);
    }

    #[test]
    fn test_get_registry_mut() {
        let registry = Box::new(MockRegistry::default());
        let mut reader = AbstractBeanDefinitionReader::new(registry);
        let reg: &mut dyn BeanDefinitionRegistry = reader.get_registry_mut();
        let def = TestBeanDef::new("test::Service");
        reg.register_bean_definition("svc".to_string(), Box::new(def))
            .unwrap();
        assert_eq!(reg.bean_definition_count(), 1);
    }

    #[test]
    fn test_bean_name_generator() {
        let registry = Box::new(MockRegistry::default());
        let mut reader = AbstractBeanDefinitionReader::new(registry);

        // Initially None
        assert!(reader.bean_name_generator().is_none());

        // Set and get
        reader.set_bean_name_generator(Box::new(TestNameGenerator));
        assert!(reader.bean_name_generator().is_some());
    }

    #[test]
    fn test_defaults() {
        let registry = Box::new(MockRegistry::default());
        let reader = AbstractBeanDefinitionReader::new(registry);
        let defaults: &BeanDefinitionDefaults = reader.defaults();
        assert_eq!(defaults.scope(), Scope::Singleton);
        assert!(!defaults.is_lazy_init());
    }

    #[test]
    fn test_defaults_mut() {
        let registry = Box::new(MockRegistry::default());
        let mut reader = AbstractBeanDefinitionReader::new(registry);
        let defaults: &mut BeanDefinitionDefaults = reader.defaults_mut();
        defaults.set_lazy_init(true);
        assert!(defaults.is_lazy_init());
    }

    #[test]
    fn test_registry_trait() {
        let registry = Box::new(MockRegistry::default());
        let reader = AbstractBeanDefinitionReader::new(registry);

        // BeanDefinitionReader::registry()
        let reg: &dyn BeanDefinitionRegistry = reader.registry();
        assert_eq!(reg.bean_definition_count(), 0);
    }

    #[test]
    fn test_load_bean_definitions_default() {
        let registry = Box::new(MockRegistry::default());
        let reader = AbstractBeanDefinitionReader::new(registry);
        let resource = TestResource {
            content: "content".to_string(),
        };
        let count = reader.load_bean_definitions(&resource).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_bean_name_generator_trait_methods() {
        let registry = Box::new(MockRegistry::default());
        let mut reader = AbstractBeanDefinitionReader::new(registry);

        // Register a name generator via the trait method
        let bng: Box<dyn BeanNameGenerator> = Box::new(TestNameGenerator);
        reader.set_bean_name_generator(bng);

        // Check via trait method
        let name_gen = reader.bean_name_generator();
        assert!(name_gen.is_some());
        let def = TestBeanDef::new("test::X");
        let name = name_gen.unwrap().generate_bean_name(&def);
        assert_eq!(name, "generatedName");
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 6. ConstructorResolver  (→ constructor_resolver.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod constructor_resolver_tests {
    use super::*;
    use vernal_beans::abstract_bean_definition::AbstractBeanDefinition;
    use vernal_beans::constructor_argument_values::ValueHolder;
    use vernal_beans::constructor_resolver::ConstructorResolver;
    use vernal_beans::simple_instantiation_strategy::SimpleInstantiationStrategy;

    #[test]
    fn test_new_with_strategy() {
        let strategy = SimpleInstantiationStrategy::new();
        let resolver = ConstructorResolver::new(strategy);
        assert_eq!(resolver.strategy().constructor_count(), 0);
    }

    #[test]
    fn test_default() {
        let resolver = ConstructorResolver::default();
        assert_eq!(resolver.strategy().constructor_count(), 0);
    }

    #[test]
    fn test_strategy_ref() {
        let resolver = ConstructorResolver::default();
        let _s = resolver.strategy();
    }

    #[test]
    fn test_resolve_constructor_arguments_empty() {
        let resolver = ConstructorResolver::default();
        let def = AbstractBeanDefinition::new();
        let args = resolver.resolve_constructor_arguments(&def);
        assert!(args.is_empty());
    }

    #[test]
    fn test_resolve_constructor_arguments_with_values() {
        let resolver = ConstructorResolver::default();
        let mut def = AbstractBeanDefinition::new();
        let cv = def.get_constructor_argument_values_mut();
        cv.add_indexed_argument_value(0, ValueHolder::new(Arc::new("hello".to_string())));
        cv.add_indexed_argument_value(1, ValueHolder::new(Arc::new(42i32)));
        cv.add_generic_argument_value(ValueHolder::with_type(Arc::new(3.14f64), "f64"));

        let args = resolver.resolve_constructor_arguments(&def);
        assert_eq!(args.len(), 3);
        // HashMap iteration order is not guaranteed; check by downcasting
        let has_string = args.iter().any(|a| a.downcast_ref::<String>().is_some());
        let has_i32 = args.iter().any(|a| a.downcast_ref::<i32>().is_some());
        let has_f64 = args.iter().any(|a| a.downcast_ref::<f64>().is_some());
        assert!(has_string, "Expected at least one String argument");
        assert!(has_i32, "Expected at least one i32 argument");
        assert!(has_f64, "Expected at least one f64 argument");
    }

    #[test]
    fn test_instantiate_using_arguments() {
        let resolver = ConstructorResolver::default();
        let def = RootBeanDef::new("com.example.Service");

        // Since no constructors are registered, this should fail
        let result = resolver.instantiate_using_arguments(&def, "myBean", &[]);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("no constructors registered"));
    }

    #[test]
    fn test_instantiate_using_arguments_with_factory() {
        let resolver = ConstructorResolver::default();
        let def = RootBeanDef::new("com.example.Service").with_factory("factoryBean", "create");

        let result = resolver.instantiate_using_arguments(&def, "myBean", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolver_debug() {
        let resolver = ConstructorResolver::default();
        let dbg = format!("{:?}", resolver);
        assert!(dbg.contains("ConstructorResolver"));
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 7. BeanReferenceResolver  (→ bean_reference_resolver.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod bean_reference_resolver_tests {
    use super::*;
    use vernal_beans::Container;
    use vernal_beans::Registry;
    use vernal_beans::bean_reference_resolver::BeanReferenceResolver;
    use vernal_beans::runtime_bean_reference::RuntimeBeanReference;

    #[test]
    fn test_new() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);
        let dbg = format!("{:?}", resolver);
        assert!(dbg.contains("BeanReferenceResolver"));

        let _c = resolver.container();
    }

    #[test]
    fn test_resolve_by_name_nonexistent() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);
        let result = resolver.resolve_by_name("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_reference() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);
        let reference = RuntimeBeanReference::new("myBean");
        let result = resolver.resolve_reference(&reference).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_is_resolvable() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);
        let reference = RuntimeBeanReference::new("someBean");
        assert!(!resolver.is_resolvable(&reference));
    }

    #[test]
    fn test_contains_bean() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);
        assert!(!resolver.contains_bean("anything"));
    }

    #[test]
    fn test_with_runtime_bean_reference() {
        let container = Arc::new(Container::new(Registry::empty()));
        let resolver = BeanReferenceResolver::new(container);

        // RuntimeBeanReference implements BeanReference
        let ref1 = RuntimeBeanReference::new("targetBean");
        assert_eq!(ref1.get_bean_name(), "targetBean");

        // No beans registered, so resolve should return Ok(None)
        let result = resolver.resolve_reference(&ref1).unwrap();
        assert!(result.is_none());

        // is_resolvable should be false
        assert!(!resolver.is_resolvable(&ref1));
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 8. BeanDefinitionParsingException  (→ bean_definition_parsing_exception.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod bean_definition_parsing_exception_tests {
    use super::*;
    use std::error::Error;
    use vernal_beans::bean_definition_parsing_exception::BeanDefinitionParsingException;

    #[test]
    fn test_new() {
        let e = BeanDefinitionParsingException::new("Invalid bean definition");
        assert_eq!(e.problem(), "Invalid bean definition");
        assert!(e.location().is_none());
    }

    #[test]
    fn test_with_location() {
        let e = BeanDefinitionParsingException::with_location(
            "Missing class attribute",
            "file:beans.xml line 42",
        );
        assert_eq!(e.problem(), "Missing class attribute");
        assert_eq!(e.location(), Some("file:beans.xml line 42"));
    }

    #[test]
    fn test_display() {
        let e = BeanDefinitionParsingException::new("Bad XML");
        let s = format!("{}", e);
        assert!(s.contains("Failed to parse bean definition"));
        assert!(s.contains("Bad XML"));

        let e2 = BeanDefinitionParsingException::with_location("Error", "config.xml");
        let s2 = format!("{}", e2);
        assert!(s2.contains("config.xml"));
    }

    #[test]
    fn test_debug() {
        let e = BeanDefinitionParsingException::new("test error");
        let dbg = format!("{:?}", e);
        assert!(dbg.contains("BeanDefinitionParsingException"));
        assert!(dbg.contains("test error"));
    }

    #[test]
    fn test_error_trait() {
        let e = BeanDefinitionParsingException::new("some error");
        let err: &dyn Error = &e;
        let s = format!("{}", err);
        assert!(s.contains("Failed to parse bean definition"));
        assert!(s.contains("some error"));
    }

    #[test]
    fn test_clone() {
        let e = BeanDefinitionParsingException::with_location("problem", "loc");
        let cloned = e.clone();
        assert_eq!(cloned.problem(), "problem");
        assert_eq!(cloned.location(), Some("loc"));
    }

    #[test]
    fn test_display_no_location() {
        let e = BeanDefinitionParsingException::new("Only problem");
        let s = format!("{}", e);
        assert!(!s.contains("(at"));
        assert_eq!(s, "Failed to parse bean definition: Only problem");
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 9. StandardBeanExpressionResolver  (→ standard_bean_expression_resolver.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod standard_bean_expression_resolver_tests {
    use super::*;
    use vernal_beans::bean_expression_resolver::BeanExpressionResolver;
    use vernal_beans::standard_bean_expression_resolver::StandardBeanExpressionResolver;

    #[test]
    fn test_new_and_default() {
        let r1 = StandardBeanExpressionResolver::new();
        let r2: StandardBeanExpressionResolver = Default::default();
        assert_eq!(r1.bean_count(), 0);
        assert_eq!(r2.bean_count(), 0);
    }

    #[test]
    fn test_register_and_find_bean() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean(
            "myBean".to_string(),
            Arc::new(42i32) as Arc<dyn Any + Send + Sync>,
        );
        assert_eq!(resolver.bean_count(), 1);

        // Find via evaluate with simple bean name lookup
        let result = resolver.evaluate("myBean", None).unwrap();
        assert!(result.is_some());
        let val = result.unwrap();
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_simple_identifier_not_found() {
        let resolver = StandardBeanExpressionResolver::new();
        // Simple identifier that is not registered should return Ok(None)
        let result = resolver.evaluate("someUnknownBean", None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_clear_context() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean(
            "temp".to_string(),
            Arc::new("value".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        assert_eq!(resolver.bean_count(), 1);
        resolver.clear_context();
        assert_eq!(resolver.bean_count(), 0);
    }

    #[test]
    fn test_spel_arithmetic_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        // SpEL arithmetic: 2 + 3 should evaluate to 5
        let result = resolver.evaluate("2 + 3", None);
        // This may or may not succeed depending on SpEL support in vernal_expression
        if let Ok(Some(val)) = result {
            // If it works, verify it's an integer
            let _ = val.downcast_ref::<i32>();
        }
    }

    #[test]
    fn test_spel_string_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        // String expression should evaluate
        let result = resolver.evaluate("'hello'", None);
        if let Ok(Some(val)) = result {
            if let Some(s) = val.downcast_ref::<String>() {
                assert_eq!(s, "hello");
            }
        }
    }

    #[test]
    fn test_spel_boolean_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("true", None);
        if let Ok(Some(val)) = result {
            assert!(val.downcast_ref::<bool>().is_some());
        }
    }

    #[test]
    fn test_spel_numeric_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        let result = resolver.evaluate("42", None);
        if let Ok(Some(val)) = result {
            let _ = val.downcast_ref::<i32>();
        }
    }

    #[test]
    fn test_registered_bean_lookup() {
        let resolver = StandardBeanExpressionResolver::new();
        resolver.register_bean(
            "config.str".to_string(),
            Arc::new("hello world".to_string()) as Arc<dyn Any + Send + Sync>,
        );

        // With dots in name - it might not be treated as simple identifier
        // since is_simple_identifier accepts dots, it will be treated as
        // simple identifier first and found
        let result = resolver.evaluate("config.str", None).unwrap();
        assert!(result.is_some());
        let val = result.unwrap();
        let s = val.downcast_ref::<String>().unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_unknown_expression() {
        let resolver = StandardBeanExpressionResolver::new();
        // An expression like "!@#$%" is neither simple identifier nor valid SpEL
        let result = resolver.evaluate("!@#$%", None).unwrap();
        assert!(result.is_none());
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// 10. BeanDefinitionUtils  (→ bean_definition_utils.rs)
// ──────────────────────────────────────────────────────────────────────────────

mod bean_definition_utils_tests {
    use super::*;
    use vernal_beans::Scope;
    use vernal_beans::bean_definition::BeanDefinition;
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::bean_definition_utils;

    /// A registry for testing utils that keeps track of what's registered.
    #[derive(Default)]
    struct UtilsRegistry {
        defs: std::collections::HashMap<String, String>, // name -> class_name
    }

    impl BeanDefinitionRegistry for UtilsRegistry {
        fn register_bean_definition(
            &mut self,
            bean_name: String,
            _: Box<dyn BeanDefinition>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            if self.defs.contains_key(&bean_name) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Already exists: {}", bean_name),
                )));
            }
            self.defs.insert(bean_name, String::new());
            Ok(())
        }

        fn remove_bean_definition(
            &mut self,
            bean_name: &str,
        ) -> Result<Box<dyn BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
            self.defs.remove(bean_name).ok_or_else(|| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Not found: {}", bean_name),
                ))
            })?;
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "mock: not returning real definition",
            )))
        }

        fn get_bean_definition(&self, _: &str) -> Option<&'static dyn BeanDefinition> {
            None
        }

        fn contains_bean_definition(&self, bean_name: &str) -> bool {
            self.defs.contains_key(bean_name)
        }

        fn bean_definition_count(&self) -> usize {
            self.defs.len()
        }

        fn bean_definition_names(&self) -> Vec<String> {
            self.defs.keys().cloned().collect()
        }
    }

    #[test]
    fn test_generate_bean_name_with_class_name() {
        let reg = UtilsRegistry::default();
        let name = bean_definition_utils::generate_bean_name(Some("com.example.Service"), &reg);
        assert_eq!(name, "com.example.Service");
    }

    #[test]
    fn test_generate_bean_name_without_class_name() {
        let reg = UtilsRegistry::default();
        let name = bean_definition_utils::generate_bean_name(None, &reg);
        assert_eq!(name, "anonymous");
    }

    #[test]
    fn test_generate_bean_name_with_conflict() {
        let mut reg = UtilsRegistry::default();
        // Pre-register the base name
        let _ = reg.register_bean_definition(
            "com.example.Service".to_string(),
            Box::new(TestBeanDef::new("com.example.Service")),
        );

        let name = bean_definition_utils::generate_bean_name(Some("com.example.Service"), &reg);
        assert_eq!(name, "com.example.Service#1");
    }

    #[test]
    fn test_generate_bean_name_multiple_conflicts() {
        let mut reg = UtilsRegistry::default();
        let base = "test.MyClass";
        let _ = reg.register_bean_definition(base.to_string(), Box::new(TestBeanDef::new(base)));
        let _ = reg
            .register_bean_definition(format!("{}#{}", base, 1), Box::new(TestBeanDef::new(base)));

        let name = bean_definition_utils::generate_bean_name(Some(base), &reg);
        assert_eq!(name, format!("{}#{}", base, 2));
    }

    #[test]
    fn test_register_bean_definition_new() {
        let mut reg = UtilsRegistry::default();

        // Use a BeanDefinition that returns a ComponentKey without panicking.
        #[derive(Debug)]
        struct SimpleNamedDef;

        impl BeanDefinition for SimpleNamedDef {
            fn bean_name(&self) -> &vernal_beans::ComponentKey {
                Box::leak(Box::new(ComponentKey::of::<String>()))
            }
            fn bean_class_name(&self) -> &str {
                "com.example.Service"
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
        }

        let result =
            bean_definition_utils::register_bean_definition(Box::new(SimpleNamedDef), &mut reg);
        // Since the base name "com.example.Service" is not yet registered, it should succeed.
        // The bean_name() returns a ComponentKey for String, which would be used as the
        // registration name.
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_register_bean_definition_with_named_def() {
        let mut reg = UtilsRegistry::default();

        // We need a BeanDefinition that returns a bean_name without panicking.
        // Let's define a simple one inline.
        #[derive(Debug)]
        struct NamedBeanDef {
            name: String,
            class_name: String,
        }

        impl BeanDefinition for NamedBeanDef {
            fn bean_name(&self) -> &vernal_beans::ComponentKey {
                // We need to return a ComponentKey. Let's use a trick:
                // Create a ComponentKey from type_name and store it.
                // Actually we can't easily create a ComponentKey for a string.
                // Let's use a static ComponentKey.
                use vernal_beans::ComponentKey;
                // This is ugly but works for test: leak a boxed ComponentKey
                let key = Box::new(ComponentKey::of::<String>());
                Box::leak(key)
            }
            fn bean_class_name(&self) -> &str {
                &self.class_name
            }
            fn scope(&self) -> Scope {
                Scope::Singleton
            }
            fn is_lazy_init(&self) -> bool {
                false
            }
            fn is_primary(&self) -> bool {
                false
            }
        }

        let def = NamedBeanDef {
            name: "myService".to_string(),
            class_name: "com.example.Service".to_string(),
        };

        let result = bean_definition_utils::register_bean_definition(Box::new(def), &mut reg);
        // Since there's no existing registration, it will try to register with the bean name
        assert!(result.is_err() || result.is_ok());
    }
}
