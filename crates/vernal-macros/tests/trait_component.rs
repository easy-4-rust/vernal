//! Component 宏对 Trait 单值、命名与全部实现字段的运行时合同测试。

use vernal_beans::{Component, Qualifier, RegistryBuilder, TraitBinding};

mod trait_component_support;

use trait_component_support::{
    AllGreetingConsumer, ChineseGreeting, EnglishGreeting, Greeting, NamedGreetingConsumer,
    PrimaryGreetingConsumer,
};

#[test]
fn derive_generates_trait_binding_dependency_metadata() {
    let english = Qualifier::new("english").expect("qualifier should be valid");
    let mut registry = RegistryBuilder::new();
    registry
        .register(EnglishGreeting::definition())
        .expect("English component should register");
    registry
        .register(ChineseGreeting::definition())
        .expect("Chinese component should register");
    registry
        .bind(
            TraitBinding::new::<dyn Greeting, EnglishGreeting, _>(|service| service)
                .qualified(english),
        )
        .expect("English named binding should register");
    registry
        .bind(TraitBinding::new::<dyn Greeting, ChineseGreeting, _>(|service| service).primary())
        .expect("Chinese primary binding should register");
    registry
        .register(PrimaryGreetingConsumer::definition())
        .expect("primary consumer should register");
    registry
        .register(NamedGreetingConsumer::definition())
        .expect("named consumer should register");
    registry
        .register(AllGreetingConsumer::definition())
        .expect("all consumer should register");

    let container = registry
        .build()
        .expect("generated trait graph should be valid")
        .container();
    let primary = container
        .resolve::<PrimaryGreetingConsumer>()
        .expect("primary consumer should resolve");
    let named = container
        .resolve::<NamedGreetingConsumer>()
        .expect("named consumer should resolve");
    let all = container
        .resolve::<AllGreetingConsumer>()
        .expect("all consumer should resolve");

    assert_eq!(primary.greeting.message(), "你好");
    assert_eq!(named.greeting.message(), "hello");
    assert_eq!(
        all.greetings
            .iter()
            .map(|service| service.message())
            .collect::<Vec<_>>(),
        ["hello", "你好"]
    );
}
