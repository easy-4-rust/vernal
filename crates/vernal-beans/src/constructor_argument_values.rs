//! ConstructorArgumentValues — Spring 风格的构造参数值集合。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.ConstructorArgumentValues`。
//!
//! 存储 Bean 构造器的参数值，支持按索引和按类型匹配。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// 构造参数值持有者。
///
/// 对应 Spring 的 `ConstructorArgumentValues.ValueHolder`。
///
/// 存储一个构造器参数的值、类型和名称。
#[derive(Clone, Debug)]
pub struct ValueHolder {
    /// 参数值。
    value: Option<Arc<dyn Any + Send + Sync>>,
    /// 参数类型名。
    type_name: Option<String>,
    /// 参数名称。
    name: Option<String>,
    /// 是否已转换。
    converted: bool,
    /// 转换后的值。
    converted_value: Option<Arc<dyn Any + Send + Sync>>,
}

impl ValueHolder {
    /// 创建新的 ValueHolder。
    pub fn new(value: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            value: Some(value),
            type_name: None,
            name: None,
            converted: false,
            converted_value: None,
        }
    }

    /// 创建带类型的 ValueHolder。
    pub fn with_type(value: Arc<dyn Any + Send + Sync>, type_name: impl Into<String>) -> Self {
        Self {
            value: Some(value),
            type_name: Some(type_name.into()),
            name: None,
            converted: false,
            converted_value: None,
        }
    }

    /// 创建带类型和名称的 ValueHolder。
    pub fn with_type_and_name(
        value: Arc<dyn Any + Send + Sync>,
        type_name: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            value: Some(value),
            type_name: Some(type_name.into()),
            name: Some(name.into()),
            converted: false,
            converted_value: None,
        }
    }

    /// 获取参数值。
    pub fn value(&self) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.value.as_ref()
    }

    /// 获取参数类型名。
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }

    /// 获取参数名称。
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// 检查是否已转换。
    pub fn is_converted(&self) -> bool {
        self.converted
    }

    /// 设置转换后的值。
    pub fn set_converted_value(&mut self, value: Arc<dyn Any + Send + Sync>) {
        self.converted_value = Some(value);
        self.converted = true;
    }

    /// 获取转换后的值。
    pub fn converted_value(&self) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.converted_value.as_ref()
    }

    /// 创建副本。
    pub fn copy(&self) -> Self {
        Self {
            value: self.value.clone(),
            type_name: self.type_name.clone(),
            name: self.name.clone(),
            converted: self.converted,
            converted_value: self.converted_value.clone(),
        }
    }
}

/// Spring 风格的构造参数值集合。
///
/// 对应 Spring 的 `ConstructorArgumentValues`。
///
/// 存储 Bean 构造器的参数值，支持：
/// - 按索引匹配（`add_indexed_argument_value`）
/// - 按类型匹配（`add_generic_argument_value`）
/// - 按名称匹配
///
/// ## 使用场景
///
/// 当容器需要通过构造器创建 Bean 时，使用此集合匹配构造器参数。
#[derive(Clone, Debug, Default)]
pub struct ConstructorArgumentValues {
    /// 按索引存储的参数值。
    indexed_argument_values: HashMap<i32, ValueHolder>,
    /// 按类型存储的参数值（通用参数）。
    generic_argument_values: Vec<ValueHolder>,
}

impl ConstructorArgumentValues {
    /// 创建空的构造参数值集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// 从另一个集合复制。
    pub fn from_other(original: &Self) -> Self {
        Self {
            indexed_argument_values: original.indexed_argument_values.clone(),
            generic_argument_values: original.generic_argument_values.clone(),
        }
    }

    // ── Indexed arguments ──────────────────────────────────────────────

    /// 按索引添加参数值。
    pub fn add_indexed_argument_value(&mut self, index: i32, value: ValueHolder) {
        self.indexed_argument_values.insert(index, value);
    }

    /// 检查是否有指定索引的参数。
    pub fn has_indexed_argument_value(&self, index: i32) -> bool {
        self.indexed_argument_values.contains_key(&index)
    }

    /// 按索引获取参数。
    pub fn get_indexed_argument_value(&self, index: i32) -> Option<&ValueHolder> {
        self.indexed_argument_values.get(&index)
    }

    /// 获取所有索引参数。
    pub fn indexed_argument_values(&self) -> &HashMap<i32, ValueHolder> {
        &self.indexed_argument_values
    }

    // ── Generic arguments ──────────────────────────────────────────────

    /// 添加通用参数值。
    pub fn add_generic_argument_value(&mut self, value: ValueHolder) {
        self.generic_argument_values.push(value);
    }

    /// 按类型获取通用参数。
    pub fn get_generic_argument_value(&self, type_name: &str) -> Option<&ValueHolder> {
        self.generic_argument_values
            .iter()
            .find(|vh| vh.type_name() == Some(type_name))
    }

    /// 获取所有通用参数。
    pub fn generic_argument_values(&self) -> &[ValueHolder] {
        &self.generic_argument_values
    }

    // ── Combined query ─────────────────────────────────────────────────

    /// 按索引和类型获取参数。
    pub fn get_argument_value(
        &self,
        index: i32,
        type_name: Option<&str>,
        name: Option<&str>,
    ) -> Option<ValueHolder> {
        // 优先按索引查找
        if let Some(vh) = self.indexed_argument_values.get(&index) {
            return Some(vh.clone());
        }

        // 然后在通用参数中查找
        self.generic_argument_values
            .iter()
            .find(|vh| {
                let type_match = type_name.map(|t| vh.type_name() == Some(t)).unwrap_or(true);
                let name_match = name.map(|n| vh.name() == Some(n)).unwrap_or(true);
                type_match && name_match
            })
            .cloned()
    }

    /// 检查是否包含命名参数。
    pub fn contains_named_argument(&self) -> bool {
        self.generic_argument_values
            .iter()
            .any(|vh| vh.name().is_some())
    }

    /// 参数总数。
    pub fn argument_count(&self) -> usize {
        self.indexed_argument_values.len() + self.generic_argument_values.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.indexed_argument_values.is_empty() && self.generic_argument_values.is_empty()
    }

    /// 清空所有参数。
    pub fn clear(&mut self) {
        self.indexed_argument_values.clear();
        self.generic_argument_values.clear();
    }

    /// 批量添加参数值。
    pub fn add_argument_values(&mut self, other: &Self) {
        for (index, vh) in &other.indexed_argument_values {
            self.indexed_argument_values.insert(*index, vh.clone());
        }
        self.generic_argument_values
            .extend(other.generic_argument_values.iter().cloned());
    }
}
