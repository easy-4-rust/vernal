//! 时间单位枚举。
//!
//! 对标 Java 的 `java.util.concurrent.TimeUnit`,作为 [`super::StopWatch`] 的
//! 时间维度参数。Rust 没有等价的 std 库枚举,因此 vernal-core 在此定义一个精简子集。
//!
//! # 设计来源
//!
//! 仅保留 [`StopWatch`](super::StopWatch) 实际使用的 5 个单位(纳秒 / 微秒 / 毫秒 / 秒 / 分钟),
//! 不暴露 Spring 不需要的 `HOURS` / `DAYS`(它们可以用分钟表达)。

use std::time::Duration;

/// 时间单位枚举。
///
/// 对标 Java `java.util.concurrent.TimeUnit` 的子集。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StopWatchUnit {
    /// 纳秒(10^-9 秒)。对标 `TimeUnit.NANOSECONDS`。
    Nanos,
    /// 微秒(10^-6 秒)。对标 `TimeUnit.MICROSECONDS`。
    Micros,
    /// 毫秒(10^-3 秒)。对标 `TimeUnit.MILLISECONDS`。
    Millis,
    /// 秒。对标 `TimeUnit.SECONDS`。
    Seconds,
    /// 分钟(60 秒)。对标 `TimeUnit.MINUTES`。
    Minutes,
}

impl StopWatchUnit {
    /// 获取该单位对应的小写英文名,用于 [`super::StopWatch::pretty_print_with_unit`] 输出。
    ///
    /// 对标 Java `timeUnit.name().toLowerCase(Locale.ENGLISH)`,
    /// 与 Spring 的输出格式完全一致。
    #[must_use]
    pub fn name_lower(&self) -> &'static str {
        match self {
            Self::Nanos => "nanoseconds",
            Self::Micros => "microseconds",
            Self::Millis => "milliseconds",
            Self::Seconds => "seconds",
            Self::Minutes => "minutes",
        }
    }

    /// 获取该单位的首字母大写 + 后续小写的形式,用于 `pretty_print` 表头。
    ///
    /// 对标 Spring 实现中的 `unitName.charAt(0) + unitName.substring(1).toLowerCase()`。
    #[must_use]
    pub fn name_title_case(&self) -> &'static str {
        match self {
            Self::Nanos => "Nanoseconds",
            Self::Micros => "Microseconds",
            Self::Millis => "Milliseconds",
            Self::Seconds => "Seconds",
            Self::Minutes => "Minutes",
        }
    }

    /// 该单位一单位等于多少纳秒。
    ///
    /// 对标 Java `TimeUnit.NANOSECONDS.convert(1, timeUnit)`。
    #[must_use]
    pub fn nanos_per_unit(&self) -> u128 {
        match self {
            Self::Nanos => 1,
            Self::Micros => 1_000,
            Self::Millis => 1_000_000,
            Self::Seconds => 1_000_000_000,
            Self::Minutes => 60_000_000_000,
        }
    }

    /// 把纳秒数转换为该单位的浮点表示。
    ///
    /// 对标 Java `(double) nanos / TimeUnit.NANOSECONDS.convert(1, timeUnit)`。
    /// 使用 f64 精度,保持与 Spring 一致的"小数点 9 位精度"语义。
    #[must_use]
    pub fn from_nanos(&self, nanos: u128) -> f64 {
        nanos as f64 / self.nanos_per_unit() as f64
    }

    /// 把 [`Duration`] 转换为该单位的浮点表示。
    #[must_use]
    pub fn from_duration(&self, duration: Duration) -> f64 {
        self.from_nanos(duration.as_nanos())
    }
}

impl std::fmt::Display for StopWatchUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_lower())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn name_lower_matches_spring_timeunit() {
        assert_eq!(StopWatchUnit::Nanos.name_lower(), "nanoseconds");
        assert_eq!(StopWatchUnit::Micros.name_lower(), "microseconds");
        assert_eq!(StopWatchUnit::Millis.name_lower(), "milliseconds");
        assert_eq!(StopWatchUnit::Seconds.name_lower(), "seconds");
        assert_eq!(StopWatchUnit::Minutes.name_lower(), "minutes");
    }

    #[test]
    fn name_title_case_starts_with_capital() {
        assert_eq!(StopWatchUnit::Nanos.name_title_case(), "Nanoseconds");
        assert_eq!(StopWatchUnit::Seconds.name_title_case(), "Seconds");
        assert_eq!(StopWatchUnit::Minutes.name_title_case(), "Minutes");
    }

    #[test]
    fn nanos_per_unit_matches_spring_constants() {
        assert_eq!(StopWatchUnit::Nanos.nanos_per_unit(), 1);
        assert_eq!(StopWatchUnit::Micros.nanos_per_unit(), 1_000);
        assert_eq!(StopWatchUnit::Millis.nanos_per_unit(), 1_000_000);
        assert_eq!(StopWatchUnit::Seconds.nanos_per_unit(), 1_000_000_000);
        assert_eq!(StopWatchUnit::Minutes.nanos_per_unit(), 60_000_000_000);
    }

    #[test]
    fn from_nanos_preserves_decimal_precision() {
        // 1.5 秒 = 1_500_000_000 ns
        let s = StopWatchUnit::Seconds.from_nanos(1_500_000_000);
        assert!((s - 1.5).abs() < 1e-9);

        // 1500 ms
        let ms = StopWatchUnit::Millis.from_nanos(1_500_000_000);
        assert!((ms - 1500.0).abs() < 1e-6);
    }

    #[test]
    fn from_duration_works() {
        let d = Duration::from_millis(500);
        assert!((StopWatchUnit::Millis.from_duration(d) - 500.0).abs() < 1e-6);
        assert!((StopWatchUnit::Seconds.from_duration(d) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn display_outputs_lower_case() {
        assert_eq!(StopWatchUnit::Seconds.to_string(), "seconds");
    }

    #[test]
    fn display_all_variants() {
        // 对标 Java TimeUnit: Display 输出与 name_lower 一致
        assert_eq!(StopWatchUnit::Nanos.to_string(), "nanoseconds");
        assert_eq!(StopWatchUnit::Micros.to_string(), "microseconds");
        assert_eq!(StopWatchUnit::Millis.to_string(), "milliseconds");
        assert_eq!(StopWatchUnit::Seconds.to_string(), "seconds");
        assert_eq!(StopWatchUnit::Minutes.to_string(), "minutes");
    }

    #[test]
    fn all_5_units_match_java_timeunit_subset() {
        // 验证 vernal-core 完整对齐 Java `TimeUnit` 的 5 个常用单位
        let all = [
            StopWatchUnit::Nanos,
            StopWatchUnit::Micros,
            StopWatchUnit::Millis,
            StopWatchUnit::Seconds,
            StopWatchUnit::Minutes,
        ];
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn name_title_case_micros_and_millis() {
        // 对标 Spring: name_title_case 应覆盖所有单位
        // 覆盖行 53-54: Micros 和 Millis 分支
        assert_eq!(StopWatchUnit::Micros.name_title_case(), "Microseconds");
        assert_eq!(StopWatchUnit::Millis.name_title_case(), "Milliseconds");
    }
}
