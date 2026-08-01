//! 属性加载支持基类。
//!
//! 对标 Spring `org.springframework.core.io.support.PropertiesLoaderSupport`。

use std::collections::BTreeMap;

use crate::io::Resource;

use super::PropertiesLoaderUtils;

/// 属性加载支持基类。
///
/// 对应 Java: org.springframework.core.io.support.PropertiesLoaderSupport
///
/// Spring 语义：持有位置列表并加载合并属性；Rust 中为值对象形态。
#[derive(Default)]
pub struct PropertiesLoaderSupport {
    locations: Vec<Box<dyn Resource>>,
    local_override: bool,
}

impl PropertiesLoaderSupport {
    /// 创建空的加载支持对象。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置资源位置列表。
    pub fn set_locations(&mut self, locations: Vec<Box<dyn Resource>>) {
        self.locations = locations;
    }

    /// 设置本地覆盖标志（后加载覆盖先加载）。
    pub fn set_local_override(&mut self, local_override: bool) {
        self.local_override = local_override;
    }

    /// 加载并合并所有位置的属性。
    ///
    /// # 错误
    ///
    /// 任一位置读取失败时返回 [`std::io::Error`]。
    pub fn merge_properties(&self) -> std::io::Result<BTreeMap<String, String>> {
        let mut merged = BTreeMap::new();
        if self.local_override {
            for resource in &self.locations {
                let props = PropertiesLoaderUtils::load_properties(resource.as_ref())?;
                merged.extend(props);
            }
        } else {
            // 默认语义: 后加载者覆盖（Spring 的 ignoreResourceNotFound 不处理时一致）
            for resource in &self.locations {
                let props = PropertiesLoaderUtils::load_properties(resource.as_ref())?;
                merged.extend(props);
            }
        }
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::ByteArrayResource;

    #[test]
    fn merges_locations_in_order() {
        // A 类（合同对齐）：对标 Spring `mergeProperties`
        let mut support = PropertiesLoaderSupport::new();
        support.set_locations(vec![
            Box::new(ByteArrayResource::new(b"k=1\n".to_vec())),
            Box::new(ByteArrayResource::new(b"k=2\nj=9\n".to_vec())),
        ]);
        let props = support.merge_properties().unwrap();
        assert_eq!(props.get("k").map(String::as_str), Some("2"));
        assert_eq!(props.get("j").map(String::as_str), Some("9"));
    }

    #[test]
    fn local_override_flag_keeps_semantics() {
        // D 类（生命周期）：标志位不改变覆盖顺序（对标 Spring 语义）
        let mut support = PropertiesLoaderSupport::new();
        support.set_local_override(true);
        support.set_locations(vec![
            Box::new(ByteArrayResource::new(b"k=1\n".to_vec())),
            Box::new(ByteArrayResource::new(b"k=3\n".to_vec())),
        ]);
        let props = support.merge_properties().unwrap();
        assert_eq!(props.get("k").map(String::as_str), Some("3"));
    }
}
