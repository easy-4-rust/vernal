//! 标准类型比较器（对标 Spring `StandardTypeComparator`）。
//!
//! 对标 Java `org.springframework.expression.spel.support.StandardTypeComparator`。
//! 支持跨类型数字 widening 比较和字符串字典序比较。

use crate::type_comparator::TypeComparator;
use crate::typed_value::{ExpressionValue, TypedValue};
use std::cmp::Ordering;

/// 标准类型比较器（对标 Spring `StandardTypeComparator`）。
///
/// 支持的比较：
/// - 同类型直接比较（Int↔Int、String↔String 等）
/// - 数字 widening 比较（Int↔Float、Long↔Double 等）
/// - 字符串字典序比较
/// - Boolean 比较（false < true）
pub struct StandardTypeComparator;

impl StandardTypeComparator {
    /// 单例实例。
    pub const INSTANCE: Self = Self;

    /// 将 ExpressionValue 转换为可比较的 f64（数字 widening）。
    fn to_f64(val: &ExpressionValue) -> Option<f64> {
        match val {
            ExpressionValue::Int(i) => Some(*i as f64),
            ExpressionValue::Long(l) => Some(*l as f64),
            ExpressionValue::Float(f) => Some(*f),
            ExpressionValue::Double(d) => Some(*d),
            ExpressionValue::BigInt(b) => b.to_f64(),
            ExpressionValue::Decimal(d) => d.to_f64(),
            ExpressionValue::Char(c) => Some(*c as u32 as f64),
            _ => None,
        }
    }

    /// 是否为数字类型。
    fn is_number(val: &ExpressionValue) -> bool {
        matches!(
            val,
            ExpressionValue::Int(_)
                | ExpressionValue::Long(_)
                | ExpressionValue::Float(_)
                | ExpressionValue::Double(_)
                | ExpressionValue::BigInt(_)
                | ExpressionValue::Decimal(_)
                | ExpressionValue::Char(_)
        )
    }
}

impl TypeComparator for StandardTypeComparator {
    fn can_compare(&self, left: &TypedValue, right: &TypedValue) -> bool {
        // 数字 ↔ 数字
        if Self::is_number(left.value()) && Self::is_number(right.value()) {
            return true;
        }
        // 同类型
        if std::mem::discriminant(left.value()) == std::mem::discriminant(right.value()) {
            return true;
        }
        // 都是字符串
        matches!(
            (left.value(), right.value()),
            (ExpressionValue::String(_), ExpressionValue::String(_))
        )
    }

    fn compare(&self, left: &TypedValue, right: &TypedValue) -> Result<Ordering, String> {
        let l = left.value();
        let r = right.value();

        // 数字类型 widening 比较
        if Self::is_number(l) && Self::is_number(r) {
            let wide_f64_l = Self::to_f64(l).unwrap_or(0.0);
            let wide_f64_r = Self::to_f64(r).unwrap_or(0.0);
            return Ok(wide_f64_l
                .partial_cmp(&wide_f64_r)
                .unwrap_or(Ordering::Equal));
        }

        // 字符串字典序
        if let (ExpressionValue::String(a), ExpressionValue::String(b)) = (l, r) {
            return Ok(a.cmp(b));
        }

        // Boolean: false < true
        if let (ExpressionValue::Boolean(a), ExpressionValue::Boolean(b)) = (l, r) {
            return Ok(a.cmp(b));
        }

        // 不支持的类型组合
        Err("NOT_COMPARABLE: cannot compare instances of these types".to_string())
    }
}

use num_traits::ToPrimitive;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_descriptor::TypeDescriptor;

    fn make_int(v: i64) -> TypedValue {
        TypedValue::new(ExpressionValue::Int(v), TypeDescriptor::INT)
    }

    fn make_float(v: f64) -> TypedValue {
        TypedValue::new(ExpressionValue::Float(v), TypeDescriptor::FLOAT)
    }

    fn make_string(s: &str) -> TypedValue {
        TypedValue::new(
            ExpressionValue::String(s.to_string()),
            TypeDescriptor::STRING,
        )
    }

    fn make_bool(b: bool) -> TypedValue {
        TypedValue::new(ExpressionValue::Boolean(b), TypeDescriptor::BOOLEAN)
    }

    #[test]
    fn same_type_int() {
        assert_eq!(
            StandardTypeComparator
                .compare(&make_int(3), &make_int(5))
                .unwrap(),
            Ordering::Less
        );
        assert_eq!(
            StandardTypeComparator
                .compare(&make_int(5), &make_int(5))
                .unwrap(),
            Ordering::Equal
        );
        assert_eq!(
            StandardTypeComparator
                .compare(&make_int(7), &make_int(3))
                .unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn mixed_int_float() {
        assert_eq!(
            StandardTypeComparator
                .compare(&make_int(3), &make_float(5.0))
                .unwrap(),
            Ordering::Less
        );
        assert_eq!(
            StandardTypeComparator
                .compare(&make_float(5.0), &make_int(3))
                .unwrap(),
            Ordering::Greater
        );
    }

    #[test]
    fn strings() {
        assert_eq!(
            StandardTypeComparator
                .compare(&make_string("abc"), &make_string("def"))
                .unwrap(),
            Ordering::Less
        );
        assert_eq!(
            StandardTypeComparator
                .compare(&make_string("abc"), &make_string("abc"))
                .unwrap(),
            Ordering::Equal
        );
    }

    #[test]
    fn booleans() {
        assert_eq!(
            StandardTypeComparator
                .compare(&make_bool(false), &make_bool(true))
                .unwrap(),
            Ordering::Less
        );
    }

    #[test]
    fn can_compare_numbers() {
        assert!(StandardTypeComparator.can_compare(&make_int(1), &make_float(2.0)));
        assert!(StandardTypeComparator.can_compare(&make_int(1), &make_int(2)));
        assert!(!StandardTypeComparator.can_compare(&make_string("a"), &make_int(1)));
    }
}
