//! 手工重复链接期注册项。

use vernal_discovery::{LINKED_COMPONENT_REGISTRATIONS, LinkedComponentRegistration};
use vernal_beans::{Component, ComponentDefinition};

use super::Database;

/// 为重复元数据合同创建一个有效组件定义。
fn database_definition() -> ComponentDefinition {
    Database::definition()
}

#[vernal_discovery::linkme::distributed_slice(LINKED_COMPONENT_REGISTRATIONS)]
#[linkme(crate = vernal_discovery::linkme)]
static DUPLICATE_REGISTRATION_ONE: LinkedComponentRegistration = LinkedComponentRegistration::new(
    "tests::discovery_support",
    "tests::DuplicateRegistration",
    database_definition,
);

#[vernal_discovery::linkme::distributed_slice(LINKED_COMPONENT_REGISTRATIONS)]
#[linkme(crate = vernal_discovery::linkme)]
static DUPLICATE_REGISTRATION_TWO: LinkedComponentRegistration = LinkedComponentRegistration::new(
    "tests::discovery_support",
    "tests::DuplicateRegistration",
    database_definition,
);
