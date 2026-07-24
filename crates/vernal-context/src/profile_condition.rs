//! Profile 驱动的组件装配条件对象。

use std::{collections::BTreeSet, fmt, sync::Arc};

use vernal_core::BoxError;

use crate::{ApplicationEnvironment, ComponentCondition, ConditionError};

/// 根据当前有效 Profile 集合决定条件模块是否装配。
///
/// Profile 在构造时完成排序、去重和合法性校验。`any` 表示任一候选生效即可，
/// `all` 表示全部候选同时生效，`none` 表示所有候选都未生效。判断使用
/// `ApplicationEnvironment` 的 Active-over-Default 语义。
#[derive(Clone)]
pub struct ProfileCondition {
    profiles: Arc<[String]>,
    require_all: bool,
    negated: bool,
}

impl ProfileCondition {
    /// 创建“任一 Profile 生效”条件。
    ///
    /// # Errors
    ///
    /// 候选为空，或任一名称为空、包含空白字符时返回 [`ConditionError`]。
    pub fn any<I, S>(profiles: I) -> Result<Self, ConditionError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(profiles, false, false)
    }

    /// 创建“全部 Profile 生效”条件。
    ///
    /// # Errors
    ///
    /// 候选为空，或任一名称为空、包含空白字符时返回 [`ConditionError`]。
    pub fn all<I, S>(profiles: I) -> Result<Self, ConditionError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(profiles, true, false)
    }

    /// 创建“所有 Profile 均未生效”条件。
    ///
    /// # Errors
    ///
    /// 候选为空，或任一名称为空、包含空白字符时返回 [`ConditionError`]。
    pub fn none<I, S>(profiles: I) -> Result<Self, ConditionError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(profiles, false, true)
    }

    /// 统一校验并冻结 Profile 集合。
    fn new<I, S>(profiles: I, require_all: bool, negated: bool) -> Result<Self, ConditionError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let candidates = profiles;
        let mut profiles: BTreeSet<String> = BTreeSet::new();
        for profile in candidates {
            let profile: String = profile.into();
            if profile.is_empty() || profile.chars().any(char::is_whitespace) {
                return Err(ConditionError::InvalidProfile { profile });
            }
            profiles.insert(profile);
        }
        if profiles.is_empty() {
            return Err(ConditionError::EmptyProfileSet);
        }
        Ok(Self {
            profiles: Arc::from(profiles.into_iter().collect::<Vec<_>>()),
            require_all,
            negated,
        })
    }
}

impl ComponentCondition for ProfileCondition {
    fn name(&self) -> &'static str {
        match (self.require_all, self.negated) {
            (_, true) => "profile.none",
            (true, false) => "profile.all",
            (false, false) => "profile.any",
        }
    }

    fn matches(&self, environment: &ApplicationEnvironment) -> Result<bool, BoxError> {
        let mut active_count = 0;
        for profile in self.profiles.iter() {
            if environment.is_profile_active(profile)? {
                active_count += 1;
            }
        }
        let matched = if self.require_all {
            active_count == self.profiles.len()
        } else {
            active_count > 0
        };
        Ok(if self.negated { !matched } else { matched })
    }
}

impl fmt::Debug for ProfileCondition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProfileCondition")
            .field("profile_count", &self.profiles.len())
            .field("condition", &self.name())
            .finish_non_exhaustive()
    }
}
