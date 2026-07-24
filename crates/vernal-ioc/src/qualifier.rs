//! 组件限定符对象。

use std::{fmt, sync::Arc};

use crate::DefinitionError;

/// 用于区分相同 Rust 类型的不同组件定义。
///
/// 限定符属于组件稳定标识的一部分，因此构建后不可变。内部使用 `Arc<str>`，
/// 让注册表、依赖图和运行时解析路径可以低成本克隆同一份文本。
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Qualifier(Arc<str>);

impl Qualifier {
    /// 创建一个经过校验的限定符。
    ///
    /// 限定符不能为空，也不能携带首尾空白。该约束可以避免配置、宏生成代码
    /// 与手工注册在视觉上相同、实际却无法匹配的问题。
    ///
    /// # Errors
    ///
    /// 当文本为空或含有首尾空白时返回 [`DefinitionError::InvalidQualifier`]。
    pub fn new(value: impl Into<String>) -> Result<Self, DefinitionError> {
        let value = value.into();
        if value.is_empty() || value.trim() != value {
            return Err(DefinitionError::InvalidQualifier { value });
        }
        Ok(Self(Arc::from(value)))
    }

    /// 返回限定符的字符串视图。
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Qualifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
