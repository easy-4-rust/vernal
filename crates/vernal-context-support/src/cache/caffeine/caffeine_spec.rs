//! CaffeineSpec 字符串配置解析 — 对标 `com.github.benmanes.caffeine.cache.CaffeineSpec`。
//!
//! 解析 Caffeine 格式的配置字符串，例如：
//! - `"maximumSize=10000"`
//! - `"maximumSize=10000,expireAfterWrite=5m"`
//! - `"maximumSize=10000,expireAfterAccess=10m,recordStats"`

use std::time::Duration;

/// CaffeineSpec 解析错误。
#[derive(Debug, thiserror::Error)]
pub enum CaffeineSpecParseError {
    /// 无效的配置键。
    #[error("无效的配置键：{0}")]
    InvalidKey(String),

    /// 无效的配置值。
    #[error("无效的配置值（key={key}）：{value}")]
    InvalidValue { key: String, value: String },

    /// 无效的时间格式。
    #[error("无效的时间格式：{0}")]
    InvalidDuration(String),

    /// 重复的配置键。
    #[error("重复的配置键：{0}")]
    DuplicateKey(String),

    /// 空的配置字符串。
    #[error("空的配置字符串")]
    Empty,
}

/// CaffeineSpec 配置。
///
/// 对标 Spring 的 `CaffeineSpec`，解析 Caffeine 格式的配置字符串。
///
/// # 支持的配置键
///
/// | 键 | 类型 | 说明 |
/// |---|---|---|
/// | `maximumSize` | u64 | 最大缓存条目数 |
/// | `maximumWeight` | u64 | 最大权重（与 maximumSize 互斥） |
/// | `expireAfterWrite` | Duration | 写入后过期时间 |
/// | `expireAfterAccess` | Duration | 访问后过期时间 |
/// | `refreshAfterWrite` | Duration | 写入后刷新时间 |
/// | `initialCapacity` | usize | 初始容量 |
/// | `recordStats` | bool | 是否记录统计信息 |
/// | `weakKeys` | bool | 是否使用弱键 |
/// | `weakValues` | bool | 是否使用弱值 |
/// | `softValues` | bool | 是否使用软值 |
#[derive(Debug, Clone, PartialEq)]
pub struct CaffeineSpec {
    /// 最大缓存条目数。
    pub maximum_size: Option<u64>,
    /// 最大权重。
    pub maximum_weight: Option<u64>,
    /// 写入后过期时间。
    pub expire_after_write: Option<Duration>,
    /// 访问后过期时间。
    pub expire_after_access: Option<Duration>,
    /// 写入后刷新时间。
    pub refresh_after_write: Option<Duration>,
    /// 初始容量。
    pub initial_capacity: Option<usize>,
    /// 是否记录统计信息。
    pub record_stats: bool,
    /// 是否使用弱键。
    pub weak_keys: bool,
    /// 是否使用弱值。
    pub weak_values: bool,
    /// 是否使用软值。
    pub soft_values: bool,
}

impl Default for CaffeineSpec {
    fn default() -> Self {
        Self {
            maximum_size: None,
            maximum_weight: None,
            expire_after_write: None,
            expire_after_access: None,
            refresh_after_write: None,
            initial_capacity: None,
            record_stats: false,
            weak_keys: false,
            weak_values: false,
            soft_values: false,
        }
    }
}

impl CaffeineSpec {
    /// 解析 CaffeineSpec 字符串。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let spec = CaffeineSpec::parse("maximumSize=10000,expireAfterWrite=5m").unwrap();
    /// assert_eq!(spec.maximum_size, Some(10000));
    /// assert_eq!(spec.expire_after_write, Some(Duration::from_secs(300)));
    /// ```
    pub fn parse(spec: &str) -> Result<Self, CaffeineSpecParseError> {
        if spec.trim().is_empty() {
            return Err(CaffeineSpecParseError::Empty);
        }

        let mut result = Self::default();

        for part in spec.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (key, value) = if let Some(eq_pos) = part.find('=') {
                (&part[..eq_pos], Some(part[eq_pos + 1..].trim()))
            } else {
                // 布尔标志（如 recordStats）
                (part, None)
            };

            match key {
                "maximumSize" => {
                    let val = value.ok_or_else(|| CaffeineSpecParseError::InvalidValue {
                        key: key.to_string(),
                        value: String::new(),
                    })?;
                    let size: u64 =
                        val.parse()
                            .map_err(|_| CaffeineSpecParseError::InvalidValue {
                                key: key.to_string(),
                                value: val.to_string(),
                            })?;
                    if result.maximum_size.is_some() {
                        return Err(CaffeineSpecParseError::DuplicateKey(key.to_string()));
                    }
                    result.maximum_size = Some(size);
                }
                "maximumWeight" => {
                    let val = value.ok_or_else(|| CaffeineSpecParseError::InvalidValue {
                        key: key.to_string(),
                        value: String::new(),
                    })?;
                    let weight: u64 =
                        val.parse()
                            .map_err(|_| CaffeineSpecParseError::InvalidValue {
                                key: key.to_string(),
                                value: val.to_string(),
                            })?;
                    result.maximum_weight = Some(weight);
                }
                "expireAfterWrite" => {
                    let val = value.ok_or_else(|| CaffeineSpecParseError::InvalidValue {
                        key: key.to_string(),
                        value: String::new(),
                    })?;
                    let duration = Self::parse_duration(val)?;
                    result.expire_after_write = Some(duration);
                }
                "expireAfterAccess" => {
                    let val = value.ok_or_else(|| CaffeineSpecParseError::InvalidValue {
                        key: key.to_string(),
                        value: String::new(),
                    })?;
                    let duration = Self::parse_duration(val)?;
                    result.expire_after_access = Some(duration);
                }
                "refreshAfterWrite" => {
                    let val = value.ok_or_else(|| CaffeineSpecParseError::InvalidValue {
                        key: key.to_string(),
                        value: String::new(),
                    })?;
                    let duration = Self::parse_duration(val)?;
                    result.refresh_after_write = Some(duration);
                }
                "initialCapacity" => {
                    let val = value.ok_or_else(|| CaffeineSpecParseError::InvalidValue {
                        key: key.to_string(),
                        value: String::new(),
                    })?;
                    let cap: usize =
                        val.parse()
                            .map_err(|_| CaffeineSpecParseError::InvalidValue {
                                key: key.to_string(),
                                value: val.to_string(),
                            })?;
                    result.initial_capacity = Some(cap);
                }
                "recordStats" => {
                    result.record_stats = true;
                }
                "weakKeys" => {
                    result.weak_keys = true;
                }
                "weakValues" => {
                    result.weak_values = true;
                }
                "softValues" => {
                    result.soft_values = true;
                }
                _ => {
                    return Err(CaffeineSpecParseError::InvalidKey(key.to_string()));
                }
            }
        }

