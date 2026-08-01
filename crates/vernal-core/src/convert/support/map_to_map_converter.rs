//! 映射 → 映射转换器。
//!
//! 对标 Spring `org.springframework.core.convert.support.MapToMapConverter`。

use std::collections::HashMap;

use crate::convert::{ConversionError, Convertible, Converter};

/// 映射 → 映射转换器。
///
/// 对应 Java: org.springframework.core.convert.support.MapToMapConverter
///
/// Spring 语义：键与值分别逐项转换到目标类型。
pub struct MapToMapConverter;

impl<K: ToString, V: ToString, T: Convertible + Eq + std::hash::Hash, U: Convertible>
    Converter<&HashMap<K, V>, HashMap<T, U>> for MapToMapConverter
{
    fn convert(&self, source: &HashMap<K, V>) -> Result<HashMap<T, U>, ConversionError> {
        let mut result = HashMap::with_capacity(source.len());
        for (key, value) in source {
            let converted_key = T::from_str_value(&key.to_string())?;
            let converted_value = U::from_str_value(&value.to_string())?;
            result.insert(converted_key, converted_value);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_keys_and_values() {
        // A 类（合同对齐）：对标 Spring 键值逐项转换
        let converter = MapToMapConverter;
        let source: HashMap<String, String> =
            [("1".to_string(), "10".to_string()), ("2".to_string(), "20".to_string())]
                .into_iter()
                .collect();
        let output: HashMap<i32, i32> = converter.convert(&source).unwrap();
        assert_eq!(output.get(&1), Some(&10));
        assert_eq!(output.get(&2), Some(&20));
    }

    #[test]
    fn value_failure_propagates() {
        // C 类（错误路径）
        let converter = MapToMapConverter;
        let source: HashMap<String, String> =
            [("1".to_string(), "bad".to_string())].into_iter().collect();
        let result: Result<HashMap<i32, i32>, _> = converter.convert(&source);
        assert!(result.is_err());
    }
}
