//! 数据单位枚举。
//!
//! 对标 Spring `org.springframework.util.unit.DataUnit`。
//!
//! 单位前缀基于[二进制前缀](https://en.wikipedia.org/wiki/Binary_prefix),
//! 表示乘以 2 的幂次:
//!
//! | 常量 | 数据大小 | 2 的幂 | 字节 |
//! |---|---|---|---|
//! | [`DataUnit::Bytes`] | 1B | 2^0 | 1 |
//! | [`DataUnit::Kilobytes`] | 1KB | 2^10 | 1,024 |
//! | [`DataUnit::Megabytes`] | 1MB | 2^20 | 1,048,576 |
//! | [`DataUnit::Gigabytes`] | 1GB | 2^30 | 1,073,741,824 |
//! | [`DataUnit::Terabytes`] | 1TB | 2^40 | 1,099,511,627,776 |

use super::DataSize;

/// 数据单位。
///
/// 对标 Spring `DataUnit` 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataUnit {
    /// 字节,后缀 `B`。
    Bytes,
    /// 千字节,后缀 `KB`(1024 字节)。
    Kilobytes,
    /// 兆字节,后缀 `MB`(1024^2 字节)。
    Megabytes,
    /// 吉字节,后缀 `GB`(1024^3 字节)。
    Gigabytes,
    /// 太字节,后缀 `TB`(1024^4 字节)。
    Terabytes,
}

impl DataUnit {
    /// 单位后缀(对标 Spring `suffix` 字段)。
    #[must_use]
    pub fn suffix(&self) -> &'static str {
        match self {
            Self::Bytes => "B",
            Self::Kilobytes => "KB",
            Self::Megabytes => "MB",
            Self::Gigabytes => "GB",
            Self::Terabytes => "TB",
        }
    }

    /// 单位对应的 1 单位 DataSize(对标 Spring `size()`)。
    #[must_use]
    pub fn size(&self) -> DataSize {
        match self {
            Self::Bytes => DataSize::of_bytes(1),
            Self::Kilobytes => DataSize::of_kilobytes(1),
            Self::Megabytes => DataSize::of_megabytes(1),
            Self::Gigabytes => DataSize::of_gigabytes(1),
            Self::Terabytes => DataSize::of_terabytes(1),
        }
    }

    /// 单位对应的字节数。
    #[must_use]
    pub const fn bytes_per_unit(&self) -> i64 {
        match self {
            Self::Bytes => 1,
            Self::Kilobytes => 1024,
            Self::Megabytes => 1024 * 1024,
            Self::Gigabytes => 1024 * 1024 * 1024,
            Self::Terabytes => 1024_i64.pow(4),
        }
    }

    /// 从后缀字符串解析 DataUnit(对标 Spring `fromSuffix(String)`)。
    ///
    /// # 错误
    ///
    /// 返回 `Err` 如果后缀不匹配任何标准单位。
    pub fn from_suffix(suffix: &str) -> Result<Self, UnknownDataUnitSuffix> {
        match suffix {
            "B" => Ok(Self::Bytes),
            "KB" => Ok(Self::Kilobytes),
            "MB" => Ok(Self::Megabytes),
            "GB" => Ok(Self::Gigabytes),
            "TB" => Ok(Self::Terabytes),
            _ => Err(UnknownDataUnitSuffix {
                suffix: suffix.to_string(),
            }),
        }
    }
}

impl std::fmt::Display for DataUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.suffix())
    }
}

/// 未知的数据单位后缀错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownDataUnitSuffix {
    /// 未识别的后缀
    pub suffix: String,
}

impl std::fmt::Display for UnknownDataUnitSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unknown data unit suffix '{}'", self.suffix)
    }
}

impl std::error::Error for UnknownDataUnitSuffix {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_matches_spring_constants() {
        assert_eq!(DataUnit::Bytes.suffix(), "B");
        assert_eq!(DataUnit::Kilobytes.suffix(), "KB");
        assert_eq!(DataUnit::Megabytes.suffix(), "MB");
        assert_eq!(DataUnit::Gigabytes.suffix(), "GB");
        assert_eq!(DataUnit::Terabytes.suffix(), "TB");
    }

    #[test]
    fn size_returns_one_unit_in_bytes() {
        assert_eq!(DataUnit::Bytes.size().to_bytes(), 1);
        assert_eq!(DataUnit::Kilobytes.size().to_bytes(), 1024);
        assert_eq!(DataUnit::Megabytes.size().to_bytes(), 1024 * 1024);
        assert_eq!(DataUnit::Gigabytes.size().to_bytes(), 1024 * 1024 * 1024);
        assert_eq!(DataUnit::Terabytes.size().to_bytes(), 1024_i64.pow(4));
    }

    #[test]
    fn bytes_per_unit_matches_spring_constants() {
        assert_eq!(DataUnit::Bytes.bytes_per_unit(), 1);
        assert_eq!(DataUnit::Kilobytes.bytes_per_unit(), 1024);
        assert_eq!(DataUnit::Terabytes.bytes_per_unit(), 1_099_511_627_776);
    }

    #[test]
    fn from_suffix_parses_all_5_units() {
        assert_eq!(DataUnit::from_suffix("B").unwrap(), DataUnit::Bytes);
        assert_eq!(DataUnit::from_suffix("KB").unwrap(), DataUnit::Kilobytes);
        assert_eq!(DataUnit::from_suffix("MB").unwrap(), DataUnit::Megabytes);
        assert_eq!(DataUnit::from_suffix("GB").unwrap(), DataUnit::Gigabytes);
        assert_eq!(DataUnit::from_suffix("TB").unwrap(), DataUnit::Terabytes);
    }

    #[test]
    fn from_suffix_rejects_unknown() {
        let err = DataUnit::from_suffix("PB").unwrap_err();
        assert_eq!(err.suffix, "PB");
        assert!(err.to_string().contains("PB"));
    }

    #[test]
    fn from_suffix_rejects_lowercase() {
        // Spring 严格区分大小写,Kb 不等于 KB
        assert!(DataUnit::from_suffix("Kb").is_err());
        assert!(DataUnit::from_suffix("kb").is_err());
    }

    #[test]
    fn display_outputs_suffix() {
        assert_eq!(DataUnit::Megabytes.to_string(), "MB");
    }
}
