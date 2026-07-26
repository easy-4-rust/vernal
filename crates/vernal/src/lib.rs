#![forbid(unsafe_code)]
#![doc = "Vernal Framework 的统一门面 crate。"]

/// `AOP` 内核命名空间。
pub use vernal_aop as aop;
/// `Beans` 内核命名空间（对标 Spring Beans）。
pub use vernal_beans as beans;
/// `IoC` 内核命名空间（兼容别名）。
pub use vernal_beans as ioc;
/// 应用上下文命名空间。
pub use vernal_context as context;
/// 框架共享合同命名空间。
pub use vernal_core as core;
/// 统一错误体系命名空间。
pub use vernal_core::error;
/// 过程宏命名空间。
pub use vernal_macros as macros;

/// 返回当前框架成熟度状态。
#[must_use]
pub const fn project_status() -> &'static str {
    vernal_core::PROJECT_STATUS
}
