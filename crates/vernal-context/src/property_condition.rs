//! 应用属性驱动的组件装配条件对象。

use std::fmt;

use vernal_core::BoxError;

use crate::{ApplicationEnvironment, ComponentCondition, ConditionError};

/// 根据属性是否存在或解析后的值是否相等决定条件模块是否装配。
///
/// 属性值只保存在不可变条件对象中用于构建期比较；`Debug`、启动报告和错误文本均
/// 不输出属性键与期望值。比较使用完成占位符展开后的字符串，属性来源失败时直接
/// 终止构建，不会因为低优先级来源存在同名键而静默降级。
#[derive(Clone)]
pub struct PropertyCondition {
    key: String,
    expected: Option<String>,
    match_if_missing: bool,
    negated: bool,
}

impl PropertyCondition {
    /// 创建“属性存在”条件。
    ///
    /// # Errors
    ///
    /// 属性键为空或包含空白字符时返回 [`ConditionError`]。
    pub fn present(key: impl Into<String>) -> Result<Self, ConditionError> {
        Self::new(key.into(), None, false, false)
    }

    /// 创建“属性不存在”条件。
    ///
    /// # Errors
    ///
    /// 属性键为空或包含空白字符时返回 [`ConditionError`]。
    pub fn missing(key: impl Into<String>) -> Result<Self, ConditionError> {
        Self::new(key.into(), None, false, true)
    }

    /// 创建“属性解析值等于期望值”条件。
    ///
    /// 缺少属性时结果为不命中。
    ///
    /// # Errors
    ///
    /// 属性键为空或包含空白字符时返回 [`ConditionError`]。
    pub fn having_value(
        key: impl Into<String>,
        expected: impl Into<String>,
    ) -> Result<Self, ConditionError> {
        Self::new(key.into(), Some(expected.into()), false, false)
    }

    /// 创建“属性缺失或解析值等于期望值”条件。
    ///
    /// 该语义对应显式的 `match_if_missing`，适合默认启用、配置后可关闭的能力。
    ///
    /// # Errors
    ///
    /// 属性键为空或包含空白字符时返回 [`ConditionError`]。
    pub fn having_value_or_missing(
        key: impl Into<String>,
        expected: impl Into<String>,
    ) -> Result<Self, ConditionError> {
        Self::new(key.into(), Some(expected.into()), true, false)
    }

    /// 创建“属性存在且解析值不等于期望值”条件。
    ///
    /// 缺少属性时结果仍为不命中，避免“未配置”被误认为显式反向选择。
    ///
    /// # Errors
    ///
    /// 属性键为空或包含空白字符时返回 [`ConditionError`]。
    pub fn not_having_value(
        key: impl Into<String>,
        expected: impl Into<String>,
    ) -> Result<Self, ConditionError> {
        Self::new(key.into(), Some(expected.into()), false, true)
    }

    /// 统一校验并创建属性条件。
    fn new(
        key: String,
        expected: Option<String>,
        match_if_missing: bool,
        negated: bool,
    ) -> Result<Self, ConditionError> {
        if key.is_empty() || key.chars().any(char::is_whitespace) {
            return Err(ConditionError::InvalidPropertyKey { key });
        }
        Ok(Self {
            key,
            expected,
            match_if_missing,
            negated,
        })
    }
}

impl ComponentCondition for PropertyCondition {
    fn name(&self) -> &'static str {
        match (&self.expected, self.negated, self.match_if_missing) {
            (None, true, _) => "property.missing",
            (None, false, _) => "property.present",
            (Some(_), true, _) => "property.not-equals",
            (Some(_), false, true) => "property.equals-or-missing",
            (Some(_), false, false) => "property.equals",
        }
    }

    fn matches(&self, environment: &ApplicationEnvironment) -> Result<bool, BoxError> {
        match &self.expected {
            None => {
                let present = environment.contains_property(&self.key)?;
                Ok(if self.negated { !present } else { present })
            }
            Some(expected) => {
                let Some(actual) = environment.property(&self.key)? else {
                    // “not equals”只对真实存在的属性成立；缺失属性不等同于显式配置
                    // 了另一值。正向条件则遵循调用方声明的 match-if-missing 语义。
                    return Ok(!self.negated && self.match_if_missing);
                };
                Ok(if self.negated {
                    actual != *expected
                } else {
                    actual == *expected
                })
            }
        }
    }
}

impl fmt::Debug for PropertyCondition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PropertyCondition")
            .field("condition", &self.name())
            .field("key", &"<redacted>")
            .field("expected", &self.expected.as_ref().map(|_| "<redacted>"))
            .finish_non_exhaustive()
    }
}
