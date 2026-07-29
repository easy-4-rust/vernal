//! Container 实现 ListableBeanFactory trait 的集成测试。
//!
//! 对应 Spring 的 `DefaultListableBeanFactoryTests` 中的 ListableBeanFactory 相关测试。

use std::any::Any;
use std::sync::Arc;
use vernal_beans::{
    BeanFactory, ComponentDefinition, ComponentKey, Container, RegistryBuilder,
    bean_definition::BeanDefinition, listable_bean_factory::ListableBeanFactory,
};

fn make_container() -> Container {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "hello".to_string()
    }));
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 42i32));
    b.register(ComponentDefinition::transient::<f64, _>(|_| 3.14f64));
    Container::new(b.build().unwrap())
}

fn make_empty_container() -> Container {
    Container::new(RegistryBuilder::new().build().unwrap())
}

fn as_listable(c: &Container) -> &dyn ListableBeanFactory {
    c
}

// ── contains_bean_definition ──────────────────────────────────────────

#[test]
fn listable_contains_bean_definition_found() {
    assert!(
        as_listable(&make_container()).contains_bean_definition(std::any::type_name::<String>())
    );
}

#[test]
fn listable_contains_bean_definition_not_found() {
    assert!(!as_listable(&make_empty_container()).contains_bean_definition("NonExistent"));
}

// ── bean_definition_count ─────────────────────────────────────────────

#[test]
fn listable_bean_definition_count_matches() {
    assert_eq!(as_listable(&make_container()).bean_definition_count(), 3);
}

#[test]
fn listable_bean_definition_count_empty() {
    assert_eq!(
        as_listable(&make_empty_container()).bean_definition_count(),
        0
    );
}

// ── bean_definition_names ─────────────────────────────────────────────

#[test]
fn listable_bean_definition_names_contains_all() {
    let c = make_container();
    let names = as_listable(&c).bean_definition_names();
    assert!(names.contains(&std::any::type_name::<String>().to_string()));
    assert!(names.contains(&std::any::type_name::<i32>().to_string()));
    assert!(names.contains(&std::any::type_name::<f64>().to_string()));
    assert_eq!(names.len(), 3);
}

// ── bean_names_for_type_id ────────────────────────────────────────────

#[test]
fn listable_bean_names_for_type_id_found() {
    let c = make_container();
    let names =
        as_listable(&c).bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
    assert_eq!(names.len(), 1);
    assert!(names.contains(&std::any::type_name::<String>().to_string()));
}

#[test]
fn listable_bean_names_for_type_id_not_found() {
    let c = make_container();
    let names =
        as_listable(&c).bean_names_for_type_id(std::any::TypeId::of::<Vec<i32>>(), true, true);
    assert!(names.is_empty());
}

#[test]
fn listable_bean_names_for_type_id_multiple() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "a".to_string()
    }));
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(vernal_beans::Qualifier::new("q2").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    let names =
        as_listable(&c).bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
    assert_eq!(names.len(), 2);
}

// ── beans_of_type_id ──────────────────────────────────────────────────

#[test]
fn listable_beans_of_type_id_found() {
    let c = make_container();
    let beans = as_listable(&c)
        .beans_of_type_id(std::any::TypeId::of::<String>(), true, true)
        .unwrap();
    assert_eq!(beans.len(), 1);
    let (_name, bean) = beans.into_iter().next().unwrap();
    assert!((*bean).downcast_ref::<String>().is_some());
}

#[test]
fn listable_beans_of_type_id_not_found() {
    let c = make_container();
    let beans = as_listable(&c)
        .beans_of_type_id(std::any::TypeId::of::<Vec<i32>>(), true, true)
        .unwrap();
    assert!(beans.is_empty());
}

#[test]
fn listable_beans_of_type_id_multiple() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "a".to_string()
    }));
    b.register(
        ComponentDefinition::singleton::<String, _>(|_| "b".to_string())
            .qualified(vernal_beans::Qualifier::new("q2").unwrap()),
    );
    let c = Container::new(b.build().unwrap());
    let beans = as_listable(&c)
        .beans_of_type_id(std::any::TypeId::of::<String>(), true, true)
        .unwrap();
    assert_eq!(beans.len(), 2);
}

#[test]
fn listable_beans_of_type_id_empty() {
    let c = make_empty_container();
    let beans = as_listable(&c)
        .beans_of_type_id(std::any::TypeId::of::<String>(), true, true)
        .unwrap();
    assert!(beans.is_empty());
}

// ── bean_post_processor_count ─────────────────────────────────────────

#[test]
fn listable_bean_post_processor_count() {
    use vernal_beans::bean_post_processor::BeanPostProcessor;
    let mut c = make_container();
    assert_eq!(as_listable(&c).bean_post_processor_count(), 0);

    struct DummyPP;
    impl BeanPostProcessor for DummyPP {
        fn post_process_before_initialization(
            &self,
            _b: Arc<dyn Any + Send + Sync>,
            _n: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(None)
        }
        fn post_process_after_initialization(
            &self,
            _b: Arc<dyn Any + Send + Sync>,
            _n: &str,
        ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>>
        {
            Ok(None)
        }
    }

    c.add_bean_post_processor(Arc::new(DummyPP));
    assert_eq!(as_listable(&c).bean_post_processor_count(), 1);
}

// ── contains_singleton_bean / contains_non_singleton_bean ─────────────

#[test]
fn listable_contains_singleton_bean_true() {
    assert!(as_listable(&make_container()).contains_singleton_bean());
}

#[test]
fn listable_contains_non_singleton_bean_true() {
    assert!(as_listable(&make_container()).contains_non_singleton_bean());
}

#[test]
fn listable_contains_singleton_bean_empty() {
    assert!(!as_listable(&make_empty_container()).contains_singleton_bean());
    assert!(!as_listable(&make_empty_container()).contains_non_singleton_bean());
}

#[test]
fn listable_contains_non_singleton_bean_false() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| {
        "a".to_string()
    }));
    let c = Container::new(b.build().unwrap());
    assert!(as_listable(&c).contains_singleton_bean());
    assert!(!as_listable(&c).contains_non_singleton_bean());
}

