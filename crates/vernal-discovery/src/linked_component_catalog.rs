//! 链接期组件目录对象。
//!
//! 对标 Spring 的 `ClassPathScanningCandidateComponentProvider`：
//! 从链接期分布式切片中按模块路径过滤组件，生成确定性、只读的注册目录。
//!
//! ## 与 Spring 的对应关系
//!
//! | Spring | Vernal |
//! |--------|--------|
//! | `ClassPathBeanDefinitionScanner` | `LinkedComponentCatalog` |
//! | `@ComponentScan(basePackages)` | `LinkedComponentCatalog::scan(base_packages)` |
//! | package 匹配 | `module_path` 前缀匹配 |
//! | `scan()` | `scan()` / `scan_all()` |

use std::collections::BTreeSet;

use vernal_beans::{DefinitionError, RegistryBuilder};

use crate::{
    LINKED_COMPONENT_REGISTRATIONS, LinkedComponentCatalogError, LinkedComponentRegistration,
};

/// 一次模块路径扫描得到的确定性、只读组件注册目录。
///
/// Catalog 与具体 `RegistryBuilder` 分离：同一个扫描结果可以安装到多个应用，每次安装
/// 都创建新的 Definition，并由各自 Container 独立拥有 Singleton、Scope 缓存和
/// 解析历史。目录按 `(module_path, name)` 排序，消除链接器和目标平台对分布式切片顺序
/// 的影响。
///
/// ## 使用方式（对标 @ComponentScan）
///
/// ```rust,ignore
/// use vernal_discovery::LinkedComponentCatalog;
///
/// // 对标 @ComponentScan(basePackages = {"my_app::web", "my_app::service"})
/// let catalog = LinkedComponentCatalog::scan(["my_app::web", "my_app::service"])?;
/// catalog.install(&mut registry)?;
///
/// // 对标 @ComponentScan（扫描所有已注册组件）
/// let catalog = LinkedComponentCatalog::scan_all()?;
/// catalog.install(&mut registry)?;
/// ```
#[derive(Clone, Debug)]
pub struct LinkedComponentCatalog {
    registrations: Vec<&'static LinkedComponentRegistration>,
}

impl LinkedComponentCatalog {
    /// 按模块路径前缀扫描组件（对标 `@ComponentScan(basePackages = {...})`）。
    ///
    /// 从链接期分布式切片中筛选模块路径以任一 `base_packages` 前缀开头的组件。
    /// 这是 `@ComponentScan(basePackages = {"com.example.web"})` 的 Rust 等价物。
    ///
    /// # 参数
    /// - `base_packages`：模块路径前缀列表（如 `["my_app::web", "my_app::service"]`）
    ///
    /// # Errors
    ///
    /// 空选择或非法模块路径会返回 [`LinkedComponentCatalogError`]。
    pub fn scan<I, S>(base_packages: I) -> Result<Self, LinkedComponentCatalogError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let requested: BTreeSet<String> = base_packages
            .into_iter()
            .map(|pkg| pkg.as_ref().to_owned())
            .collect();

        if requested.is_empty() {
            return Err(LinkedComponentCatalogError::EmptySelection);
        }
        for pkg in &requested {
            Self::validate_package(pkg)?;
        }

        // 按模块路径前缀过滤：组件的 module_path 以任一 base_package 开头
        let mut registrations = LINKED_COMPONENT_REGISTRATIONS
            .iter()
            .filter(|reg| {
                requested
                    .iter()
                    .any(|pkg| reg.module_path().starts_with(pkg.as_str()))
            })
            .collect::<Vec<_>>();

        // 按 (module_path, name) 排序，使输出顺序确定
        registrations.sort_unstable_by(|left, right| {
            (left.module_path(), left.name()).cmp(&(right.module_path(), right.name()))
        });

        // 校验重复名称
        Self::validate_no_duplicates(&registrations)?;

