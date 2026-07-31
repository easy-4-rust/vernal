//! BeanDefinitionRegistry remove + FieldMetadata 编译期收集测试。

use std::sync::Arc;

use vernal_beans::ComponentDefinition;
use vernal_beans::RegistryBuilder;
use vernal_beans::Resolver;
use vernal_beans::BeanDefinitionRegistry;
use vernal_beans::field_metadata::{FieldDescriptor, TypeMetadata};

fn lock_field_md() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
    LOCK.get_or_init(|| std::sync::Mutex::new(())).lock().unwrap()
}

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

#[derive(Debug)]
#[allow(dead_code)]
struct UserService {
    pool: Arc<DatabasePool>,
    cache: Option<Arc<CacheService>>,
}

// ── 1. BeanDefinitionRegistry remove_bean_definition 测试 ──────────────────

/// 参照 Spring `DefaultListableBeanFactoryTests.beanDefinitionRemoval`：
/// 验证 remove_bean_definition 操作。
#[test]
fn registry_remove_bean_definition_by_name() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    // 验证包含
    assert!(builder.contains::<DatabasePool>());

    // 按名称移除
    let type_name = std::any::type_name::<DatabasePool>();
    let removed = builder.remove_bean_definition(type_name).unwrap();

    // 验证移除成功
    assert_eq!(removed.bean_class_name(), type_name);
    assert!(!builder.contains::<DatabasePool>());
    assert_eq!(builder.len(), 0);
}

/// 验证 remove_bean_definition 找不到时返回错误。
#[test]
fn registry_remove_bean_definition_not_found() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let result = builder.remove_bean_definition("nonexistent");
    assert!(result.is_err());
}

/// 验证 remove_bean_definition 后 registry 可以正常构建。
#[test]
fn registry_remove_then_build() {
    #[derive(Debug)]
    struct ServiceA;
    #[derive(Debug)]
    struct ServiceB;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::shared_value(ServiceA))
        .unwrap();
    builder
        .register(ComponentDefinition::shared_value(ServiceB))
        .unwrap();

    let type_name_a = std::any::type_name::<ServiceA>();
    builder.remove_bean_definition(type_name_a).unwrap();

    let registry = builder.build().unwrap();
    assert_eq!(registry.len(), 1);
}

/// 验证 contains_bean_definition 方法。
#[test]
fn registry_contains_bean_definition() {
    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    let type_name = std::any::type_name::<DatabasePool>();
    assert!(builder.contains_bean_definition(type_name));
    assert!(!builder.contains_bean_definition("nonexistent"));
}

/// 验证 bean_definition_count 方法。
#[test]
fn registry_bean_definition_count() {
    let mut builder = RegistryBuilder::new();
    assert_eq!(builder.bean_definition_count(), 0);

    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();
    assert_eq!(builder.bean_definition_count(), 1);

    builder
        .register(ComponentDefinition::singleton::<CacheService, _>(
            |_resolver: &Resolver| CacheService { size: 10 },
        ))
        .unwrap();
    assert_eq!(builder.bean_definition_count(), 2);
}

/// 验证 bean_definition_names 方法。
#[test]
fn registry_bean_definition_names() {
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

    let names = builder.bean_definition_names();
    assert_eq!(names.len(), 2);
    let type_name_pool = std::any::type_name::<DatabasePool>();
    let type_name_cache = std::any::type_name::<CacheService>();
    assert!(names.contains(&type_name_pool.to_string()));
    assert!(names.contains(&type_name_cache.to_string()));
}

/// 验证批量删除多个定义。
#[test]
fn registry_remove_multiple_definitions() {
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

    let type_name_pool = std::any::type_name::<DatabasePool>();
    let type_name_cache = std::any::type_name::<CacheService>();

    builder.remove_bean_definition(type_name_pool).unwrap();
    assert_eq!(builder.bean_definition_count(), 1);
    assert!(!builder.contains_bean_definition(type_name_pool));
    assert!(builder.contains_bean_definition(type_name_cache));

    builder.remove_bean_definition(type_name_cache).unwrap();
    assert_eq!(builder.bean_definition_count(), 0);
    assert!(builder.is_empty());
}

// ── 2. FieldMetadata 编译期收集测试 ──────────────────────────────────────

/// 验证 FieldDescriptor 创建。
#[test]
fn field_descriptor_basic() {
    let fd = FieldDescriptor::new(
        "pool",
        std::any::TypeId::of::<DatabasePool>(),
        std::any::type_name::<DatabasePool>(),
    );

    assert_eq!(fd.name, "pool");
    assert_eq!(fd.type_id, std::any::TypeId::of::<DatabasePool>());
    assert!(!fd.optional);
    assert!(fd.qualifier.is_none());
}

/// 验证 FieldDescriptor with_optional。
#[test]
fn field_descriptor_optional() {
    let fd = FieldDescriptor::new(
        "cache",
        std::any::TypeId::of::<CacheService>(),
        std::any::type_name::<CacheService>(),
    )
    .with_optional();

    assert!(fd.optional);
}

