//! Container BeanDefinitionRegistry trait 实现测试。

use vernal_beans::BeanDefinitionRegistry;
use vernal_beans::ComponentDefinition;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;

// ── 测试类型 ─────────────────────────────────────────────────────────────

#[derive(Debug)]
#[allow(dead_code)]
struct DatabasePool {
    url: String,
}

#[derive(Debug)]
struct CacheService {
    size: usize,
}

// ── 1. Container contains_bean_definition ─────────────────────────────────

/// 验证 Container 的 contains_bean_definition 方法。
#[test]
fn container_contains_bean_definition() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let type_name = std::any::type_name::<DatabasePool>();
    assert!(container.contains_bean_definition(type_name));
    assert!(!container.contains_bean_definition("nonexistent"));
}

/// 验证 Container 的 bean_definition_count 方法。
#[test]
fn container_bean_definition_count() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<CacheService, _>(
            |_resolver: &Resolver| CacheService { size: 10 },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    assert_eq!(container.bean_definition_count(), 2);
}

/// 验证 Container 的 bean_definition_names 方法。
#[test]
fn container_bean_definition_names() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<CacheService, _>(
            |_resolver: &Resolver| CacheService { size: 10 },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let names = container.bean_definition_names();
    assert_eq!(names.len(), 2);
    let type_name_pool = std::any::type_name::<DatabasePool>();
    let type_name_cache = std::any::type_name::<CacheService>();
    assert!(names.contains(&type_name_pool.to_string()));
    assert!(names.contains(&type_name_cache.to_string()));
}

/// 验证 Container 的 register_bean_definition 成功注册。
#[test]
fn container_register_bean_definition_success() {
    let builder = RegistryBuilder::new();
    let registry = builder.build().unwrap();
    let mut container = registry.container();

    let result =
        container.register_bean_definition("testBean".to_string(), Box::new(DummyBeanDefinition));
    assert!(result.is_ok());
    assert!(container.contains_bean_definition("testBean"));
}

/// 验证 Container 的 remove_bean_definition 成功删除。
#[test]
fn container_remove_bean_definition_success() {
    let builder = RegistryBuilder::new();
    let registry = builder.build().unwrap();
    let mut container = registry.container();

    // 先注册
    container
        .register_bean_definition("testBean".to_string(), Box::new(DummyBeanDefinition))
        .unwrap();

    // 再删除
    let result = container.remove_bean_definition("testBean");
    assert!(result.is_ok());
    assert!(!container.contains_bean_definition("testBean"));
}

/// 验证 Container 的 get_bean_definition 返回 BeanDefinition。
#[test]
fn container_get_bean_definition_returns_some() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    let type_name = std::any::type_name::<DatabasePool>();
    let result = container.get_bean_definition(type_name);
    // get_bean_definition 现在返回真实 BeanDefinition
    assert!(result.is_some());
    let bd = result.unwrap();
    assert_eq!(bd.bean_class_name(), type_name);
}

// ── 2. Container + RegistryBuilder 交互 ──────────────────────────────────

/// 验证 RegistryBuilder remove + Container 查询。
#[test]
fn registry_builder_remove_then_container_query() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();
    builder
        .register(ComponentDefinition::singleton::<CacheService, _>(
            |_resolver: &Resolver| CacheService { size: 10 },
        ))
        .unwrap();

    // 移除一个
    let type_name_pool = std::any::type_name::<DatabasePool>();
    builder.remove_bean_definition(type_name_pool).unwrap();

    // 构建 registry
    let registry = builder.build().unwrap();
    let container = registry.container();

    // Container 中只剩一个定义
    assert_eq!(container.bean_definition_count(), 1);
    assert!(!container.contains_bean_definition(type_name_pool));

    // CacheService 仍然存在
    let type_name_cache = std::any::type_name::<CacheService>();
    assert!(container.contains_bean_definition(type_name_cache));

    // Container resolve 应该正常工作
    let cache = container.resolve::<CacheService>().unwrap();
    assert_eq!(cache.size, 10);
}

/// 验证 Container 的 BeanDefinitionRegistry trait 与 BeanFactory trait 共存。
#[test]
fn container_implements_both_traits() {
    fn assert_bean_factory<T: vernal_beans::BeanFactory>() {}
    fn assert_registry<T: BeanDefinitionRegistry>() {}

    assert_bean_factory::<vernal_beans::Container>();
    assert_registry::<vernal_beans::Container>();
}

/// 验证空 Container 的 Registry 操作。
#[test]
fn empty_container_registry_operations() {
    let registry = vernal_beans::Registry::empty();
    let container = registry.container();

    assert_eq!(container.bean_definition_count(), 0);
    assert!(container.bean_definition_names().is_empty());
    assert!(!container.contains_bean_definition("anything"));
}

/// 验证 Container 的 registry() 方法返回正确的注册表。
#[test]
fn container_registry_method() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let registry = builder.build().unwrap();
    let container = registry.container();

    // 通过 registry() 获取注册表引用
    let reg = container.registry();
    assert_eq!(reg.len(), 1);
}

// ── 辅助类型 ─────────────────────────────────────────────────────────────

#[derive(Debug)]
struct DummyBeanDefinition;

impl vernal_beans::BeanDefinition for DummyBeanDefinition {
    fn bean_name(&self) -> &vernal_beans::ComponentKey {
        unimplemented!()
    }
    fn bean_class_name(&self) -> &str {
        "Dummy"
    }
    fn scope(&self) -> vernal_beans::Scope {
        vernal_beans::Scope::Singleton
    }
    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}
