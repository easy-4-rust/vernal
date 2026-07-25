//! 链接期组件注册表。

use crate::LinkedComponentRegistration;

/// 收集所有显式声明 `#[component(discover = "...")]` 的组件元数据。
///
/// 该切片是只读链接期事实，不是运行时容器。应用必须通过
/// [`crate::LinkedComponentCatalog::discover`] 选择分组，并把结果显式安装到自己
/// 的 `RegistryBuilder`；仅仅链接本 crate 不会创建或注册任何组件实例。
#[linkme::distributed_slice]
pub static LINKED_COMPONENT_REGISTRATIONS: [LinkedComponentRegistration] = [..];
