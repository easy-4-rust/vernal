//! 可选链接期组件发现运行时合同测试。
//!
//! 把 fixture 直接放在本文件,确保 cargo test 编译测试时 fixture 也被编译进
//! 测试 binary（cargo test 不递归子目录，mod index_support 在 tests/ 子目录
//! 时不会被自动链接）。

use std::sync::Arc;

use vernal_beans::{Component, ComponentDefinition, RegistryBuilder};
use vernal_context::VernalApplicationBuilder;
use vernal_context_indexer::{LinkedComponentIndex, LinkedComponentIndexError};

// =============================================================================
// 测试 fixture（直接放在测试文件以确保链接到 binary）
// =============================================================================

/// 测试数据库组件对象。
#[derive(vernal_macros::Component)]
pub struct TestDatabase;

/// 测试订单服务组件对象。
#[derive(vernal_macros::Component)]
pub struct TestOrderService {
    database: Arc<TestDatabase>,
}

impl TestOrderService {
    /// 返回当前 Context 注入的数据库 Singleton。
    #[must_use]
    pub fn database(&self) -> &Arc<TestDatabase> {
        &self.database
    }
}

/// 测试审计任务组件对象。
#[derive(vernal_macros::Component)]
pub struct TestAuditWorker;

/// 显式注册测试组件对象。
pub struct ManualComponent;

// 重复链接期条目 fixture（触发 DuplicateEntry 错误）
// 放在另一个 module_path,只有当用户扫描 ["duplicate_test_group"] 时才会被发现,
// 这样正常扫描不会受影响。
fn duplicate_test_definition() -> vernal_beans::ComponentDefinition {
    TestDatabase::definition()
}

#[vernal_context_indexer::linkme::distributed_slice(vernal_context_indexer::LINKED_COMPONENT_INDEX)]
#[linkme(crate = vernal_context_indexer::linkme)]
static DUPLICATE_TEST_A: vernal_context_indexer::LinkedComponentEntry =
    vernal_context_indexer::LinkedComponentEntry::new(
        "duplicate_test_group",
        "duplicate_test_group::TestDuplicate",
        &["Component"],
        duplicate_test_definition,
    );

#[vernal_context_indexer::linkme::distributed_slice(vernal_context_indexer::LINKED_COMPONENT_INDEX)]
#[linkme(crate = vernal_context_indexer::linkme)]
static DUPLICATE_TEST_B: vernal_context_indexer::LinkedComponentEntry =
    vernal_context_indexer::LinkedComponentEntry::new(
        "duplicate_test_group",
        "duplicate_test_group::TestDuplicate",
        &["Component"],
        duplicate_test_definition,
    );

#[test]
fn explicit_group_selection_is_deterministic_and_excludes_other_groups() {
    // fixture 的 module_path 在测试 binary 中是 `linked_component_index_runtime`
    let catalog = LinkedComponentIndex::scan(["linked_component_index_runtime"])
        .expect("core discovery catalog");
    let names = catalog
        .static_entries()
        .iter()
        .map(|entry| entry.name())
        .collect::<Vec<_>>();

    assert_eq!(catalog.len(), 3, "got names: {names:?}");
    assert!(!catalog.is_empty());
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));

    let mut registry = RegistryBuilder::new();
    catalog
        .install(&mut registry)
        .expect("atomic catalog install");
    let container = registry.build().expect("discovered graph").container();
    let service = container
        .resolve::<TestOrderService>()
        .expect("discovered order service");
    assert!(Arc::ptr_eq(
        service.database(),
        &container
            .resolve::<TestDatabase>()
            .expect("discovered database")
    ));
}

#[test]
fn one_catalog_installs_into_isolated_containers_without_sharing_singletons() {
    let catalog = LinkedComponentIndex::scan(["linked_component_index_runtime"])
        .expect("core discovery catalog");
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
        .resolve::<TestOrderService>()
        .expect("first order service");
    let second_service = second
        .resolve::<TestOrderService>()
        .expect("second order service");
    assert!(!Arc::ptr_eq(&first_service, &second_service));
    assert!(!Arc::ptr_eq(
        first_service.database(),
        second_service.database()
    ));
}

#[test]
fn catalog_install_is_atomic_when_an_existing_definition_conflicts() {
    let catalog = LinkedComponentIndex::scan(["linked_component_index_runtime"])
        .expect("core discovery catalog");
    let mut registry = RegistryBuilder::new();
    registry
        .register(TestDatabase::definition())
        .expect("existing database definition");
    let before = registry.len();

    catalog
        .install(&mut registry)
        .expect_err("duplicate discovered database must reject complete batch");
    assert_eq!(registry.len(), before);
}

#[test]
fn empty_and_duplicate_groups_fail_closed() {
    // 空基包集合应立即拒绝（fail-closed）
    assert_eq!(
        LinkedComponentIndex::scan::<[&str; 0], &str>([]).expect_err("empty group selection"),
        LinkedComponentIndexError::EmptySelection
    );

    // 重复链接期条目应触发 DuplicateEntry（扫描 ["duplicate_test_group"] 时）
    assert!(matches!(
        LinkedComponentIndex::scan(["duplicate_test_group"]).expect_err("duplicate entry"),
        LinkedComponentIndexError::DuplicateEntry { .. }
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
    let catalog = LinkedComponentIndex::scan(["linked_component_index_runtime"])
        .expect("core discovery catalog");
    let mut builder = VernalApplicationBuilder::current().expect("Tokio application builder");
    builder
        .register_all(catalog.component_definitions())
        .expect("discovered application definitions");
    let context = builder.build().expect("application context");
    context.refresh().await.expect("context refresh");
    context.start().await.expect("context start");

    let service = context
        .container()
        .resolve::<TestOrderService>()
        .expect("application discovered service");
    assert!(Arc::ptr_eq(
        service.database(),
        &context
            .container()
            .resolve::<TestDatabase>()
            .expect("application discovered database")
    ));
    context.close().await.expect("context close");
}