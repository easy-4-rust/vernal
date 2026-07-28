//! 字符串转换错误。
//!
//! 对标 Spring `org.springframework.core.convert.ConversionException`。

/// 字符串转换错误。
///
/// 对应 Java: org.springframework.core.convert.ConversionException
#[derive(Debug, Clone)]
pub struct ConversionError {
    /// 原始值
    pub value: String,
    /// 目标类型名
    pub target_type: &'static str,
    /// 错误原因
    pub reason: String,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "无法将 \"{}\" 转换为 {}: {}",
            self.value, self.target_type, self.reason
        )
    }
}

impl std::error::Error for ConversionError {}
