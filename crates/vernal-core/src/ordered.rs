//! 初始化排序常量。
//!
//! 对标 tx_di 的 `init_sort` 模式：同一拓扑深度的组件按此值升序排列。
//! 提供三个标准层级常量，供 `#[component(init_sort = N)]` 使用。
//!
//! # 设计来源
//!
//! tx_di 使用 `init_sort = i32::MIN` 让日志组件最先初始化，`init_sort = i32::MAX` 让
//! Web 服务器最后启动。vernal 将这个模式标准化为三个常量。

/// 基础设施层排序值（日志、配置中心等）。
///
/// 对标 tx_di 的 `init_sort = i32::MIN`。
/// 使用此值的组件会在所有业务组件之前初始化。
pub const INIT_SORT_INFRASTRUCTURE: i32 = i32::MIN + 1;

/// 业务组件排序值（默认）。
///
/// 大多数业务组件使用此值。
pub const INIT_SORT_BUSINESS: i32 = 0;

/// 应用层排序值（Web 服务器、后台任务等）。
///
/// 使用此值的组件会在所有业务组件之后初始化。
pub const INIT_SORT_APPLICATION: i32 = i32::MAX - 1;

/// 默认排序值（最晚初始化）。
///
/// 当未指定 `init_order` 时使用的默认值。
pub const INIT_SORT_DEFAULT: i32 = i32::MAX;
