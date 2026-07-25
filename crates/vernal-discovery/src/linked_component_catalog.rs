//! 链接期组件目录对象。

use std::collections::{BTreeSet, HashSet};

use vernal_ioc::{DefinitionError, RegistryBuilder};

use crate::{
    LINKED_COMPONENT_REGISTRATIONS, LinkedComponentCatalogError, LinkedComponentRegistration,
};

/// 一次显式分组选择得到的确定性、只读组件注册目录。
///
/// Catalog 与具体 `RegistryBuilder` 分离：同一个选择可以安装到多个应用，每次安装
/// 都创建新的 Definition，并由各自 Container 独立拥有 Singleton、Scope 缓存和
/// 解析历史。目录按 `(group, name)` 排序，消除链接器和目标平台对分布式切片顺序
/// 的影响。
#[derive(Clone, Debug)]
pub struct LinkedComponentCatalog {
    registrations: Vec<&'static LinkedComponentRegistration>,
}

impl LinkedComponentCatalog {
    /// 从链接期只读清单中选择一个或多个明确分组。
    ///
    /// 空选择、非法分组、未知分组以及同组重复稳定名称全部 fail-closed。没有
    /// `discover` 标记的普通 `Component` 不会进入目录。
    ///
    /// # Errors
    ///
    /// 返回 [`LinkedComponentCatalogError`] 描述选择或静态元数据错误。
    pub fn discover<I, S>(groups: I) -> Result<Self, LinkedComponentCatalogError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let requested = groups
            .into_iter()
            .map(|group| group.as_ref().to_owned())
            .collect::<BTreeSet<_>>();
        if requested.is_empty() {
            return Err(LinkedComponentCatalogError::EmptySelection);
        }
        for group in &requested {
            Self::validate_group(group)?;
        }

        // 只读取调用方明确选择的分组；依赖图中其他 crate 即使提交了注册项，也不会
        // 因为被链接而悄悄进入当前应用。
        let mut registrations = LINKED_COMPONENT_REGISTRATIONS
            .iter()
            .filter(|registration| requested.contains(registration.group()))
            .collect::<Vec<_>>();
        let matched_groups = registrations
            .iter()
            .map(|registration| registration.group())
            .collect::<HashSet<_>>();
        if let Some(missing) = requested
            .iter()
            .find(|group| !matched_groups.contains(group.as_str()))
        {
            return Err(LinkedComponentCatalogError::MissingGroup {
                group: missing.as_str().into(),
            });
        }

        registrations.sort_unstable_by(|left, right| {
            (left.group(), left.name()).cmp(&(right.group(), right.name()))
        });
        for pair in registrations.windows(2) {
            let [left, right] = pair else {
                continue;
            };
            Self::validate_registration(left)?;
            if left.group() == right.group() && left.name() == right.name() {
                return Err(LinkedComponentCatalogError::DuplicateRegistration {
                    group: left.group().into(),
                    name: left.name().into(),
                });
            }
        }
        if let Some(last) = registrations.last() {
            Self::validate_registration(last)?;
        }

        Ok(Self { registrations })
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
    pub fn component_definitions(&self) -> impl Iterator<Item = vernal_ioc::ComponentDefinition> {
        self.registrations
            .iter()
            .map(|registration| registration.component_definition())
    }

    /// 返回确定性排序后的只读注册项。
    #[must_use]
    pub fn registrations(&self) -> &[&'static LinkedComponentRegistration] {
        &self.registrations
    }

    /// 返回本次选择包含的注册项数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// 返回本次选择是否为空。
    ///
    /// 成功的 `discover` 至少匹配每个请求分组各一个注册项，因此正常情况下为
    /// `false`；该方法主要服务通用诊断和未来的过滤组合。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// 校验调用方分组或手工静态注册项的分组文本。
    fn validate_group(group: &str) -> Result<(), LinkedComponentCatalogError> {
        if group.is_empty()
            || group
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(LinkedComponentCatalogError::InvalidGroup {
                group: group.into(),
            });
        }
        Ok(())
    }

    /// 校验手工提交的稳定声明名，防止空白诊断和不确定身份进入目录。
    fn validate_registration(
        registration: &LinkedComponentRegistration,
    ) -> Result<(), LinkedComponentCatalogError> {
        Self::validate_group(registration.group())?;
        if registration.name().is_empty()
            || registration
                .name()
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(LinkedComponentCatalogError::InvalidRegistration {
                group: registration.group().into(),
                name: registration.name().into(),
            });
        }
        Ok(())
    }
}
