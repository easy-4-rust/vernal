//! 时间间隔转换器。
//!
//! 对标 Spring 的 `StringToDurationConverter`（`org.springframework.core.convert.support`）。
//! 支持 ISO-8601 风格的 `PT1H30M` 表示法,以及简化语法 `30s` / `5m` / `1h` / `1d`。
//!
//! 仅依赖 std,无需任何外部 crate。

use std::time::Duration;

use super::{ConversionError, Convertible};

/// 时间间隔转换器。
///
/// 将字符串解析为 [`std::time::Duration`]。
pub struct DurationConverter;

impl Convertible for Duration {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ConversionError {
                value: value.to_string(),
                target_type: "Duration",
                reason: "时间间隔字符串不能为空".to_string(),
            });
        }

        // 优先尝试 ISO-8601 格式 (PT1H30M, PT45S, P1DT2H)
        if let Some(d) = parse_iso8601(trimmed) {
            return Ok(d);
        }

        // 后备:简化语法 (30s, 5m, 1h, 1d)
        parse_simplified(trimmed).ok_or_else(|| ConversionError {
            value: value.to_string(),
            target_type: "Duration",
            reason: format!(
                "无法解析时间间隔: '{value}' (支持 ISO-8601 如 'PT1H30M' 或简化语法如 '30s'/'5m'/'1h'/'1d')"
            ),
        })
    }
}

/// 解析 ISO-8601 Duration 格式(简化子集)
///
/// 支持:
/// - `PT<n>S` (秒)
/// - `PT<n>M` (分钟)
/// - `PT<n>H` (小时)
/// - `PT<n>D` (天,作为小时处理 = n * 24)
/// - `P<n>D` (天,仅日期部分)
/// - 组合: `PT1H30M`、`PT2H30M45S`
fn parse_iso8601(s: &str) -> Option<Duration> {
    if !s.starts_with('P') {
        return None;
    }

    // 简化:去除 'P' 前缀
    let body = &s[1..];
    let mut total_secs: u64 = 0;
    let mut current_num = String::new();

    for c in body.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if c == 'T' {
            // T 之后是时间部分
            current_num.clear();
            continue;
        } else {
            if current_num.is_empty() {
                return None;
            }
            let n: u64 = current_num.parse().ok()?;
            current_num.clear();
            // 区分 'T' 之前(D/W/M/Y)和之后(H/M/S)
            let is_time_part = body.starts_with('T') || body.find('T').is_some();
            if c == 'D' {
                if is_time_part {
                    return None;
                }
                total_secs += n * 24 * 3600;
            } else if c == 'H' {
                total_secs += n * 3600;
            } else if c == 'M' {
                total_secs += n * 60;
            } else if c == 'S' {
                total_secs += n;
            } else {
                return None;
            }
        }
    }
    if current_num.is_empty() && total_secs > 0 {
        Some(Duration::from_secs(total_secs))
    } else {
        None
    }
}

/// 解析简化语法:30s / 5m / 1h / 1d
fn parse_simplified(s: &str) -> Option<Duration> {
    let (num_str, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()));
    if num_str.is_empty() || unit.is_empty() {
        return None;
    }
    let n: u64 = num_str.parse().ok()?;
    match unit {
        "s" => Some(Duration::from_secs(n)),
        "m" => Some(Duration::from_secs(n * 60)),
        "h" => Some(Duration::from_secs(n * 3600)),
        "d" => Some(Duration::from_secs(n * 24 * 3600)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn parses_iso8601_seconds() {
        let d = Duration::from_str_value("PT45S").unwrap();
        assert_eq!(d, Duration::from_secs(45));
    }

    #[test]
    fn parses_iso8601_minutes() {
        let d = Duration::from_str_value("PT30M").unwrap();
        assert_eq!(d, Duration::from_secs(30 * 60));
    }

    #[test]
    fn parses_iso8601_hours() {
        let d = Duration::from_str_value("PT2H").unwrap();
        assert_eq!(d, Duration::from_secs(2 * 3600));
    }

    #[test]
    fn parses_iso8601_combined() {
        let d = Duration::from_str_value("PT1H30M").unwrap();
        assert_eq!(d, Duration::from_secs(3600 + 30 * 60));
    }

    #[test]
    fn parses_simplified_seconds() {
        let d = Duration::from_str_value("30s").unwrap();
        assert_eq!(d, Duration::from_secs(30));
    }

    #[test]
    fn parses_simplified_minutes() {
        let d = Duration::from_str_value("5m").unwrap();
        assert_eq!(d, Duration::from_secs(300));
    }

    #[test]
    fn parses_simplified_hours() {
        let d = Duration::from_str_value("2h").unwrap();
        assert_eq!(d, Duration::from_secs(2 * 3600));
    }

    #[test]
    fn parses_simplified_days() {
        let d = Duration::from_str_value("1d").unwrap();
        assert_eq!(d, Duration::from_secs(86400));
    }

    #[test]
    fn rejects_empty_string() {
        let err = Duration::from_str_value("").unwrap_err();
        assert_eq!(err.target_type, "Duration");
    }

    #[test]
    fn rejects_invalid_format() {
        let err = Duration::from_str_value("not-a-duration").unwrap_err();
        assert_eq!(err.target_type, "Duration");
        assert!(err.reason.contains("无法解析"));
    }

    #[test]
    fn trims_whitespace() {
        let d = Duration::from_str_value("  30s  ").unwrap();
        assert_eq!(d, Duration::from_secs(30));
    }

    #[test]
    fn duration_conversion_via_conversion_service() {
        let d: Duration = super::super::ConversionService::convert("1h").unwrap();
        assert_eq!(d, Duration::from_secs(3600));
    }

    #[test]
    fn parses_iso8601_days_in_date_part() {
        // 对标 ISO-8601 P<n>D 中日期段的 D
        let d = Duration::from_str_value("P2D").unwrap();
        assert_eq!(d, Duration::from_secs(2 * 24 * 3600));
    }

    #[test]
    fn rejects_iso8601_days_in_time_part() {
        // ISO-8601 规定: T 之后的 D 不合法 (D 仅用于日期段)
        let err = Duration::from_str_value("PT1D").unwrap_err();
        assert_eq!(err.target_type, "Duration");
    }

    #[test]
    fn rejects_iso8601_unknown_suffix() {
        // 对标 Spring `StringToDurationConverter` 拒绝未知单位
        let err = Duration::from_str_value("P1X").unwrap_err();
        assert_eq!(err.target_type, "Duration");
    }

    #[test]
    fn rejects_iso8601_without_digit_before_suffix() {
        // P 后紧接非数字字符 → parser 返回 None (current_num 空)
        let err = Duration::from_str_value("P").unwrap_err();
        assert_eq!(err.target_type, "Duration");
    }

    #[test]
    fn rejects_simplified_unknown_unit() {
        // 对标 Spring 简化语法只接受 s/m/h/d
        let err = Duration::from_str_value("5y").unwrap_err();
        assert_eq!(err.target_type, "Duration");
    }

    #[test]
    fn rejects_simplified_without_unit() {
        // 纯数字无单位: parse_simplified 返回 None
        let err = Duration::from_str_value("42").unwrap_err();
        assert_eq!(err.target_type, "Duration");
    }
}
