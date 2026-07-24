//! Context-local 应用环境对象。

use std::{any::type_name, fmt, str::FromStr, sync::Arc};

use crate::{ApplicationEnvironmentBuilder, EnvironmentError, EnvironmentSnapshot, PropertySource};

const MAX_PLACEHOLDER_DEPTH: usize = 32;

/// Context-local、不可变且按优先级解析的应用环境。
///
/// 本对象吸收 Spring Environment 的“PropertySource + Profile”核心语义，但保持
/// Rust 原生：具体 TOML/YAML/Hutool `.setting`/环境变量/Nacos 加载器由适配器
/// 提供，Environment 只负责精确键查找、`${key:default}` 展开、类型转换和来源
/// 顺序。它不使用进程级全局状态，也不会把属性值写入 `Debug` 或错误文本。
pub struct ApplicationEnvironment {
    sources: Arc<[Arc<dyn PropertySource>]>,
    active_profiles: Arc<[String]>,
    default_profiles: Arc<[String]>,
}

impl ApplicationEnvironment {
    /// 创建应用环境建造器。
    #[must_use]
    pub fn builder() -> ApplicationEnvironmentBuilder {
        ApplicationEnvironmentBuilder::new()
    }

    /// 返回没有 PropertySource、使用 `default` Profile 的环境。
    #[must_use]
    pub fn empty() -> Self {
        ApplicationEnvironmentBuilder::new().build()
    }

    /// 按优先级读取并展开一个可选字符串属性。
    ///
    /// 高优先级来源返回值后不会继续访问低优先级来源。`${key}` 引用使用相同
    /// 优先级规则；`${key:default}` 在引用缺失时展开默认文本，并支持嵌套占位符。
    ///
    /// # Errors
    ///
    /// 属性键非法、来源读取失败、占位符缺失/循环/超限时返回
    /// [`EnvironmentError`]。
    pub fn property(&self, key: &str) -> Result<Option<String>, EnvironmentError> {
        Self::validate_key(key)?;
        let Some((_source_name, raw)) = self.locate(key)? else {
            return Ok(None);
        };
        let mut path = vec![key.to_owned()];
        self.resolve_text(key, &raw, &mut path, 0).map(Some)
    }

    /// 按优先级读取一个未展开占位符的原始字符串属性。
    ///
    /// 该入口主要供需要自行解释模板语法的配置适配器使用；普通业务配置应优先
    /// 调用 [`Self::property`]。
    ///
    /// # Errors
    ///
    /// 属性键非法或来源读取失败时返回 [`EnvironmentError`]。
    pub fn raw_property(&self, key: &str) -> Result<Option<String>, EnvironmentError> {
        Self::validate_key(key)?;
        self.locate(key)
            .map(|located| located.map(|(_, value)| value))
    }

    /// 把可选属性解析为调用方指定的 Rust 类型。
    ///
    /// 解析错误不会保留原始属性值或底层 `FromStr` 错误文本，防止敏感配置泄漏。
    ///
    /// # Errors
    ///
    /// 查找、占位符展开或类型转换失败时返回 [`EnvironmentError`]。
    pub fn get<T>(&self, key: &str) -> Result<Option<T>, EnvironmentError>
    where
        T: FromStr,
    {
        Self::validate_key(key)?;
        let Some((source_name, raw)) = self.locate(key)? else {
            return Ok(None);
        };
        let mut path = vec![key.to_owned()];
        let value = self.resolve_text(key, &raw, &mut path, 0)?;
        value
            .parse::<T>()
            .map(Some)
            .map_err(|_| EnvironmentError::InvalidPropertyValue {
                key: key.to_owned(),
                source_name,
                target_type: type_name::<T>(),
            })
    }

    /// 读取并解析一个必需属性。
    ///
    /// # Errors
    ///
    /// 属性不存在时返回 [`EnvironmentError::MissingProperty`]；其他解析失败与
    /// [`Self::get`] 相同。
    pub fn require<T>(&self, key: &str) -> Result<T, EnvironmentError>
    where
        T: FromStr,
    {
        self.get(key)?
            .ok_or_else(|| EnvironmentError::MissingProperty {
                key: key.to_owned(),
            })
    }

    /// 判断任一来源是否包含指定属性。
    ///
    /// # Errors
    ///
    /// 属性键非法或来源读取失败时返回 [`EnvironmentError`]。
    pub fn contains_property(&self, key: &str) -> Result<bool, EnvironmentError> {
        Self::validate_key(key)?;
        self.locate(key).map(|located| located.is_some())
    }

    /// 返回从高到低的 `PropertySource` 名称。
    ///
    /// 只暴露来源名，不枚举属性键和值，避免诊断接口成为凭证泄漏面。
    #[must_use]
    pub fn property_source_names(&self) -> impl ExactSizeIterator<Item = &str> {
        self.sources.iter().map(|source| source.name())
    }

    /// 返回显式启用的 Profile；空集合表示应使用默认 Profile。
    #[must_use]
    pub fn active_profiles(&self) -> &[String] {
        &self.active_profiles
    }

    /// 返回没有 Active Profile 时使用的默认 Profile。
    #[must_use]
    pub fn default_profiles(&self) -> &[String] {
        &self.default_profiles
    }

    /// 返回当前实际生效的 Profile。
    pub fn effective_profiles(&self) -> impl ExactSizeIterator<Item = &str> {
        let profiles = if self.active_profiles.is_empty() {
            &self.default_profiles
        } else {
            &self.active_profiles
        };
        profiles.iter().map(String::as_str)
    }

