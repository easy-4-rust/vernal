//! 应用环境建造器对象。

use std::{collections::BTreeSet, sync::Arc};

use crate::{ApplicationEnvironment, EnvironmentError, PropertySource};

/// 在 Context 构建阶段收集 `PropertySource` 与 Profile 的可变建造器。
///
/// `PropertySource` 顺序是显式合同：索引越小优先级越高。调用方使用
/// [`Self::add_first`] 或 [`Self::add_last`] 表达覆盖关系，避免依赖隐含的文件
/// 加载顺序。构建结果不可变，可安全作为普通 `IoC` Singleton 共享。
#[derive(Clone)]
pub struct ApplicationEnvironmentBuilder {
    sources: Vec<Arc<dyn PropertySource>>,
    active_profiles: BTreeSet<String>,
    default_profiles: BTreeSet<String>,
}

impl ApplicationEnvironmentBuilder {
    /// 创建仅包含 `default` 默认 Profile 的空环境建造器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 在最高优先级位置加入 `PropertySource`。
    ///
    /// # Errors
    ///
    /// 来源名称非法或已存在同名来源时返回 [`EnvironmentError`]。
    pub fn add_first(
        &mut self,
        source: Arc<dyn PropertySource>,
    ) -> Result<&mut Self, EnvironmentError> {
        self.validate_source(&source)?;
        self.sources.insert(0, source);
        Ok(self)
    }

    /// 在最低优先级位置加入 `PropertySource`。
    ///
    /// # Errors
    ///
    /// 来源名称非法或已存在同名来源时返回 [`EnvironmentError`]。
    pub fn add_last(
        &mut self,
        source: Arc<dyn PropertySource>,
    ) -> Result<&mut Self, EnvironmentError> {
        self.validate_source(&source)?;
        self.sources.push(source);
        Ok(self)
    }

    /// 启用一个应用 Profile。
    ///
    /// Active Profile 非空时替代默认 Profile 集合。重复添加会被确定性去重。
    ///
    /// # Errors
    ///
    /// Profile 为空或包含空白字符时返回 [`EnvironmentError`]。
    pub fn active_profile(
        &mut self,
        profile: impl Into<String>,
    ) -> Result<&mut Self, EnvironmentError> {
        let profile = Self::validated_profile(profile.into())?;
        self.active_profiles.insert(profile);
        Ok(self)
    }

    /// 添加一个在没有 Active Profile 时生效的默认 Profile。
    ///
    /// # Errors
    ///
    /// Profile 为空或包含空白字符时返回 [`EnvironmentError`]。
    pub fn default_profile(
        &mut self,
        profile: impl Into<String>,
    ) -> Result<&mut Self, EnvironmentError> {
        let profile = Self::validated_profile(profile.into())?;
        self.default_profiles.insert(profile);
        Ok(self)
    }

    /// 清除全部 Active Profile，让默认 Profile 重新生效。
    pub fn clear_active_profiles(&mut self) -> &mut Self {
        self.active_profiles.clear();
        self
    }

    /// 清除包括内建 `default` 在内的全部默认 Profile。
    pub fn clear_default_profiles(&mut self) -> &mut Self {
        self.default_profiles.clear();
        self
    }

    /// 冻结来源顺序与 Profile 集合，生成可共享的应用环境。
    #[must_use]
    pub fn build(self) -> ApplicationEnvironment {
        ApplicationEnvironment::new(
            self.sources,
            self.active_profiles.into_iter().collect(),
            self.default_profiles.into_iter().collect(),
        )
    }

    /// 校验来源名称和同名冲突，但不读取任何属性值。
    fn validate_source(&self, source: &Arc<dyn PropertySource>) -> Result<(), EnvironmentError> {
        let name = source.name();
        if name.trim().is_empty() || name.chars().any(char::is_control) {
            return Err(EnvironmentError::InvalidPropertySourceName {
                name: name.to_owned(),
            });
        }
        if self.sources.iter().any(|existing| existing.name() == name) {
            return Err(EnvironmentError::DuplicatePropertySource {
                name: name.to_owned(),
            });
        }
        Ok(())
    }

    /// 校验 Profile，避免同一个逻辑名称出现不可见空白差异。
    fn validated_profile(profile: String) -> Result<String, EnvironmentError> {
        if profile.is_empty() || profile.chars().any(char::is_whitespace) {
            return Err(EnvironmentError::InvalidProfile { profile });
        }
        Ok(profile)
    }
}

impl Default for ApplicationEnvironmentBuilder {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            active_profiles: BTreeSet::new(),
            default_profiles: BTreeSet::from([String::from("default")]),
        }
    }
}