// ── bean_names_iterator ───────────────────────────────────────────────

#[test]
fn listable_bean_names_iterator() {
    let c = make_container();
    let names: Vec<String> = as_listable(&c).bean_names_iterator().collect();
    assert_eq!(names.len(), 3);
}

#[test]
fn listable_bean_names_iterator_empty() {
    let c = make_empty_container();
    let names: Vec<String> = as_listable(&c).bean_names_iterator().collect();
    assert!(names.is_empty());
}

// ── BeanFactoryUtils with ListableBeanFactory ─────────────────────────

#[test]
fn bean_factory_utils_with_listable() {
    use vernal_beans::bean_factory_utils::BeanFactoryUtils;

    let c = make_container();
    let names = BeanFactoryUtils::bean_definition_names(as_listable(&c));
    assert_eq!(names.len(), 3);

    let string_names =
        BeanFactoryUtils::bean_names_for_type(std::any::TypeId::of::<String>(), as_listable(&c));
    assert_eq!(string_names.len(), 1);

    let count =
        BeanFactoryUtils::count_beans_for_type(std::any::TypeId::of::<String>(), as_listable(&c));
    assert_eq!(count, 1);
}

// ── get_bean still works ──────────────────────────────────────────────

#[test]
fn listable_works_with_get_bean() {
    let c = make_container();
    let bean = c
        .get_bean_by_type_id(std::any::TypeId::of::<String>())
        .unwrap();
    let s = (*bean).downcast_ref::<String>().unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn listable_bean_names_after_dynamic_register() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    let mut c = make_container();
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("dynamic.test.Bean");
    c.register_bean_definition("dynamic_bean".to_string(), Box::new(def))
        .unwrap();

    let names = as_listable(&c).bean_definition_names();
    assert!(names.contains(&"dynamic_bean".to_string()));
}

// ── Edge: beans_of_type_id resolves correctly ─────────────────────────

#[test]
fn listable_beans_of_type_id_values_are_resolved() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<i32, _>(|_| 100i32));
    b.register(
        ComponentDefinition::singleton::<i32, _>(|_| 200i32)
            .qualified(vernal_beans::Qualifier::new("alt").unwrap()),
    );
    let c = Container::new(b.build().unwrap());

    let beans = as_listable(&c)
        .beans_of_type_id(std::any::TypeId::of::<i32>(), true, true)
        .unwrap();
    assert_eq!(beans.len(), 2);

    let values: Vec<i32> = beans
        .values()
        .filter_map(|b| (*b).downcast_ref::<i32>().copied())
        .collect();
    assert!(values.contains(&100i32));
    assert!(values.contains(&200i32));
}

// ── Edge: bean_names_for_type_id with dynamic_definitions ──────────────

#[test]
fn listable_bean_names_for_type_id_with_dynamic() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    let mut c = make_container();
    // 注册一个 dynamic definition (type_name 包含 "String")
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("dynamic.String.Def");
    c.register_bean_definition("dynamic_bean".to_string(), Box::new(def))
        .unwrap();

    // bean_names_for_type_id should include registry definitions + all non-deleted dynamic definitions
    // (dynamic definitions don't have TypeId info, so they appear for all type queries)
    let names =
        as_listable(&c).bean_names_for_type_id(std::any::TypeId::of::<String>(), true, true);
    assert!(names.len() >= 1);
}

// ── Edge: beans_of_type_id with resolve failure ───────────────────────

#[test]
fn listable_beans_of_type_id_resolve_failure() {
    let mut b = RegistryBuilder::new();
    b.register(ComponentDefinition::singleton::<String, _>(|_| -> String {
        panic!("deliberate panic");
    }));
    let c = Container::new(b.build().unwrap());

    // The panic should propagate through resolve_definition
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = as_listable(&c).beans_of_type_id(std::any::TypeId::of::<String>(), true, true);
    }));
    assert!(result.is_err());
}

// ── Edge: bean_names_for_type_id with deleted dynamic definition ──────

#[test]
fn listable_bean_names_after_delete() {
    use vernal_beans::bean_definition_registry::BeanDefinitionRegistry;
    use vernal_beans::root_bean_definition::RootBeanDefinition;

    let mut c = make_container();
    let mut def = RootBeanDefinition::new();
    def.set_bean_class_name("temp.Bean");
    c.register_bean_definition("temp_bean".to_string(), Box::new(def))
        .unwrap();
    // 删除后变为 __DELETED__ 标记
    let _removed = c.remove_bean_definition("temp_bean").unwrap();

    // 已删除的 dynamic definition 应该不在 names 列表中
    let names = as_listable(&c).bean_definition_names();
    assert!(!names.contains(&"temp_bean".to_string()));
}
