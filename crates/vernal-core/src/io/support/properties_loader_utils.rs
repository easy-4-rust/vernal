//! 属性加载工具。
//!
//! 对标 Spring `org.springframework.core.io.support.PropertiesLoaderUtils`。

use std::collections::BTreeMap;

use crate::io::Resource;
use crate::properties_file::parse_properties;

/// 属性加载工具（静态辅助函数）。
///
/// 对应 Java: org.springframework.core.io.support.PropertiesLoaderUtils
pub struct PropertiesLoaderUtils;

impl PropertiesLoaderUtils {
    /// 从资源加载属性表。
    ///
    /// 对标 Spring `loadProperties(Resource)`。
    ///
    /// # 错误
    ///
    /// 资源读取失败时返回 [`std::io::Error`]。
    pub fn load_properties(resource: &dyn Resource) -> std::io::Result<BTreeMap<String, String>> {
        let content = resource.read_string()?;
        let mut flat = std::collections::HashMap::new();
        parse_properties(&content, &mut flat);
        Ok(flat.into_iter().collect())
    }

    /// 从多个位置加载并合并属性（后者覆盖前者）。
    ///
    /// 对标 Spring `fillProperties` 语义。
    ///
    /// # 错误
    ///
    /// 任一资源读取失败时返回 [`std::io::Error`]。
    pub fn load_all_properties(
        resources: &[Box<dyn Resource>],
    ) -> std::io::Result<BTreeMap<String, String>> {
        let mut merged = BTreeMap::new();
        for resource in resources {
            let props = Self::load_properties(resource.as_ref())?;
            merged.extend(props);
        }
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn loads_properties_from_resource() {
        // A 类（合同对齐）：对标 Spring `loadProperties(Resource)`
        let resource = ByteArrayResource::new(b"a=1\nb=2\n".to_vec());
        let props = PropertiesLoaderUtils::load_properties(&resource).unwrap();
        assert_eq!(props.get("a").map(String::as_str), Some("1"));
        assert_eq!(props.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn merge_later_wins() {
        // B 类（边界行为）：对标 Spring 多位置合并覆盖
        let first = ByteArrayResource::new(b"k=first\nonly=1\n".to_vec());
        let second = ByteArrayResource::new(b"k=second\n".to_vec());
        let merged =
            PropertiesLoaderUtils::load_all_properties(&[Box::new(first), Box::new(second)])
                .unwrap();
        assert_eq!(merged.get("k").map(String::as_str), Some("second"));
        assert_eq!(merged.get("only").map(String::as_str), Some("1"));
    }

    #[test]
    fn invalid_utf8_returns_error() {
        // C 类（错误路径）
        let resource = ByteArrayResource::new(vec![0xFF, 0xFE, 0x00]);
        assert!(PropertiesLoaderUtils::load_properties(&resource).is_err());
    }
}