        Ok(result)
    }

    /// 解析时间持续量字符串。
    ///
    /// 支持格式：
    /// - `Ns`：N 秒
    /// - `Nm`：N 分钟
    /// - `Nh`：N 小时
    /// - `Nd`：N 天
    /// - 纯数字：毫秒
    fn parse_duration(value: &str) -> Result<Duration, CaffeineSpecParseError> {
        let value = value.trim();

        if let Some(s) = value.strip_suffix('s') {
            let secs: u64 = s
                .parse()
                .map_err(|_| CaffeineSpecParseError::InvalidDuration(value.to_string()))?;
            return Ok(Duration::from_secs(secs));
        }
        if let Some(m) = value.strip_suffix('m') {
            let mins: u64 = m
                .parse()
                .map_err(|_| CaffeineSpecParseError::InvalidDuration(value.to_string()))?;
            return Ok(Duration::from_secs(mins * 60));
        }
        if let Some(h) = value.strip_suffix('h') {
            let hours: u64 = h
                .parse()
                .map_err(|_| CaffeineSpecParseError::InvalidDuration(value.to_string()))?;
            return Ok(Duration::from_secs(hours * 3600));
        }
        if let Some(d) = value.strip_suffix('d') {
            let days: u64 = d
                .parse()
                .map_err(|_| CaffeineSpecParseError::InvalidDuration(value.to_string()))?;
            return Ok(Duration::from_secs(days * 86400));
        }

        // 纯数字作为毫秒
        let millis: u64 = value
            .parse()
            .map_err(|_| CaffeineSpecParseError::InvalidDuration(value.to_string()))?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        assert!(CaffeineSpec::parse("").is_err());
    }

    #[test]
    fn test_parse_maximum_size() {
        let spec = CaffeineSpec::parse("maximumSize=10000").unwrap();
        assert_eq!(spec.maximum_size, Some(10000));
    }

    #[test]
    fn test_parse_expire_after_write() {
        let spec = CaffeineSpec::parse("expireAfterWrite=5m").unwrap();
        assert_eq!(spec.expire_after_write, Some(Duration::from_secs(300)));
    }

    #[test]
    fn test_parse_multiple() {
        let spec =
            CaffeineSpec::parse("maximumSize=10000,expireAfterWrite=5m,recordStats").unwrap();
        assert_eq!(spec.maximum_size, Some(10000));
        assert_eq!(spec.expire_after_write, Some(Duration::from_secs(300)));
        assert!(spec.record_stats);
    }

    #[test]
    fn test_parse_duration_formats() {
        assert_eq!(
            CaffeineSpec::parse_duration("30s").unwrap(),
            Duration::from_secs(30)
        );
        assert_eq!(
            CaffeineSpec::parse_duration("5m").unwrap(),
            Duration::from_secs(300)
        );
        assert_eq!(
            CaffeineSpec::parse_duration("2h").unwrap(),
            Duration::from_secs(7200)
        );
        assert_eq!(
            CaffeineSpec::parse_duration("1d").unwrap(),
            Duration::from_secs(86400)
        );
        assert_eq!(
            CaffeineSpec::parse_duration("1500").unwrap(),
            Duration::from_millis(1500)
        );
    }

    #[test]
    fn test_parse_invalid_key() {
        assert!(CaffeineSpec::parse("invalidKey=100").is_err());
    }

    #[test]
    fn test_parse_invalid_value() {
        assert!(CaffeineSpec::parse("maximumSize=abc").is_err());
    }

    #[test]
    fn test_parse_invalid_duration() {
        assert!(CaffeineSpec::parse("expireAfterWrite=abc").is_err());
    }

    #[test]
    fn test_default() {
        let spec = CaffeineSpec::default();
        assert_eq!(spec.maximum_size, None);
        assert_eq!(spec.expire_after_write, None);
        assert!(!spec.record_stats);
    }
}
