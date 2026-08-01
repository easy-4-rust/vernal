//! 数据大小(以字节为单位的不可变值)。
//!
//! 对标 Spring `org.springframework.util.unit.DataSize`,基于
//! [二进制前缀](https://en.wikipedia.org/wiki/Binary_prefix)(2 的幂次)。
//!
//! # 示例
//!
//! ```rust
//! use vernal_core::util::unit::{DataSize, DataUnit};
//!
//! // 构造
//! let size = DataSize::of_megabytes(5);
//! assert_eq!(size.to_bytes(), 5_242_880);
//!
//! // 解析
//! let parsed = DataSize::parse("12KB").unwrap();
//! assert_eq!(parsed.to_bytes(), 12_288);
//!
//! // 默认单位
//! let with_default = DataSize::parse_with_default_unit("20", DataUnit::Kilobytes).unwrap();
//! assert_eq!(with_default.to_bytes(), 20_480);
//! ```
//!
//! # 单位表
//!
//! | 单位 | 缩写 | 字节数 |
//! |---|---|---|
//! | 字节 | 1B | 1 |
//! | 千字节 | 1KB | 1,024 |
//! | 兆字节 | 1MB | 1,048,576 |
//! | 吉字节 | 1GB | 1,073,741,824 |
//! | 太字节 | 1TB | 1,099,511,627,776 |

use super::DataUnit;

/// 每千字节字节数。
pub const BYTES_PER_KB: i64 = 1024;
/// 每兆字节字节数。
pub const BYTES_PER_MB: i64 = BYTES_PER_KB * 1024;
/// 每吉字节字节数。
pub const BYTES_PER_GB: i64 = BYTES_PER_MB * 1024;
/// 每太字节字节数。
pub const BYTES_PER_TB: i64 = BYTES_PER_GB * 1024;

/// 数据大小(以字节为单位)。
///
/// 对标 Spring `DataSize`,不可变且线程安全。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DataSize {
    bytes: i64,
}

impl DataSize {
    /// 创建表示指定字节数的 `DataSize`。
    ///
    /// 对标 Spring `DataSize.ofBytes(long)`。
    #[must_use]
    pub const fn of_bytes(bytes: i64) -> Self {
        Self { bytes }
    }

    /// 创建表示指定千字节数的 `DataSize`。
    ///
    /// 对标 Spring `DataSize.ofKilobytes(long)`。
    #[must_use]
    pub const fn of_kilobytes(kilobytes: i64) -> Self {
        Self {
            bytes: kilobytes * BYTES_PER_KB,
        }
    }

    /// 创建表示指定兆字节数的 `DataSize`。
    ///
    /// 对标 Spring `DataSize.ofMegabytes(long)`。
    #[must_use]
    pub const fn of_megabytes(megabytes: i64) -> Self {
        Self {
            bytes: megabytes * BYTES_PER_MB,
        }
    }

    /// 创建表示指定吉字节数的 `DataSize`。
    ///
    /// 对标 Spring `DataSize.ofGigabytes(long)`。
    #[must_use]
    pub const fn of_gigabytes(gigabytes: i64) -> Self {
        Self {
            bytes: gigabytes * BYTES_PER_GB,
        }
    }

    /// 创建表示指定太字节数的 `DataSize`。
    ///
    /// 对标 Spring `DataSize.ofTerabytes(long)`。
    #[must_use]
    pub const fn of_terabytes(terabytes: i64) -> Self {
        Self {
            bytes: terabytes * BYTES_PER_TB,
        }
    }

    /// 创建指定单位与数量的 `DataSize`。
    ///
    /// 对标 Spring `DataSize.of(long, DataUnit)`。
    #[must_use]
    pub const fn of(amount: i64, unit: DataUnit) -> Self {
        Self {
            bytes: amount * unit.bytes_per_unit(),
        }
    }

    /// 从文本字符串解析(无默认单位时假设为字节)。
    ///
    /// 对标 Spring `DataSize.parse(CharSequence)`。
    ///
    /// # 示例
    ///
    /// - `"20"` → 20 字节
    /// - `"20B"` → 20 字节
    /// - `"12KB"` → 12,288 字节
    /// - `"5MB"` → 5,242,880 字节
    pub fn parse(text: &str) -> Result<Self, DataSizeParseError> {
        Self::parse_with_default_unit(text, None)
    }

