//! 链接期组件注册项对象。
//!
//! 对标 Spring 的 `BeanDefinition` 元数据：每个 `#[derive(Component)]` 的结构体
//! 自动进入链接期分布式切片，携带模块路径和定义工厂。消费方通过
//! `ComponentScanModule` 按模块路径过滤，对标 `@ComponentScan(basePackages = ...)`。

use vernal_beans::ComponentDefinition;

/// 一个由过程宏提交到链接期分布式切片的不可变组件定义入口。
///
/// 注册项保存模块路径、稳定声明名和定义工厂，不保存组件实例、Container、
/// Scope 或运行时状态。每个应用 Context 安装 Catalog 时都会重新创建
/// [`ComponentDefinition`]，因此链接期发现不会成为跨 Context 的 Service
/// Locator，也不会共享 Singleton。
///
/// ## 与 Spring 的对应关系
///
/// | Spring | Vernal |
/// |--------|--------|
/// | `@Component` 类 | `#[derive(Component)]` 结构体 |
/// | `@ComponentScan(basePackages)` | `ComponentScanModule::base_packages()` |
/// | classpath 扫描 | linkme 分布式切片 |
/// | package name | `module_path!()` |
#[derive(Clone, Copy, Debug)]
pub struct LinkedComponentRegistration {
    /// 组件所在的 Rust 模块路径（对标 Java package name）
    module_path: &'static str,
    /// 包含模块路径和类型名的稳定声明名称
    name: &'static str,
    /// 组件定义工厂函数
    definition: fn() -> ComponentDefinition,
}

impl LinkedComponentRegistration {
    /// 创建一个静态链接期注册项。
    ///
    /// `module_path` 和 `name` 由 `Component` 派生宏在编译期自动生成；
    /// `module_path` 使用 `module_path!()` 宏，`name` 使用 `concat!(module_path, "::", stringify!(Type))`。
    #[must_use]
    pub const fn new(
        module_path: &'static str,
        name: &'static str,
        definition: fn() -> ComponentDefinition,
    ) -> Self {
        Self {
            module_path,
            name,
            definition,
        }
    }

    /// 返回组件所在的 Rust 模块路径。
    ///
    /// 对标 Java 的 package name，用于 `ComponentScanModule` 的 `base_packages` 过滤。
    #[must_use]
    pub const fn module_path(&self) -> &'static str {
        self.module_path
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
