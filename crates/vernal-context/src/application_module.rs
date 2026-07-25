//! 应用模块装配契约。

use vernal_core::BoxError;

use crate::ApplicationModuleRegistrar;

/// 由应用或消费方 Bridge 实现的显式装配模块。
///
/// 模块通过 [`ApplicationModuleRegistrar`] 声明组件、Trait Binding、生命周期、
/// AOP Advisor、Operation、属性来源和条件组件模块。`VernalApplicationBuilder`
/// 会先在隔离的 Registrar 中执行完整配置，再统一校验并提交；配置或提交失败不会
/// 留下半个模块。
///
/// 该契约不使用 classpath 扫描、链接段自动注册或进程级全局表。Hutool-Rust、
/// Sa-Token-Rust、Ddd4r 以及普通应用可以在自己的 crate 中实现模块，同时保持
/// 依赖方向始终指向 Vernal。
pub trait ApplicationModule: Sized {
    /// 返回稳定、无敏感数据的模块诊断名称。
    ///
    /// 默认值是完整 Rust 类型名。实现者可覆盖为 `sa-token.security` 一类静态
    /// 名称；同一应用中名称必须唯一。
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// 向隔离 Registrar 声明模块贡献。
    ///
    /// 返回错误时 Registrar 会被整体丢弃，应用建造器保持调用前状态。错误正文
    /// 不进入普通 `Display` 或 `Debug`；需要根因时调用方可显式遍历错误链。
    ///
    /// # Errors
    ///
    /// 模块自身配置、外部输入转换或业务前置校验失败时返回类型擦除错误。
    fn configure(self, registrar: &mut ApplicationModuleRegistrar) -> Result<(), BoxError>;
}
