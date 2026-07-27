//! MutablePropertyValues — Spring 风格的可变属性值集合。
//!
//! 对应 Java 类：`org.springframework.beans.MutablePropertyValues`。
//!
//! 存储一组 Bean 属性的名称-值对，支持添加、修改、删除和查询。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::property_value::PropertyValue;

/// Spring 风格的可变属性值集合。
///
/// 对应 Spring 的 `MutablePropertyValues`。
///
/// 存储一组 Bean 属性的名称-值对：
/// - 支持按名称添加、修改、删除
/// - 支持按名称查询
/// - 支持批量添加
///
/// ## 与 tx_di 的关系
///
/// tx_di 使用 `Component::Deps` 关联类型在编译期声明依赖。
/// `MutablePropertyValues` 是 Spring 风格的属性集合，用于 XML/注解驱动的配置。
#[derive(Clone, Debug, Default)]
pub struct MutablePropertyValues {
    /// 属性值列表（按声明顺序）。
    property_values: Vec<PropertyValue>,
    /// 属性名称索引。
    names: HashMap<String, usize>,
}

impl MutablePropertyValues {
    /// 创建空的属性值集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 PropertyValue 列表创建。
    pub fn from_vec(values: Vec<PropertyValue>) -> Self {
        let mut pvs = Self::new();
        for value in values {
            pvs.add(value);
        }
        pvs
    }

    /// 添加属性值。
    pub fn add(&mut self, pv: PropertyValue) {
        if let Some(&idx) = self.names.get(pv.name()) {
            self.property_values[idx] = pv;
        } else {
            let idx = self.property_values.len();
            self.names.insert(pv.name().to_string(), idx);
            self.property_values.push(pv);
        }
    }

    /// 添加属性值（按名称和值）。
    pub fn add_value(&mut self, name: impl Into<String>, value: Arc<dyn Any + Send + Sync>) {
        self.add(PropertyValue::new(name, value));
    }

    /// 检查是否包含指定属性。
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }

    /// 获取属性值。
    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.names.get(name).map(|&idx| &self.property_values[idx])
    }

    /// 获取属性值列表。
    pub fn get_property_values(&self) -> &[PropertyValue] {
        &self.property_values
    }

    /// 属性数量。
    pub fn len(&self) -> usize {
        self.property_values.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.property_values.is_empty()
    }

    /// 清空所有属性。
    pub fn clear(&mut self) {
        self.property_values.clear();
        self.names.clear();
    }

    /// 批量添加属性值。
    pub fn add_property_values(&mut self, other: &Self) {
        for pv in &other.property_values {
            self.add(pv.clone());
        }
    }
}
