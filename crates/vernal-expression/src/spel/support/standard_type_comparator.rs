//! 标准类型比较器。
//!
//! 对标 Spring 的 `StandardTypeComparator`。

use crate::type_comparator::TypeComparator;
use crate::typed_value::TypedValue;

/// 标准类型比较器。
///
/// 对标 Spring 的 `org.springframework.expression.spel.support.StandardTypeComparator`。
pub struct StandardTypeComparator;

impl StandardTypeComparator {
    /// 单例实例。
    pub const INSTANCE: Self = Self;
}

impl TypeComparator for StandardTypeComparator {
    fn can_compare(&self, _left: &TypedValue, _right: &TypedValue) -> bool {
        true
    }

    fn compare(&self, left: &TypedValue, right: &TypedValue) -> Result<std::cmp::Ordering, String> {
        let l = left.value();
        let r = right.value();
        let ordering = match (l, r) {
            (
                crate::typed_value::ExpressionValue::Int(a),
                crate::typed_value::ExpressionValue::Int(b),
            ) => a.cmp(b),
            (
                crate::typed_value::ExpressionValue::Float(a),
                crate::typed_value::ExpressionValue::Float(b),
            ) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
            (
                crate::typed_value::ExpressionValue::Int(a),
                crate::typed_value::ExpressionValue::Float(b),
            ) => (*a as f64)
                .partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal),
            (
                crate::typed_value::ExpressionValue::Float(a),
                crate::typed_value::ExpressionValue::Int(b),
            ) => a
                .partial_cmp(&(*b as f64))
                .unwrap_or(std::cmp::Ordering::Equal),
            (
                crate::typed_value::ExpressionValue::String(a),
                crate::typed_value::ExpressionValue::String(b),
            ) => a.cmp(b),
            (
                crate::typed_value::ExpressionValue::Boolean(a),
                crate::typed_value::ExpressionValue::Boolean(b),
            ) => a.cmp(b),
            _ => return Err("不支持的类型比较".to_string()),
        };
        Ok(ordering)
    }
}
