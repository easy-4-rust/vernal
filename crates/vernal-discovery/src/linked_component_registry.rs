//! 链接期组件注册表。
//!
//! 对标 Spring 的 classpath 扫描结果：所有 `#[derive(Component)]` 的结构体
//! 自动进入此分布式切片，无需显式标注 `discover` 属性。
//! 消费方通过 `ComponentScanModule` 按模块路径过滤，对标 `@ComponentScan`。

use crate::LinkedComponentRegistration;

/// 收集所有 `#[derive(Component)]` 的组件元数据。
///
/// 该切片是只读链接期事实，不是运行时容器。应用必须通过
/// [`crate::LinkedComponentCatalog::scan`] 选择模块路径范围，并把结果显式安装到
/// 自己的 `RegistryBuilder`；仅仅链接本 crate 不会创建或注册任何组件实例。
///
/// ## 与 Spring 的对应关系
///
/// - Spring：classpath 扫描所有 `@Component` 类
/// - Vernal：linkme 分布式切片收集所有 `#[derive(Component)]` 结构体
/// - Spring：`@ComponentScan(basePackages)` 过滤包路径
/// - Vernal：`ComponentScanModule::base_packages()` 过滤模块路径
#[linkme::distributed_slice]
pub static LINKED_COMPONENT_REGISTRATIONS: [LinkedComponentRegistration] = [..];
