//! 链接期组件索引对象。
//!
//! 对标 Spring `org.springframework.context.index.CandidateComponentsIndex`：
//! 从链接期分布式切片 [`crate::LINKED_COMPONENT_INDEX`] 中按模块路径过滤 +
//! stereotype 查询，生成确定性、只读的注册索引。
//!
//! ## 与 Spring 的对应关系
//!
//! | Spring | Vernal |
//! |--------|--------|
//! | `ClassPathBeanDefinitionScanner` | `LinkedComponentIndex::scan(base_packages)` |
//! | `@ComponentScan(basePackages)` | `LinkedComponentIndex::scan(base_packages)` |
//! | package 匹配 | `module_path` 前缀匹配 |
//! | `scan()` | `scan()` / `scan_all()` |
//! | `CandidateComponentsIndex#registerCandidateType` | `add_entry(entry)` |
//! | `CandidateComponentsIndexLoader#clearCache` | `clear_cache()` |

use std::collections::BTreeSet;

use vernal_beans::{DefinitionError, RegistryBuilder};

use crate::{LINKED_COMPONENT_INDEX, LinkedComponentEntry, LinkedComponentIndexError};

/// 一次模块路径扫描得到的确定性、只读组件注册索引。
///
/// 对应 Spring `org.springframework.context.index.CandidateComponentsIndex`：
///
/// Index 与具体 `RegistryBuilder` 分离：同一个扫描结果可以安装到多个应用，每次
/// 安装都创建新的 Definition，并由各自 Container 独立拥有 Singleton、Scope
/// 缓存和解析历史。索引按 `(module_path, name)` 排序，消除链接器和目标平台对
/// 分布式切片顺序的影响。
///
/// ## 内部存储
///
/// - `static_entries`：链接期 [`crate::LINKED_COMPONENT_INDEX`] 切片的引用副本
/// - `runtime_entries`：运行时通过 [`Self::add_entry`] 注入的额外条目
/// - `base_packages`：注册时的基包路径集合（用于 [`Self::has`] 判定）
/// - `complete`：true 表示索引来自链接期（对应 Spring `CandidateComponentsIndex#complete=true`），
///   false 表示纯运行时注册（对应 Spring 7.0 编程注册路径 `complete=false`）
#[derive(Clone, Debug)]
pub struct LinkedComponentIndex {
    /// 链接期静态条目（来自 `LINKED_COMPONENT_INDEX`）
    static_entries: Vec<&'static LinkedComponentEntry>,
    /// 运行时注入条目
    runtime_entries: Vec<LinkedComponentEntry>,
    /// 已注册的基包路径集合
    base_packages: BTreeSet<String>,
    /// 是否为完整索引（true = 链接期完整加载；false = 纯运行时注册）
    complete: bool,
}

impl LinkedComponentIndex {
    /// 按模块路径前缀扫描条目（对标 `@ComponentScan(basePackages = {...})`）。
    ///
    /// 从链接期分布式切片中筛选模块路径以任一 `base_packages` 前缀开头的条目。
    /// 这是 `@ComponentScan(basePackages = {"com.example.web"})` 的 Rust 等价物。
    ///
    /// # 参数
    /// - `base_packages`：模块路径前缀列表（如 `["my_app::web", "my_app::service"]`）
    ///
    /// # Errors
    ///
    /// 空选择或非法模块路径会返回 [`LinkedComponentIndexError`]。
    pub fn scan<I, S>(base_packages: I) -> Result<Self, LinkedComponentIndexError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let requested: BTreeSet<String> = base_packages
            .into_iter()
            .map(|pkg| pkg.as_ref().to_owned())
            .collect();

        if requested.is_empty() {
            return Err(LinkedComponentIndexError::EmptySelection);
        }
        for pkg in &requested {
            Self::validate_package(pkg)?;
        }

        // 按模块路径前缀过滤：组件的 module_path 以任一 base_package 开头
        let mut entries = LINKED_COMPONENT_INDEX
            .iter()
            .filter(|entry| {
                requested
                    .iter()
                    .any(|pkg| entry.module_path().starts_with(pkg.as_str()))
            })
            .collect::<Vec<_>>();