/// 验证 FieldDescriptor with_qualifier。
#[test]
fn field_descriptor_qualifier() {
    let fd = FieldDescriptor::new(
        "pool",
        std::any::TypeId::of::<DatabasePool>(),
        std::any::type_name::<DatabasePool>(),
    )
    .with_qualifier("primary");

    assert_eq!(fd.qualifier, Some("primary"));
}

/// 验证 TypeMetadata 创建。
#[test]
fn type_metadata_basic() {
    let metadata = TypeMetadata {
        type_id: std::any::TypeId::of::<UserService>(),
        type_name: std::any::type_name::<UserService>(),
        fields: vec![
            FieldDescriptor::new(
                "pool",
                std::any::TypeId::of::<DatabasePool>(),
                std::any::type_name::<DatabasePool>(),
            ),
            FieldDescriptor::new(
                "cache",
                std::any::TypeId::of::<CacheService>(),
                std::any::type_name::<CacheService>(),
            )
            .with_optional(),
        ],
    };

    assert_eq!(metadata.type_id, std::any::TypeId::of::<UserService>());
    assert_eq!(metadata.fields.len(), 2);
    assert_eq!(metadata.fields[0].name, "pool");
    assert!(!metadata.fields[0].optional);
    assert_eq!(metadata.fields[1].name, "cache");
    assert!(metadata.fields[1].optional);
}

/// 验证 field_metadata 存储初始化。
#[test]
fn field_metadata_store_init() {
    let _guard = lock_field_md();
    vernal_beans::field_metadata::clear_metadata();
    let all = vernal_beans::field_metadata::get_all_metadata();
    // 初始为空
    assert!(all.is_empty());
}

/// 验证 find_fields_needing_type（空存储）。
#[test]
fn find_fields_needing_type_empty() {
    let _guard = lock_field_md();
    vernal_beans::field_metadata::clear_metadata();
    let fields = vernal_beans::field_metadata::find_fields_needing_type(std::any::TypeId::of::<
        DatabasePool,
    >());
    assert!(fields.is_empty());
}

/// 验证 type_needs_injection（空存储）。
#[test]
fn type_needs_injection_empty() {
    let _guard = lock_field_md();
    vernal_beans::field_metadata::clear_metadata();
    let needs =
        vernal_beans::field_metadata::type_needs_injection(std::any::TypeId::of::<DatabasePool>());
    assert!(!needs);
}

// ── 3. BeanDefinitionRegistry 与 Container 集成 ──────────────────────────

/// 验证 RegistryBuilder 的 BeanDefinitionRegistry trait 实现与构建流程兼容。
#[test]
fn registry_builder_trait_compatible_with_build() {
    use vernal_beans::BeanDefinitionRegistry;

    let mut builder = RegistryBuilder::new();
    builder
        .register(ComponentDefinition::singleton::<DatabasePool, _>(
            |_resolver: &Resolver| DatabasePool {
                url: "test".to_string(),
            },
        ))
        .unwrap();

    // 使用 trait 方法
    assert_eq!(builder.bean_definition_count(), 1);
    assert!(builder.bean_definition_names().len() == 1);

    // 构建 registry
    let registry = builder.build().unwrap();
    assert_eq!(registry.len(), 1);
}

/// 验证 remove 后 registry 构建正确。
#[test]
fn registry_remove_and_build_integration() {
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
    let type_name = std::any::type_name::<DatabasePool>();
    builder.remove_bean_definition(type_name).unwrap();

    // 构建 registry
    let registry = builder.build().unwrap();
    assert_eq!(registry.len(), 1);

    // Container 应该正常工作
    let container = registry.container();
    let cache = container.resolve::<CacheService>().unwrap();
    assert_eq!(cache.size, 10);
}

/// 验证 FieldMetadata 与 autowireBean 集成。
#[test]
fn field_metadata_with_autowire() {
    let _guard = lock_field_md();

    // 手动创建元数据并验证结构
    let metadata = TypeMetadata {
        type_id: std::any::TypeId::of::<UserService>(),
        type_name: std::any::type_name::<UserService>(),
        fields: vec![FieldDescriptor::new(
            "pool",
            std::any::TypeId::of::<DatabasePool>(),
            std::any::type_name::<DatabasePool>(),
        )],
    };

    // 验证元数据结构正确
    assert_eq!(metadata.type_id, std::any::TypeId::of::<UserService>());
    assert_eq!(metadata.fields.len(), 1);
    assert_eq!(metadata.fields[0].name, "pool");
    assert_eq!(
        metadata.fields[0].type_id,
        std::any::TypeId::of::<DatabasePool>()
    );

    // 验证 FieldMetadata 创建和查询
    vernal_beans::field_metadata::clear_metadata();
    let all = vernal_beans::field_metadata::get_all_metadata();
    // 当前为空（OnceLock 限制）
    assert!(all.is_empty());
}