    /// 从文本字符串解析,使用指定的默认单位。
    ///
    /// 对标 Spring `DataSize.parse(CharSequence, DataUnit)`。
    pub fn parse_with_default_unit(
        text: &str,
        default_unit: impl Into<Option<DataUnit>>,
    ) -> Result<Self, DataSizeParseError> {
        let trimmed: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        if trimmed.is_empty() {
            return Err(DataSizeParseError {
                input: text.to_string(),
                reason: "empty input".to_string(),
            });
        }

        // 分离数字部分与单位部分
        let mut split = trimmed.len();
        for (i, c) in trimmed.char_indices() {
            if !c.is_ascii_digit() && c != '-' && c != '+' {
                split = i;
                break;
            }
        }

        let amount_str = &trimmed[..split];
        let unit_str = &trimmed[split..];

        let amount: i64 =
            amount_str
                .parse()
                .map_err(|e: std::num::ParseIntError| DataSizeParseError {
                    input: text.to_string(),
                    reason: format!("invalid number '{amount_str}': {e}"),
                })?;

        let unit = if unit_str.is_empty() {
            // 无单位:使用默认单位(或字节)
            default_unit.into().unwrap_or(DataUnit::Bytes)
        } else {
            DataUnit::from_suffix(unit_str).map_err(|e| DataSizeParseError {
                input: text.to_string(),
                reason: e.to_string(),
            })?
        };

        Ok(Self::of(amount, unit))
    }

    /// 检查大小是否为负数(不包括零)。
    ///
    /// 对标 Spring `DataSize.isNegative()`。
    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.bytes < 0
    }

    /// 返回字节数。
    ///
    /// 对标 Spring `DataSize.toBytes()`。
    #[must_use]
    pub const fn to_bytes(&self) -> i64 {
        self.bytes
    }

    /// 返回千字节数(向下取整)。
    ///
    /// 对标 Spring `DataSize.toKilobytes()`。
    #[must_use]
    pub const fn to_kilobytes(&self) -> i64 {
        self.bytes / BYTES_PER_KB
    }

    /// 返回兆字节数(向下取整)。
    ///
    /// 对标 Spring `DataSize.toMegabytes()`。
    #[must_use]
    pub const fn to_megabytes(&self) -> i64 {
        self.bytes / BYTES_PER_MB
    }

    /// 返回吉字节数(向下取整)。
    ///
    /// 对标 Spring `DataSize.toGigabytes()`。
    #[must_use]
    pub const fn to_gigabytes(&self) -> i64 {
        self.bytes / BYTES_PER_GB
    }

    /// 返回太字节数(向下取整)。
    ///
    /// 对标 Spring `DataSize.toTerabytes()`。
    #[must_use]
    pub const fn to_terabytes(&self) -> i64 {
        self.bytes / BYTES_PER_TB
    }
}

impl Default for DataSize {
    fn default() -> Self {
        Self::of_bytes(0)
    }
}

impl std::fmt::Display for DataSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bytes", self.bytes)
    }
}

/// 数据大小解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSizeParseError {
    /// 原始输入
    pub input: String,
    /// 错误原因
    pub reason: String,
}

impl std::fmt::Display for DataSizeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "'{}' is not a valid data size: {}",
            self.input, self.reason
        )
    }
}

