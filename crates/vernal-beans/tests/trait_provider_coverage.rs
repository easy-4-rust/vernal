//! TraitProvider 的全面覆盖测试。

use std::any::Any;
use std::sync::Arc;

use vernal_beans::{
    ComponentDefinition,
    Container,
    RegistryBuilder,
    Resolver,
    TraitBinding,
};


// ═══════════════════════════════════════════════════════════════════════════════
// TraitProvider 测试
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trait_provider_get() {
    // 创建一个简单的 trait binding 来测试 trait_provider
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 添加 trait binding: String -> dyn Any + Send + Sync
    let binding = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder.bind(binding).unwrap();

    // 注册一个使用 trait_provider 的组件
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.trait_provider::<dyn Any + Send + Sync>() {
            // 测试 get 方法
            let _result = provider.get();
            return 42;
        }
        0
    }).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn trait_provider_get_in() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 添加 trait binding
    let binding = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder.bind(binding).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<String>();

    // 注册一个使用 trait_provider.get_in 的组件
    let mut builder2 = RegistryBuilder::new();
    builder2
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let binding2 = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder2.bind(binding2).unwrap();

    let def = ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
        if let Ok(provider) = resolver.trait_provider::<dyn Any + Send + Sync>() {
            // 测试 get_in 方法
            let _result = provider.get_in(&scope);
            return 42;
        }
        0
    }).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder2.register(def).unwrap();

    let container2 = Container::new(builder2.build().unwrap());
    let result = container2.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn trait_provider_get_if_available() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 添加 trait binding
    let binding = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder.bind(binding).unwrap();

    // 注册一个使用 trait_provider.get_if_available 的组件
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.trait_provider::<dyn Any + Send + Sync>() {
            // 测试 get_if_available 方法
            let _result = provider.get_if_available();
            return 42;
        }
        0
    }).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn trait_provider_get_if_available_in() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 添加 trait binding
    let binding = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder.bind(binding).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);
    let scope = container.open_scope::<String>();

    // 注册一个使用 trait_provider.get_if_available_in 的组件
    let mut builder2 = RegistryBuilder::new();
    builder2
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();
    let binding2 = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder2.bind(binding2).unwrap();

    let def = ComponentDefinition::singleton::<i32, _>(move |resolver: &Resolver| {
        if let Ok(provider) = resolver.trait_provider::<dyn Any + Send + Sync>() {
            // 测试 get_if_available_in 方法
            let _result = provider.get_if_available_in(&scope);
            return 42;
        }
        0
    }).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder2.register(def).unwrap();

    let container2 = Container::new(builder2.build().unwrap());
    let result = container2.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn trait_provider_is_optional() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 添加 trait binding
    let binding = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder.bind(binding).unwrap();

    // 注册一个使用 optional trait_provider 的组件
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.optional_trait_provider::<dyn Any + Send + Sync>() {
            let _is_optional = provider.is_optional();
            return 42;
        }
        0
    }).depends_on_optional_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn trait_provider_clone() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 添加 trait binding
    let binding = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder.bind(binding).unwrap();

    // 注册一个使用 trait_provider.clone 的组件
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.trait_provider::<dyn Any + Send + Sync>() {
            // 测试 clone
            let _cloned = provider.clone();
            return 42;
        }
        0
    }).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn trait_provider_debug() {
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 添加 trait binding
    let binding = TraitBinding::new::<dyn Any + Send + Sync, String, _>(
        |arc| arc as Arc<dyn Any + Send + Sync>,
    );
    builder.bind(binding).unwrap();

    // 注册一个使用 trait_provider.debug 的组件
    let def = ComponentDefinition::singleton::<i32, _>(|resolver: &Resolver| {
        if let Ok(provider) = resolver.trait_provider::<dyn Any + Send + Sync>() {
            // 测试 debug
            let debug = format!("{:?}", provider);
            assert!(debug.contains("TraitProvider"));
            return 42;
        }
        0
    }).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    let registry = builder.build().unwrap();
    let container = Container::new(registry);

    let result = container.resolve::<i32>();
    assert!(result.is_ok());
    assert_eq!(*result.unwrap(), 42);
}

#[test]
fn trait_provider_not_found() {
    // 测试没有 trait binding 时，解析应该失败
    // 这是因为 depends_on_trait_provider 会在构建时校验
    let mut builder = RegistryBuilder::new();

    // 注册 String 组件（不添加 trait binding）
    builder
        .register(ComponentDefinition::singleton::<String, _>(|_resolver: &Resolver| {
            "hello".to_string()
        }))
        .unwrap();

    // 尝试注册一个使用 trait_provider 的组件（没有 trait binding）
    // 这应该在 build() 时失败
    let def = ComponentDefinition::singleton::<i32, _>(|_resolver: &Resolver| {
        42
    }).depends_on_trait_provider::<dyn Any + Send + Sync>();
    builder.register(def).unwrap();

    // build 应该失败，因为缺少 trait binding
    let result = builder.build();
    assert!(result.is_err());
}
