//! 链接期组件索引切片。
//!
//! 对应 Spring `META-INF/spring.components` 物理文件 + `CandidateComponentsIndexLoader` 类：
//! 所有 `#[derive(Component)]` 的结构体自动进入此分布式切片，无需显式标注。
//! 消费方通过 [`crate::LinkedComponentIndex`] 按模块路径 + stereotype 过滤，
//! 对标 `@ComponentScan(basePackages = ...)`。

use crate::LinkedComponentEntry;

/// 链接期组件索引切片常量。
///
/// 对标 Spring `CandidateComponentsIndexLoader#COMPONENTS_RESOURCE_LOCATION = "META-INF/spring.components"`：
/// - Spring：classpath 上的物理 Properties 文件
/// - Vernal：链接期 linkme 分布式切片（`&'static`），是物理文件的 1:1 Rust 等价物
///
/// 该切片是只读链接期事实，不是运行时容器。应用必须通过
/// [`crate::LinkedComponentIndex::scan`] 选择模块路径范围，并把结果显式安装到
/// 自己的 `RegistryBuilder`；仅仅链接本 crate 不会创建或注册任何组件实例。
///
/// ## 与 Spring 的对应关系
///
/// - Spring：classpath 扫描所有 `@Component` 类（通过 `spring-context-indexer` 生成的索引文件）
/// - Vernal：linkme 分布式切片收集所有 `#[derive(Component)]` 结构体
/// - Spring：`@ComponentScan(basePackages)` 过滤包路径
/// - Vernal：`LinkedComponentIndex::scan(base_packages)` 过滤模块路径
///
/// ## 链接期注入机制
///
/// 每个 `#[derive(Component)]` 的结构体由 `vernal-macros` 自动生成一个
/// `#[linkme::distributed_slice(LINKED_COMPONENT_INDEX)]` 的静态条目，链接器
/// 将所有 `__VERNAL_LINKED_COMPONENT_ENTRY_<Type>` 静态项拼成这个连续切片。
/// 启动时 [`LINKED_COMPONENT_INDEX`] 就是"所有已注册组件"的索引。
#[linkme::distributed_slice]
pub static LINKED_COMPONENT_INDEX: [LinkedComponentEntry] = [..];