impl std::error::Error for DataSizeParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn of_bytes_returns_raw_value() {
        assert_eq!(DataSize::of_bytes(100).to_bytes(), 100);
        assert_eq!(DataSize::of_bytes(-50).to_bytes(), -50);
        assert_eq!(DataSize::of_bytes(0).to_bytes(), 0);
    }

    #[test]
    fn of_kilobytes_multiplies_by_1024() {
        assert_eq!(DataSize::of_kilobytes(1).to_bytes(), 1024);
        assert_eq!(DataSize::of_kilobytes(12).to_bytes(), 12_288);
    }

    #[test]
    fn of_megabytes_multiplies_by_1024_squared() {
        assert_eq!(DataSize::of_megabytes(1).to_bytes(), 1_048_576);
        assert_eq!(DataSize::of_megabytes(5).to_bytes(), 5_242_880);
    }

    #[test]
    fn of_gigabytes_multiplies_by_1024_cubed() {
        assert_eq!(DataSize::of_gigabytes(1).to_bytes(), 1_073_741_824);
    }

    #[test]
    fn of_terabytes_multiplies_by_1024_to_the_fourth() {
        assert_eq!(DataSize::of_terabytes(1).to_bytes(), 1_099_511_627_776);
    }

    #[test]
    fn of_with_unit_works() {
        assert_eq!(DataSize::of(5, DataUnit::Megabytes).to_bytes(), 5_242_880);
        assert_eq!(DataSize::of(20, DataUnit::Bytes).to_bytes(), 20);
    }

    #[test]
    fn parse_plain_number_defaults_to_bytes() {
        let size = DataSize::parse("20").unwrap();
        assert_eq!(size.to_bytes(), 20);
    }

    #[test]
    fn parse_explicit_bytes_suffix() {
        let size = DataSize::parse("20B").unwrap();
        assert_eq!(size.to_bytes(), 20);
    }

    #[test]
    fn parse_kilobytes() {
        let size = DataSize::parse("12KB").unwrap();
        assert_eq!(size.to_bytes(), 12_288);
    }

    #[test]
    fn parse_megabytes() {
        let size = DataSize::parse("5MB").unwrap();
        assert_eq!(size.to_bytes(), 5_242_880);
    }

    #[test]
    fn parse_gigabytes() {
        let size = DataSize::parse("1GB").unwrap();
        assert_eq!(size.to_bytes(), 1_073_741_824);
    }

    #[test]
    fn parse_terabytes() {
        let size = DataSize::parse("1TB").unwrap();
        assert_eq!(size.to_bytes(), 1_099_511_627_776);
    }

    #[test]
    fn parse_with_default_unit_when_no_unit() {
        let size = DataSize::parse_with_default_unit("20", DataUnit::Kilobytes).unwrap();
        assert_eq!(size.to_bytes(), 20_480);
    }

    #[test]
    fn parse_with_default_unit_overridden_by_explicit_unit() {
        let size = DataSize::parse_with_default_unit("5MB", DataUnit::Kilobytes).unwrap();
        assert_eq!(size.to_bytes(), 5_242_880);
    }

    #[test]
    fn parse_with_whitespace_is_trimmed() {
        let size = DataSize::parse("  12KB  ").unwrap();
        assert_eq!(size.to_bytes(), 12_288);
    }

    #[test]
    fn parse_negative_value() {
        let size = DataSize::parse("-5MB").unwrap();
        assert_eq!(size.to_bytes(), -5_242_880);
        assert!(size.is_negative());
    }

    #[test]
    fn parse_empty_returns_error() {
        let err = DataSize::parse("").unwrap_err();
        assert!(err.reason.contains("empty"));
    }

    #[test]
    fn parse_invalid_suffix_returns_error() {
        let err = DataSize::parse("5PB").unwrap_err();
        assert!(err.to_string().contains("PB"));
    }

    #[test]
    fn parse_non_number_returns_error() {
        let err = DataSize::parse("abc").unwrap_err();
        assert!(err.to_string().contains("not a valid data size"));
    }

    #[test]
    fn to_kilobytes_truncates_down() {
        let size = DataSize::of_bytes(1500);
        assert_eq!(size.to_kilobytes(), 1);
    }

    #[test]
    fn to_megabytes_truncates_down() {
        let size = DataSize::of_bytes(1_500_000);
        assert_eq!(size.to_megabytes(), 1);
    }

    #[test]
    fn is_negative_returns_true_for_negative() {
        assert!(DataSize::of_bytes(-1).is_negative());
        assert!(!DataSize::of_bytes(0).is_negative());
        assert!(!DataSize::of_bytes(1).is_negative());
    }

    #[test]
    fn equality_and_ordering() {
        let small = DataSize::of_bytes(100);
        let big = DataSize::of_kilobytes(1);
        assert_eq!(small, DataSize::of_bytes(100));
        assert!(small < big);
        assert!(big > small);
    }

    #[test]
    fn comparable_via_ord() {
        let mut sizes = [DataSize::of_megabytes(5),
            DataSize::of_bytes(100),
            DataSize::of_kilobytes(1)];
        sizes.sort();
        assert_eq!(sizes[0].to_bytes(), 100);
        assert_eq!(sizes[1].to_bytes(), 1024);
        assert_eq!(sizes[2].to_bytes(), 5_242_880);
    }

    #[test]
    fn display_shows_bytes() {
        let size = DataSize::of_kilobytes(1);
        assert_eq!(size.to_string(), "1024 bytes");
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(DataSize::default().to_bytes(), 0);
    }

    #[test]
    fn to_gigabytes_truncates_down() {
        // 对标 Spring `DataSize.toGigabytes()`
        // 1GB 边界正好
        assert_eq!(DataSize::of_bytes(BYTES_PER_GB).to_gigabytes(), 1);
        // 1GB - 1 字节向下取整为 0
        assert_eq!(DataSize::of_bytes(BYTES_PER_GB - 1).to_gigabytes(), 0);
        // 2.5GB 向下取整为 2
        assert_eq!(
            DataSize::of_bytes(BYTES_PER_GB * 5 / 2).to_gigabytes(),
            2
        );
    }

    #[test]
    fn to_terabytes_truncates_down() {
        // 对标 Spring `DataSize.toTerabytes()`
        // 1TB 边界正好
        assert_eq!(DataSize::of_bytes(BYTES_PER_TB).to_terabytes(), 1);
        // 1TB - 1 字节向下取整为 0
        assert_eq!(DataSize::of_bytes(BYTES_PER_TB - 1).to_terabytes(), 0);
        // 2TB 整
        assert_eq!(DataSize::of_terabytes(2).to_terabytes(), 2);
    }
}