        Ok(Self { registrations })
    }

    /// 扫描所有已注册组件（对标 `@ComponentScan` 无参）。
    ///
    /// 收集链接期分布式切片中的所有组件，不做模块路径过滤。
    /// 适用于小型项目或测试场景。
    ///
    /// # Errors
    ///
    /// 重复稳定名称会返回 [`LinkedComponentCatalogError::DuplicateRegistration`]。
    pub fn scan_all() -> Result<Self, LinkedComponentCatalogError> {
        let mut registrations: Vec<_> = LINKED_COMPONENT_REGISTRATIONS.iter().collect();

        // 按 (module_path, name) 排序
        registrations.sort_unstable_by(|left, right| {
            (left.module_path(), left.name()).cmp(&(right.module_path(), right.name()))
        });

        // 校验重复名称
        Self::validate_no_duplicates(&registrations)?;

        Ok(Self { registrations })
    }

    /// 合并多个目录为一个原子批次。
    ///
    /// 将多次独立扫描的结果合并，用于组合不同模块路径的场景。
    ///
    /// # Errors
    ///
    /// 合并后发现重复注册会返回错误。
    pub fn merge(catalogs: &[&Self]) -> Result<Self, LinkedComponentCatalogError> {
        let mut all: Vec<&'static LinkedComponentRegistration> = Vec::new();
        for catalog in catalogs {
            all.extend(catalog.registrations.iter().copied());
        }

        // 按 (module_path, name) 排序
        all.sort_unstable_by(|left, right| {
            (left.module_path(), left.name()).cmp(&(right.module_path(), right.name()))
        });

        Self::validate_no_duplicates(&all)?;

        Ok(Self { registrations: all })
    }

    /// 把目录中的全部定义作为一个原子批次安装到显式注册表。
    ///
    /// 任何定义与现有注册表或批次内其他定义冲突时，底层 `register_all` 会在修改
    /// Registry 前返回错误，不留下部分发现结果。
    ///
    /// # Errors
    ///
    /// 返回 [`DefinitionError`] 表示组件身份冲突。
    pub fn install<'registry>(
        &self,
        registry: &'registry mut RegistryBuilder,
    ) -> Result<&'registry mut RegistryBuilder, DefinitionError> {
        registry.register_all(
            self.registrations
                .iter()
                .map(|registration| registration.component_definition()),
        )
    }

    /// 为高层应用建造器或其他显式装配入口生成一批全新的组件定义。
    ///
    /// 调用方可把返回迭代器直接交给
    /// `VernalApplicationBuilder::register_all`。迭代器不暴露或缓存实例，每次
    /// 调用都从静态定义工厂重新创建完整批次。
    pub fn component_definitions(&self) -> impl Iterator<Item = vernal_beans::ComponentDefinition> {
        self.registrations
            .iter()
            .map(|registration| registration.component_definition())
    }

    /// 返回确定性排序后的只读注册项。
    #[must_use]
    pub fn registrations(&self) -> &[&'static LinkedComponentRegistration] {
        &self.registrations
    }

    /// 返回本次扫描包含的注册项数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// 返回本次扫描是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// 校验模块路径前缀的合法性。
    fn validate_package(package: &str) -> Result<(), LinkedComponentCatalogError> {
        if package.is_empty() || package.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(LinkedComponentCatalogError::InvalidGroup {
                group: package.into(),
            });
        }
        Ok(())
    }

    /// 校验注册项列表中无重复名称。
    fn validate_no_duplicates(
        registrations: &[&'static LinkedComponentRegistration],
    ) -> Result<(), LinkedComponentCatalogError> {
        for pair in registrations.windows(2) {
            let [left, right] = pair else {
                continue;
            };
            Self::validate_registration(left)?;
            if left.name() == right.name() {
                return Err(LinkedComponentCatalogError::DuplicateRegistration {
                    group: left.module_path().into(),
                    name: left.name().into(),
                });
            }
        }
        if let Some(last) = registrations.last() {
            Self::validate_registration(last)?;
        }
        Ok(())
    }

    /// 校验注册项的声明名合法性。
    fn validate_registration(
        registration: &LinkedComponentRegistration,
    ) -> Result<(), LinkedComponentCatalogError> {
        if registration.name().is_empty()
            || registration
                .name()
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(LinkedComponentCatalogError::InvalidRegistration {
                group: registration.module_path().into(),
                name: registration.name().into(),
            });
        }
        Ok(())
    }
}
