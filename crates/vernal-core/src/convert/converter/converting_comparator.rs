//! 转换比较器。
//!
//! 对标 Spring `org.springframework.core.convert.converter.ConvertingComparator`。

use std::cmp::Ordering;

use crate::convert::ConversionError;

/// 转换比较器。
///
/// 对应 Java: org.springframework.core.convert.converter.ConvertingComparator
///
/// Spring 语义：先把比较对象通过转换器映射为可比较类型，再按映射结果比较
/// （对标 `Comparator<T> { Comparable<R> convert(T) }`）。
pub struct ConvertingComparator<S: ?Sized, R, F>
where
    F: Fn(&S) -> Result<R, ConversionError>,
{
    converter: F,
    _marker: std::marker::PhantomData<fn(&S) -> R>,
}

impl<S: ?Sized, R, F> ConvertingComparator<S, R, F>
where
    F: Fn(&S) -> Result<R, ConversionError>,
{
    /// 创建比较器。
    ///
    /// # 参数
    ///
    /// - `converter`: 把比较对象转换为可比较类型的函数
    pub fn new(converter: F) -> Self {
        Self {
            converter,
            _marker: std::marker::PhantomData,
        }
    }
}

fn compare_keys<R>(left: &R, right: &R) -> Ordering
where
    R: PartialOrd,
{
    left.partial_cmp(right).unwrap_or(Ordering::Equal)
}

impl<S: ?Sized, R, F> ConvertingComparator<S, R, F>
where
    R: PartialOrd,
    F: Fn(&S) -> Result<R, ConversionError>,
{
    /// 直接比较两个对象（映射结果按 `PartialOrd` 比较）。
    #[must_use]
    pub fn compare(&self, left: &S, right: &S) -> Ordering {
        let convert = &self.converter;
        match convert(left).and_then(|l| convert(right).map(|r| (l, r))) {
            Ok((l, r)) => compare_keys(&l, &r),
            Err(_) => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_after_conversion() {
        // A 类（合同对齐）：对标 Spring 先转换后比较
        let comparator = ConvertingComparator::new(|s: &str| {
            s.parse::<i64>().map_err(|e| ConversionError {
                value: s.to_string(),
                target_type: "i64",
                reason: e.to_string(),
            })
        });
        assert_eq!(comparator.compare("10", "2"), Ordering::Greater);
        assert_eq!(comparator.compare("2", "10"), Ordering::Less);
        assert_eq!(comparator.compare("7", "7"), Ordering::Equal);
    }

    #[test]
    fn conversion_failure_treats_as_equal() {
        // C 类（错误路径）：转换失败时保持稳定顺序
        let comparator = ConvertingComparator::new(|s: &str| {
            s.parse::<i64>().map_err(|e| ConversionError {
                value: s.to_string(),
                target_type: "i64",
                reason: e.to_string(),
            })
        });
        assert_eq!(comparator.compare("x", "10"), Ordering::Equal);
    }

    #[test]
    fn unconvertible_values_treated_equal_in_sort() {
        // C 类（错误路径）：转换失败保持稳定顺序（sort 场景）
        let comparator = ConvertingComparator::new(|s: &str| {
            s.parse::<i64>().map_err(|e| ConversionError {
                value: s.to_string(),
                target_type: "i64",
                reason: e.to_string(),
            })
        });
        let mut values = vec!["x", "10"];
        values.sort_by(|a, b| comparator.compare(a, b));
        assert_eq!(values, vec!["x", "10"]);
    }

    #[test]
    fn sorts_with_comparator() {
        // B 类（边界行为）：可配合 `sort_by` 使用
        let comparator = ConvertingComparator::new(|s: &str| {
            s.parse::<i64>().map_err(|e| ConversionError {
                value: s.to_string(),
                target_type: "i64",
                reason: e.to_string(),
            })
        });
        let mut values = vec!["10", "2", "1"];
        values.sort_by(|a, b| comparator.compare(a, b));
        assert_eq!(values, vec!["1", "2", "10"]);
    }
}