    /// 判断指定 Profile 当前是否生效。
    ///
    /// # Errors
    ///
    /// Profile 为空或包含空白字符时返回 [`EnvironmentError`]。
    pub fn is_profile_active(&self, profile: &str) -> Result<bool, EnvironmentError> {
        if profile.is_empty() || profile.chars().any(char::is_whitespace) {
            return Err(EnvironmentError::InvalidProfile {
                profile: profile.to_owned(),
            });
        }
        Ok(self.effective_profiles().any(|active| active == profile))
    }

    /// 生成不包含属性键和值的可序列化诊断快照。
    #[must_use]
    pub fn snapshot(&self) -> EnvironmentSnapshot {
        EnvironmentSnapshot::new(
            self.property_source_names().map(str::to_owned).collect(),
            self.active_profiles.to_vec(),
            self.default_profiles.to_vec(),
            self.effective_profiles().map(str::to_owned).collect(),
        )
    }

    /// 由建造器冻结内部集合，避免公开构造绕过名称和重复校验。
    pub(crate) fn new(
        sources: Vec<Arc<dyn PropertySource>>,
        active_profiles: Vec<String>,
        default_profiles: Vec<String>,
    ) -> Self {
        Self {
            sources: Arc::from(sources),
            active_profiles: Arc::from(active_profiles),
            default_profiles: Arc::from(default_profiles),
        }
    }

    /// 按来源优先级查找原始值，并保留来源名供类型错误诊断。
    fn locate(&self, key: &str) -> Result<Option<(String, String)>, EnvironmentError> {
        for source in self.sources.iter() {
            if let Some(value) = source.get(key)? {
                return Ok(Some((source.name().to_owned(), value)));
            }
        }
        Ok(None)
    }

    /// 递归展开一个属性文本，并通过路径与深度双重约束拒绝无限递归。
    fn resolve_text(
        &self,
        property: &str,
        input: &str,
        path: &mut Vec<String>,
        depth: usize,
    ) -> Result<String, EnvironmentError> {
        if depth > MAX_PLACEHOLDER_DEPTH {
            return Err(EnvironmentError::PlaceholderDepthExceeded {
                property: property.to_owned(),
                limit: MAX_PLACEHOLDER_DEPTH,
            });
        }

        let mut output = String::with_capacity(input.len());
        let mut cursor = 0;
        while let Some(relative_start) = input[cursor..].find("${") {
            let start = cursor + relative_start;
            output.push_str(&input[cursor..start]);
            let Some(end) = Self::matching_placeholder_end(input, start) else {
                return Err(EnvironmentError::MalformedPlaceholder {
                    property: property.to_owned(),
                });
            };
            let body = &input[start + 2..end];
            let (placeholder, default) = body
                .split_once(':')
                .map_or((body, None), |(key, value)| (key, Some(value)));
            if placeholder.is_empty() || placeholder.chars().any(char::is_whitespace) {
                return Err(EnvironmentError::MalformedPlaceholder {
                    property: property.to_owned(),
                });
            }

            // 已存在于当前展开路径即为真实配置环；路径只含属性键，不含属性值。
            if path.iter().any(|visited| visited == placeholder) {
                let mut cycle = path.clone();
                cycle.push(placeholder.to_owned());
                return Err(EnvironmentError::CircularPlaceholder { path: cycle });
            }

            let replacement = if let Some((_source_name, raw)) = self.locate(placeholder)? {
                path.push(placeholder.to_owned());
                let resolved = self.resolve_text(property, &raw, path, depth + 1)?;
                path.pop();
                resolved
            } else if let Some(default) = default {
                self.resolve_text(property, default, path, depth + 1)?
            } else {
                return Err(EnvironmentError::UnresolvedPlaceholder {
                    property: property.to_owned(),
                    placeholder: placeholder.to_owned(),
                });
            };
            output.push_str(&replacement);
            cursor = end + 1;
        }
        output.push_str(&input[cursor..]);
        Ok(output)
    }

    /// 查找支持嵌套默认值的占位符闭合位置。
    fn matching_placeholder_end(input: &str, start: usize) -> Option<usize> {
        let mut cursor = start + 2;
        let mut nesting = 1_usize;
        while cursor < input.len() {
            if input[cursor..].starts_with("${") {
                nesting = nesting.saturating_add(1);
                cursor += 2;
                continue;
            }
            if input[cursor..].starts_with('}') {
                nesting = nesting.saturating_sub(1);
                if nesting == 0 {
                    return Some(cursor);
                }
                cursor += 1;
                continue;
            }
            let character = input[cursor..].chars().next()?;
            cursor += character.len_utf8();
        }
        None
    }

    /// 统一验证公开查询入口的属性键。
    fn validate_key(key: &str) -> Result<(), EnvironmentError> {
        if key.is_empty() || key.chars().any(char::is_whitespace) {
            return Err(EnvironmentError::InvalidPropertyKey {
                key: key.to_owned(),
            });
        }
        Ok(())
    }
}

impl Default for ApplicationEnvironment {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Debug for ApplicationEnvironment {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationEnvironment")
            .field("sources", &self.property_source_names().collect::<Vec<_>>())
            .field("active_profiles", &self.active_profiles)
            .field("default_profiles", &self.default_profiles)
            .finish()
    }
}
