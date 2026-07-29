//! BeanMetadataAttributeAccessor — Spring 风格的 Bean 元数据属性访问器。
//!
//! 对应 Java 类：`org.springframework.beans.BeanMetadataAttributeAccessor`。
//!
//! 通过 `HashMap<String, BeanMetadataAttribute>` 存储和检索具名属性。

use std::any::Any;
use std::collections::HashMap;

use crate::bean_metadata_attribute::BeanMetadataAttribute;
use crate::bean_metadata_element::BeanMetadataElement;

/// Spring 风格的 Bean 元数据属性访问器。
///
/// 对应 Spring 的 `BeanMetadataAttributeAccessor`。
///
/// 提供以字符串为键的元数据属性存储和检索能力。
/// 通常作为组合字段嵌入到其他类型中，而非通过继承使用。
#[derive(Debug, Default)]
pub struct BeanMetadataAttributeAccessor {
    /// 内部属性存储。
    attributes: HashMap<String, BeanMetadataAttribute>,
}

impl BeanMetadataAttributeAccessor {
    /// 创建空的 BeanMetadataAttributeAccessor。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置指定名称的属性值。
    ///
    /// 如果已存在同名属性，则更新其值；否则创建新属性。
    pub fn set_attribute(&mut self, name: impl Into<String>, value: Box<dyn Any + Send>) {
        let name = name.into();
        if let Some(attr) = self.attributes.get_mut(&name) {
            attr.set_value(value);
        } else {
            self.attributes
                .insert(name.clone(), BeanMetadataAttribute::with_value(name, value));
        }
    }

    /// 获取指定名称的属性值。
    pub fn get_attribute(&self, name: &str) -> Option<&dyn Any> {
        self.attributes.get(name).and_then(|attr| attr.value())
    }

    /// 获取指定名称的 BeanMetadataAttribute。
    pub fn get_metadata_attribute(&self, name: &str) -> Option<&BeanMetadataAttribute> {
        self.attributes.get(name)
    }

    /// 删除指定名称的属性。
    ///
    /// 返回被删除的属性，如果不存在则返回 `None`。
    pub fn remove_attribute(&mut self, name: &str) -> Option<BeanMetadataAttribute> {
        self.attributes.remove(name)
    }

    /// 检查是否存在指定名称的属性。
    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    /// 获取所有属性名称。
    pub fn attribute_names(&self) -> Vec<&str> {
        self.attributes.keys().map(|s| s.as_str()).collect()
    }

    /// 清空所有属性。
    pub fn clear(&mut self) {
        self.attributes.clear();
    }

    /// 属性数量。
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

impl BeanMetadataElement for BeanMetadataAttributeAccessor {
    fn get_source(&self) -> Option<&dyn Any> {
        None
    }
}
