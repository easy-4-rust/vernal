//! 内存映射属性来源对象。

use std::collections::BTreeMap;

use crate::{EnvironmentError, PropertySource};

/// 由有序字符串 Map 提供属性的不可变 `PropertySource`。
///
/// 该实现适合测试、程序化配置，以及 Hutool-Rust、Sa-Token-Rust Bridge 把已经
/// 解析好的配置映射注入 Vernal。对象不读取文件或进程环境，因此构建后的行为
/// 完全确定；属性值不会出现在 `Debug` 输出中。
pub struct MapPropertySource {
    name: String,
    values: BTreeMap<String, String>,
}

impl MapPropertySource {
    /// 创建并完整校验一个不可变属性来源。
    ///
    /// # Errors
    ///
    /// 来源名称为空、属性键非法或输入中存在重复键时返回 [`EnvironmentError`]；
    /// 失败不会保留部分 Map。
    pub fn new<I, K, V>(name: impl Into<String>, values: I) -> Result<Self, EnvironmentError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let name = name.into();
        if name.trim().is_empty() || name.chars().any(char::is_control) {
            return Err(EnvironmentError::InvalidPropertySourceName { name });
        }

        // 先在局部 Map 中校验全部输入，任一键失败都不会产生半成品来源。
        let mut collected = BTreeMap::new();
        for (key, value) in values {
            let key = key.into();
            if key.is_empty() || key.chars().any(char::is_whitespace) {
                return Err(EnvironmentError::InvalidPropertyKey { key });
            }
            if collected.insert(key.clone(), value.into()).is_some() {
                return Err(EnvironmentError::DuplicatePropertyKey {
                    source_name: name,
                    key,
                });
            }
        }

        Ok(Self {
            name,
            values: collected,
        })
    }

    /// 返回属性数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// 判断来源是否没有属性。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl PropertySource for MapPropertySource {
    fn name(&self) -> &str {
        &self.name
    }

    fn get(&self, key: &str) -> Result<Option<String>, EnvironmentError> {
        Ok(self.values.get(key).cloned())
    }
}

impl std::fmt::Debug for MapPropertySource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MapPropertySource")
            .field("name", &self.name)
            .field("property_count", &self.values.len())
            .finish()
    }
}