        // 按 (module_path, name) 排序，使输出顺序确定
        entries.sort_unstable_by(|left, right| {
            (left.module_path(), left.name()).cmp(&(right.module_path(), right.name()))
        });

        // 校验重复名称
        Self::validate_no_duplicates(&entries)?;

        Ok(Self {
            static_entries: entries,
            runtime_entries: Vec::new(),
            base_packages: requested,
            complete: true,
        })
    }

    /// 扫描所有已注册条目（对标 `@ComponentScan` 无参）。
    ///
    /// 收集链接期分布式切片中的所有条目，不做模块路径过滤。
    /// 适用于小型项目或测试场景。
    ///
    /// # Errors
    ///
    /// 重复稳定名称会返回 [`LinkedComponentIndexError::DuplicateEntry`]。
    pub fn scan_all() -> Result<Self, LinkedComponentIndexError> {
        let mut entries: Vec<_> = LINKED_COMPONENT_INDEX.iter().collect();

        // 按 (module_path, name) 排序
        entries.sort_unstable_by(|left, right| {
            (left.module_path(), left.name()).cmp(&(right.module_path(), right.name()))
        });

        Self::validate_no_duplicates(&entries)?;

        Ok(Self {
            static_entries: entries,
            runtime_entries: Vec::new(),
            base_packages: BTreeSet::new(),
            complete: true,
        })
    }

    /// 创建一个空的运行时索引（对标 Spring 7.0 `new CandidateComponentsIndex()`）。
    ///
    /// Spring 7.0 新增的编程注册入口：从空目录开始，后续通过 [`Self::add_entry`]
    /// 注入条目。返回的索引 `complete=false`，不会自动包含链接期条目。
    #[must_use]
    pub fn empty() -> Self {
        Self {
            static_entries: Vec::new(),
            runtime_entries: Vec::new(),
            base_packages: BTreeSet::new(),
            complete: false,
        }
    }

    /// 合并多个索引为一个原子批次。
    ///
    /// 将多次独立扫描的结果合并，用于组合不同模块路径的场景。
    ///
    /// # Errors
    ///
    /// 合并后发现重复注册会返回错误。
    pub fn merge(indices: &[&Self]) -> Result<Self, LinkedComponentIndexError> {
        let mut all: Vec<&'static LinkedComponentEntry> = Vec::new();
        for index in indices {
            all.extend(index.static_entries.iter().copied());
        }

        // 按 (module_path, name) 排序
        all.sort_unstable_by(|left, right| {
            (left.module_path(), left.name()).cmp(&(right.module_path(), right.name()))
        });

        Self::validate_no_duplicates(&all)?;

        let mut base_packages = BTreeSet::new();
        for index in indices {
            base_packages.extend(index.base_packages.iter().cloned());
        }

        Ok(Self {
            static_entries: all,
            runtime_entries: Vec::new(),
            base_packages,
            complete: true,
        })
    }

    /// 运行时注入一个条目（对标 Spring 7.0 `CandidateComponentsIndexLoader#addIndex`）。
    ///
    /// 运行时注入的条目会与链接期条目一起被 [`Self::iter_entries`] 返回，
    /// 但优先级最高（后注入的覆盖先注入的同名条目）。
    pub fn add_entry(&mut self, entry: LinkedComponentEntry) {
        self.runtime_entries.push(entry);
    }

    /// 清空所有运行时注入的条目（对标 Spring 7.0 `CandidateComponentsIndexLoader#clearCache`）。
    ///
    /// 注意：只清空 `runtime_entries`，链接期 `static_entries` 保持不变。
    pub fn clear_cache(&mut self) {
        self.runtime_entries.clear();
    }

    /// 注册基包路径集合（对标 Spring `CandidateComponentsIndex#registerScan`）。
    ///
    /// 当 `complete=false`（编程注册模式）时，应用通过此方法显式声明感兴趣的
    /// 基包；后续 [`Self::has`] 用作判定。
    ///
    /// # Errors
    ///
    /// 非法模块路径会返回 [`LinkedComponentIndexError::InvalidGroup`]。
    pub fn register_scan<I, S>(&mut self, base_packages: I) -> Result<(), LinkedComponentIndexError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for pkg in base_packages {
            let pkg = pkg.as_ref();
            Self::validate_package(pkg)?;
            self.base_packages.insert(pkg.to_owned());
        }
        Ok(())
    }

    /// 判断给定包路径是否已被索引扫描或注册（对标 `CandidateComponentsIndex#hasScannedPackage`）。
    ///
    /// Spring 行为：
    /// - `complete=true` 时（即索引来自文件加载）总是返回 `true`
    /// - `complete=false` 时（即编程注册）使用 `AntPathMatcher(".")` 做路径匹配
    ///
    /// vernal 使用 `module_path` 前缀匹配（对标 Spring 精确匹配分支）。
    #[must_use]
    pub fn has(&self, package_name: &str) -> bool {
        if self.complete {
            return true;
        }
        self.base_packages
            .iter()
            .any(|pkg| package_name.starts_with(pkg.as_str()) || pkg.starts_with(package_name))
    }

    /// 查询携带指定 stereotype 的所有候选 type（对标 `CandidateComponentsIndex#getCandidateTypes`）。
    ///
    /// Spring 用 `AntPathMatcher(".")` 在 `basePackage` 下做路径匹配；
    /// vernal 用 `module_path` 前缀匹配。
    ///
    /// # 参数
    /// - `base_package`：候选 type 所属的基包路径
    /// - `stereotype`：要查询的 stereotype 字符串
    ///
    /// # 返回
    ///
    /// 匹配的候选 type 全限定名集合。
    #[must_use]
    pub fn get(&self, base_package: &str, stereotype: &str) -> BTreeSet<&'static str> {
        let mut result = BTreeSet::new();

        for entry in &self.static_entries {
            if entry.module_path().starts_with(base_package) && entry.has_stereotype(stereotype) {
                result.insert(entry.name());
            }
        }

        for entry in &self.runtime_entries {
            let package = entry.module_path();
            let module_path_separator = "::";
            let pkg = match package.rfind(module_path_separator) {
                Some(idx) => &package[..idx],
                None => package,
            };
            if pkg.starts_with(base_package) && entry.has_stereotype(stereotype) {
                // runtime entries 名称是 owned String，转为 'static 等价物（直接借用）
                result.insert(entry.name());
            }
        }

        result
    }

    /// 返回已注册基包路径集合（对标 `CandidateComponentsIndex#getRegisteredScans`）。
    #[must_use]
    pub fn base_packages(&self) -> &BTreeSet<String> {
        &self.base_packages
    }

    /// 返回所有已注册 stereotype 集合（对标 `CandidateComponentsIndex#getRegisteredStereotypes`）。
    #[must_use]
    pub fn stereotypes(&self) -> BTreeSet<&'static str> {
        let mut all = BTreeSet::new();
        for entry in &self.static_entries {
            for stereotype in entry.stereotypes() {
                all.insert(*stereotype);
            }
        }
        for entry in &self.runtime_entries {
            for stereotype in entry.stereotypes() {
                all.insert(*stereotype);
            }
        }
        all
    }

    /// 判断索引是否完整加载（对标 `CandidateComponentsIndex#complete`）。
    ///
    /// `complete=true` 表示链接期完整加载；`false` 表示纯运行时注册。
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    /// 把索引中的全部定义作为一个原子批次安装到显式注册表。
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
        let mut definitions: Vec<vernal_beans::ComponentDefinition> =
            Vec::with_capacity(self.static_entries.len() + self.runtime_entries.len());
        for entry in &self.static_entries {
            definitions.push(entry.component_definition());
        }
        for entry in &self.runtime_entries {
            definitions.push(entry.component_definition());
        }
        registry.register_all(definitions.into_iter())
    }

    /// 为高层应用建造器或其他显式装配入口生成一批全新的组件定义。
    ///
    /// 调用方可把返回迭代器直接交给
    /// `VernalApplicationBuilder::register_all`。迭代器不暴露或缓存实例，每次
    /// 调用都从静态定义工厂重新创建完整批次。
    pub fn component_definitions(
        &self,
    ) -> impl Iterator<Item = vernal_beans::ComponentDefinition> + '_ {
        self.static_entries
            .iter()
            .map(|entry| entry.component_definition())
            .chain(
                self.runtime_entries
                    .iter()
                    .map(|entry| entry.component_definition()),
            )
    }

    /// 迭代全部条目（链接期 + 运行时）。
    ///
    /// 链接期条目返回 `&'static LinkedComponentEntry`，运行时条目返回 owned
    /// 临时引用（生命周期为 `'a`）。
    pub fn iter_entries(&self) -> impl Iterator<Item = EntryRef<'_>> {
        let static_iter = self.static_entries.iter().map(|e| EntryRef::Static(*e));
        let runtime_iter = self.runtime_entries.iter().map(EntryRef::Runtime);
        static_iter.chain(runtime_iter)
    }

    /// 返回确定性排序后的只读链接期条目。
    #[must_use]
    pub fn static_entries(&self) -> &[&'static LinkedComponentEntry] {
        &self.static_entries
    }

    /// 返回运行时注入条目。
    #[must_use]
    pub fn runtime_entries(&self) -> &[LinkedComponentEntry] {
        &self.runtime_entries
    }

    /// 返回本次索引包含的条目总数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.static_entries.len() + self.runtime_entries.len()
    }

    /// 返回本次索引是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.static_entries.is_empty() && self.runtime_entries.is_empty()
    }

    /// 校验模块路径前缀的合法性。
    fn validate_package(package: &str) -> Result<(), LinkedComponentIndexError> {
        if package.is_empty() || package.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(LinkedComponentIndexError::InvalidGroup {
                group: package.into(),
            });
        }
        Ok(())
    }

    /// 校验条目列表中无重复名称。
    fn validate_no_duplicates(
        entries: &[&'static LinkedComponentEntry],
    ) -> Result<(), LinkedComponentIndexError> {
        for pair in entries.windows(2) {
            let [left, right] = pair else {
                continue;
            };
            Self::validate_entry(left)?;
            if left.name() == right.name() {
                return Err(LinkedComponentIndexError::DuplicateEntry {
                    group: left.module_path().into(),
                    name: left.name().into(),
                });
            }
        }
        if let Some(last) = entries.last() {
            Self::validate_entry(last)?;
        }
        Ok(())
    }

    /// 校验条目的声明名合法性。
    fn validate_entry(entry: &LinkedComponentEntry) -> Result<(), LinkedComponentIndexError> {
        if entry.name().is_empty()
            || entry
                .name()
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err(LinkedComponentIndexError::InvalidEntry {
                group: entry.module_path().into(),
                name: entry.name().into(),
            });
        }
        Ok(())
    }
}

