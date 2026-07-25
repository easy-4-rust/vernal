//! 类型安全配置对象绑定错误。

use std::{any::type_name, error::Error, fmt};

use crate::EnvironmentError;

/// 描述一个配置结构体字段绑定失败的位置和脱敏原因。
///
/// 错误只保留配置类型、Rust 字段名和完整属性键，不保存原始属性值。底层
/// [`EnvironmentError`] 同样遵守该边界，因此配置错误可以安全进入启动日志；
/// 调用方仍可通过 [`Error::source`] 检查结构化原因。
pub struct ConfigurationPropertiesError {
    configuration_type: &'static str,
    field: &'static str,
    property_key: String,
    source: Box<dyn Error + Send + Sync + 'static>,
}

impl ConfigurationPropertiesError {
    /// 包装 Environment 在单个字段查找、占位符展开或类型转换阶段的失败。
    #[must_use]
    pub fn environment<T>(
        field: &'static str,
        property_key: String,
        source: EnvironmentError,
    ) -> Self {
        Self {
            configuration_type: type_name::<T>(),
            field,
            property_key,
            source: Box::new(source),
        }
    }

    /// 包装嵌套配置对象的绑定失败，同时保留外层字段边界。
    #[must_use]
    pub fn nested<T>(field: &'static str, property_key: String, source: Self) -> Self {
        Self {
            configuration_type: type_name::<T>(),
            field,
            property_key,
            source: Box::new(source),
        }
    }

    /// 返回发生绑定失败的配置对象 Rust 类型名。
    #[must_use]
    pub const fn configuration_type(&self) -> &'static str {
        self.configuration_type
    }

    /// 返回发生绑定失败的 Rust 字段名。
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// 返回对应的完整属性键。
    #[must_use]
    pub fn property_key(&self) -> &str {
        &self.property_key
    }
}

impl fmt::Display for ConfigurationPropertiesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "configuration type {} cannot bind field {} from property {}",
            self.configuration_type, self.field, self.property_key
        )
    }
}

impl fmt::Debug for ConfigurationPropertiesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConfigurationPropertiesError")
            .field("configuration_type", &self.configuration_type)
            .field("field", &self.field)
            .field("property_key", &self.property_key)
            .field("source", &"<redacted>")
            .finish()
    }
}

impl Error for ConfigurationPropertiesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}
