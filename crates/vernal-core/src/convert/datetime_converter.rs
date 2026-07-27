//! 日期时间转换器(feature = "convert-time")。
//!
//! 对标 Spring 的 `StringToInstantConverter` / `StringToLocalDateConverter`,
//! 将字符串解析为 [`time::OffsetDateTime`]。
//!
//! # 启用方式
//!
//! ```toml
//! [dependencies]
//! vernal-core = { features = ["convert-time"] }
//! ```

use super::{ConversionError, Convertible};

/// 日期时间转换器。
///
/// 对标 Spring `StringToInstantConverter`。
pub struct DatetimeConverter;

impl Convertible for time::OffsetDateTime {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        // 优先尝试 RFC 3339(对标 Spring `Instant.parse()`)
        if let Ok(dt) =
            time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
        {
            return Ok(dt);
        }
        // 后备:尝试 ISO 8601(简化)
        time::OffsetDateTime::parse(
            value,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: "OffsetDateTime",
            reason: format!("日期时间解析失败(支持 RFC 3339 / ISO 8601): {e}"),
        })
    }
}

impl Convertible for time::Date {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        time::Date::parse(
            value,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: "Date",
            reason: format!("日期解析失败: {e}"),
        })
    }
}

impl Convertible for time::Time {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        time::Time::parse(
            value,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: "Time",
            reason: format!("时间解析失败: {e}"),
        })
    }
}

impl Convertible for time::PrimitiveDateTime {
    fn from_str_value(value: &str) -> Result<Self, ConversionError> {
        time::PrimitiveDateTime::parse(
            value,
            &time::format_description::well_known::Iso8601::DEFAULT,
        )
        .map_err(|e| ConversionError {
            value: value.to_string(),
            target_type: "PrimitiveDateTime",
            reason: format!("日期时间(无时区)解析失败: {e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc3339_with_z() {
        let dt = time::OffsetDateTime::from_str_value("2026-01-01T00:00:00Z").unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), time::Month::January);
    }

    #[test]
    fn parses_rfc3339_with_offset() {
        let dt = time::OffsetDateTime::from_str_value("2026-07-27T10:30:45+08:00").unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn parses_rfc3339_with_fractional_seconds() {
        let dt = time::OffsetDateTime::from_str_value("2026-07-27T10:30:45.123456789Z").unwrap();
        assert_eq!(dt.millisecond(), 123);
    }

    #[test]
    fn rejects_empty_string() {
        let err = time::OffsetDateTime::from_str_value("").unwrap_err();
        assert_eq!(err.target_type, "OffsetDateTime");
    }

    #[test]
    fn rejects_invalid_format() {
        let err = time::OffsetDateTime::from_str_value("not-a-date").unwrap_err();
        assert_eq!(err.target_type, "OffsetDateTime");
        assert!(err.reason.contains("解析失败"));
    }

    #[test]
    fn date_parses_iso8601() {
        let d = time::Date::from_str_value("2026-07-27").unwrap();
        assert_eq!(d.year(), 2026);
    }

    #[test]
    fn time_parses_iso8601() {
        let t = time::Time::from_str_value("10:30:45").unwrap();
        assert_eq!(t.hour(), 10);
    }

    #[test]
    fn offset_datetime_via_conversion_service() {
        let dt: time::OffsetDateTime =
            super::super::ConversionService::convert("2026-01-01T00:00:00Z").unwrap();
        assert_eq!(dt.year(), 2026);
    }
}
