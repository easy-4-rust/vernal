//! 条件转换器 trait。
//!
//! 对标 Spring `org.springframework.core.convert.converter.ConditionalConverter`。

use super::convertible_pair::ConvertiblePair;

/// 条件转换器 trait。
///
/// 对应 Java: org.springframework.core.convert.converter.ConditionalConverter
pub trait ConditionalConverter: Send + Sync {
    /// 判断该转换器是否适用于给定的源/目标类型对。
    ///
    /// 对应 Java: `ConditionalConverter#matches(TypeDescriptor, TypeDescriptor)`
    fn matches(&self, _pair: &ConvertiblePair) -> bool;
}
