//! 链接期组件注册项对象。

use vernal_ioc::ComponentDefinition;

/// 一个由过程宏提交到链接期分布式切片的不可变组件定义入口。
///
/// 注册项只保存静态分组、稳定声明名和定义工厂，不保存组件实例、Container、
/// Scope 或运行时状态。每个应用 Context 安装 Catalog 时都会重新创建
/// [`ComponentDefinition`]，因此链接期发现不会成为跨 Context 的 Service
/// Locator，也不会共享 Singleton。
#[derive(Clone, Copy, Debug)]
pub struct LinkedComponentRegistration {
    group: &'static str,
    name: &'static str,
    definition: fn() -> ComponentDefinition,
}

impl LinkedComponentRegistration {
    /// 创建一个静态链接期注册项。
    ///
    /// `group` 和 `name` 由 `Component` 派生宏在编译期生成并校验；手工提交注册项
    /// 时，Catalog 仍会在发现阶段执行相同的运行时防御性校验。
    #[must_use]
    pub const fn new(
        group: &'static str,
        name: &'static str,
        definition: fn() -> ComponentDefinition,
    ) -> Self {
        Self {
            group,
            name,
            definition,
        }
    }

    /// 返回应用显式选择的发现分组。
    #[must_use]
    pub const fn group(&self) -> &'static str {
        self.group
    }

    /// 返回包含模块路径和类型名的稳定声明名称。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// 为一次应用注册事务创建新的组件定义。
    #[must_use]
    pub fn component_definition(&self) -> ComponentDefinition {
        (self.definition)()
    }
}
