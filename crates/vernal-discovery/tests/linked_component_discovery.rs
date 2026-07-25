//! 可选链接期组件发现运行时合同测试。

mod discovery_support;

use std::sync::Arc;

use discovery_support::{AuditWorker, Database, ManualComponent, OrderService};
use vernal_context::VernalApplicationBuilder;
use vernal_discovery::{LinkedComponentCatalog, LinkedComponentCatalogError};
use vernal_beans::{Component, ComponentDefinition, RegistryBuilder};

#[test]
fn explicit_group_selection_is_deterministic_and_excludes_other_groups() {
    let catalog = LinkedComponentCatalog::scan(["test.core"]).expect("core discovery catalog");
    let names = catalog
        .registrations()
        .iter()
        .map(|registration| registration.name())
        .collect::<Vec<_>>();

    assert_eq!(catalog.len(), 2);
    assert!(!catalog.is_empty());
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));

    let mut registry = RegistryBuilder::new();
    catalog
        .install(&mut registry)
        .expect("atomic catalog install");
    let container = registry.build().expect("discovered graph").container();
    let service = container
        .resolve::<OrderService>()
        .expect("discovered order service");
    assert!(Arc::ptr_eq(
        service.database(),
        &container
            .resolve::<Database>()
            .expect("discovered database")
    ));
    assert!(container.resolve::<AuditWorker>().is_err());
}

#[test]
fn one_catalog_installs_into_isolated_containers_without_sharing_singletons() {
    let catalog = LinkedComponentCatalog::scan(["test.core"]).expect("core discovery catalog");
    let mut first_registry = RegistryBuilder::new();
    let mut second_registry = RegistryBuilder::new();
    catalog
        .install(&mut first_registry)
        .expect("first catalog install");
    catalog
        .install(&mut second_registry)
        .expect("second catalog install");
    let first = first_registry.build().expect("first graph").container();
    let second = second_registry.build().expect("second graph").container();

    let first_service = first
        .resolve::<OrderService>()
        .expect("first order service");
    let second_service = second
        .resolve::<OrderService>()
        .expect("second order service");
    assert!(!Arc::ptr_eq(&first_service, &second_service));
    assert!(!Arc::ptr_eq(
        first_service.database(),
        second_service.database()
    ));
}

#[test]
fn catalog_install_is_atomic_when_an_existing_definition_conflicts() {
    let catalog = LinkedComponentCatalog::scan(["test.core"]).expect("core discovery catalog");
    let mut registry = RegistryBuilder::new();
    registry
        .register(Database::definition())
        .expect("existing database definition");
    let before = registry.len();

    catalog
        .install(&mut registry)
        .expect_err("duplicate discovered database must reject complete batch");
    assert_eq!(registry.len(), before);
}

#[test]
fn empty_unknown_and_duplicate_groups_fail_closed() {
    assert_eq!(
        LinkedComponentCatalog::scan::<[&str; 0], &str>([]).expect_err("empty group selection"),
        LinkedComponentCatalogError::EmptySelection
    );
    assert!(matches!(
        LinkedComponentCatalog::scan(["test.missing"]).expect_err("unknown group"),
        LinkedComponentCatalogError::MissingGroup { .. }
    ));
    assert!(matches!(
        LinkedComponentCatalog::scan(["test.duplicate"]).expect_err("duplicate group"),
        LinkedComponentCatalogError::DuplicateRegistration { .. }
    ));
}

#[test]
fn ordinary_component_definition_remains_explicit_and_unchanged() {
    let definition = ComponentDefinition::singleton::<ManualComponent, _>(|_| ManualComponent);
    let mut registry = RegistryBuilder::new();
    registry
        .register(definition)
        .expect("ordinary explicit definition");
    assert_eq!(registry.len(), 1);
}

#[tokio::test]
async fn selected_catalog_enters_the_high_level_application_context_explicitly() {
    let catalog = LinkedComponentCatalog::scan(["test.core"]).expect("core discovery catalog");
    let mut builder = VernalApplicationBuilder::current().expect("Tokio application builder");
    builder
        .register_all(catalog.component_definitions())
        .expect("discovered application definitions");
    let context = builder.build().expect("application context");
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");

    let service = context
        .container()
        .resolve::<OrderService>()
        .expect("application discovered service");
    assert!(Arc::ptr_eq(
        service.database(),
        &context
            .container()
            .resolve::<Database>()
            .expect("application discovered database")
    ));
    context.close().await.expect("context close");
}