/// 索引条目引用枚举。
///
/// [`LinkedComponentIndex::iter_entries`] 返回类型，区分链接期（`'static`）与
/// 运行时（owned）两种条目来源。
#[derive(Clone, Debug)]
pub enum EntryRef<'a> {
    /// 链接期条目（`&'static LinkedComponentEntry`）。
    Static(&'static LinkedComponentEntry),
    /// 运行时注入条目（owned）。
    Runtime(&'a LinkedComponentEntry),
}

impl<'a> EntryRef<'a> {
    /// 返回条目 module_path。
    #[must_use]
    pub fn module_path(&self) -> &'static str {
        match self {
            Self::Static(e) => e.module_path(),
            Self::Runtime(e) => e.module_path(),
        }
    }

    /// 返回条目 name。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Static(e) => e.name(),
            Self::Runtime(e) => e.name(),
        }
    }

    /// 返回条目 stereotypes 数组切片。
    #[must_use]
    pub fn stereotypes(&self) -> &[&'static str] {
        match self {
            Self::Static(e) => e.stereotypes(),
            Self::Runtime(e) => e.stereotypes(),
        }
    }

    /// 判断是否携带指定 stereotype。
    #[must_use]
    pub fn has_stereotype(&self, stereotype: &str) -> bool {
        match self {
            Self::Static(e) => e.has_stereotype(stereotype),
            Self::Runtime(e) => e.has_stereotype(stereotype),
        }
    }
}
