//! 链接期组件条目对象。
//!
//! 对应 Spring `org.springframework.context.index.processor.ItemMetadata`：
//! 一条索引记录(type 全限定名 + stereotypes Set)。vernal 在链接期通过
//! `#[linkme::distributed_slice]` 收集所有 `#[derive(Component)]` 的结构体。
//!
//! ## 与 Spring 的对应关系
//!
//! | Spring | Vernal |
//! |--------|--------|
//! | `ItemMetadata.type` | `LinkedComponentEntry::module_path` + `name` |
//! | `ItemMetadata.stereotypes` | `LinkedComponentEntry::stereotypes` |
//! | classpath 扫描 | linkme 分布式切片 |
//! | package name | `module_path!()` |

use vernal_beans::ComponentDefinition;

/// 一个由过程宏提交到链接期分布式切片的不可变组件定义条目。
///
/// 条目保存模块路径、稳定声明名、stereotype 集合和定义工厂，不保存组件实例、
/// Container、Scope 或运行时状态。每个应用 Context 安装 Index 时都会重新创建
/// [`ComponentDefinition`]，因此链接期发现不会成为跨 Context 的 Service
/// Locator，也不会共享 Singleton。
///
/// ## stereotype 字段语义
///
/// stereotype 字段（`&'static [&'static str]`）是该候选组件携带的所有
/// stereotype 标记，按字典序排列（与 Spring `ItemMetadata.stereotypes`
/// 语义一致）。Spring 中 stereotype 来源有三类：
/// - `@Indexed` 元注解传播
/// - `jakarta.*` 命名空间注解
/// - `package-info` 文件
///
/// vernal 的 stereotype 由 `#[component(stereotype = "...")]` 属性显式指定；
/// 若用户未标注，则默认为 `{"Component"}`，对应 Spring `@Component` 自带
/// `@Indexed` 元注解的语义。
///
/// ## 与 Spring 的对应关系
///
/// | Spring | Vernal |
/// |--------|--------|
/// | `@Component` 类 | `#[derive(Component)]` 结构体 |
/// | `@ComponentScan(basePackages)` | `LinkedComponentIndex::scan(base_packages)` |
/// | classpath 扫描 | linkme 分布式切片 |
/// | package name | `module_path!()` |
/// | `ItemMetadata.stereotypes` | `LinkedComponentEntry::stereotypes` |
#[derive(Clone, Debug)]
pub struct LinkedComponentEntry {
    /// 组件所在的 Rust 模块路径（对标 Java package name）
    module_path: &'static str,
    /// 包含模块路径和类型名的稳定声明名称
    name: &'static str,
    /// 该候选携带的 stereotype 集合（对标 `ItemMetadata.stereotypes`）。
    /// 按字典序排序的静态字符串数组。
    stereotypes: &'static [&'static str],
    /// 组件定义工厂函数
    definition: fn() -> ComponentDefinition,
}

impl LinkedComponentEntry {
    /// 创建一个静态链接期条目。
    ///
    /// `module_path` 和 `name` 由 `Component` 派生宏在编译期自动生成；
    /// `module_path` 使用 `module_path!()` 宏，`name` 使用
    /// `concat!(module_path, "::", stringify!(Type))`。`stereotypes` 由
    /// `#[component(stereotype = "...")]` 属性提供；若用户未标注，则默认为
    /// `&["Component"]`，对应 Spring `@Component` 自带 `@Indexed` 元注解的语义。
    ///
    /// 接受 `&'static [&'static str]` 数组形式，可在 `static` 上下文中调用
    /// （数组字面量是 const 表达式）。
    #[must_use]
    pub const fn new(
        module_path: &'static str,
        name: &'static str,
        stereotypes: &'static [&'static str],
        definition: fn() -> ComponentDefinition,
    ) -> Self {
        Self {
            module_path,
            name,
            stereotypes,
            definition,
        }
    }

    /// 创建一个默认 stereotype = `["Component"]` 的静态链接期条目。
    ///
    /// 这是无 `#[component(stereotype = "...")]` 属性时的默认行为，对标 Spring
    /// `@Component` 自带 `@Indexed` 元注解。
    #[must_use]
    pub const fn new_default(
        module_path: &'static str,
        name: &'static str,
        definition: fn() -> ComponentDefinition,
    ) -> Self {
        Self::new(module_path, name, &["Component"], definition)
    }

    /// 返回组件所在的 Rust 模块路径。
    ///
    /// 对标 Java 的 package name，用于 `LinkedComponentIndex::scan` 的 `base_packages` 过滤。
    #[must_use]
    pub const fn module_path(&self) -> &'static str {
        self.module_path
    }

    /// 返回包含模块路径和类型名的稳定声明名称。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// 返回该候选携带的 stereotype 数组（对标 `ItemMetadata.getStereotypes()`）。
    #[must_use]
    pub const fn stereotypes(&self) -> &[&'static str] {
        self.stereotypes
    }

    /// 检查该候选是否携带指定的 stereotype。
    #[must_use]
    pub fn has_stereotype(&self, stereotype: &str) -> bool {
        self.stereotypes.contains(&stereotype)
    }

    /// 迭代 stereotype 字符串。
    pub fn stereotypes_iter(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.stereotypes.iter().copied()
    }

    /// 为一次应用注册事务创建新的组件定义。
    #[must_use]
    pub fn component_definition(&self) -> ComponentDefinition {
        (self.definition)()
    }
}