//! 对应 Java 类：`org.springframework.context.index.processor.CandidateComponentsMetadata`
//!
//! 候选组件元数据容器。
//!
//! @author Stephane Nicoll
//! @since 5.0

use std::collections::BTreeSet;

use crate::index::item_metadata::ItemMetadata;

/// 候选组件元数据容器。
///
/// 对应 Spring `org.springframework.context.index.processor.CandidateComponentsMetadata`：
/// - 内部维护 `List<ItemMetadata>`，按插入顺序保存所有条目
/// - `add(ItemMetadata)` 追加条目
/// - `getItems()` 返回只读视图
/// - `toString()` 返回 `"CandidateComponentsMetadata{items=[...]}"` 格式
///
/// 与 vernal 的关系：
/// - [`crate::LinkedComponentIndex`] 内部 `Vec<LinkedComponentEntry>` 对应 Spring 的 `items` 列表
/// - vernal 不直接用 `CandidateComponentsMetadata`，但保留完整语义以镜像 Spring `processor/` 子模块
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CandidateComponentsMetadata {
    /// 条目列表。
    items: Vec<ItemMetadata>,
}

impl CandidateComponentsMetadata {
    /// 创建一个空的 [`CandidateComponentsMetadata`]。
    ///
    /// 对应 Spring `CandidateComponentsMetadata()` 无参构造器。
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 追加一个 [`ItemMetadata`]。
    ///
    /// 对应 Spring `CandidateComponentsMetadata#add(ItemMetadata)`。
    pub fn add(&mut self, item: ItemMetadata) {
        self.items.push(item);
    }

    /// 返回只读视图的条目列表。
    ///
    /// 对应 Spring `CandidateComponentsMetadata#getItems()`，返回
    /// `Collections.unmodifiableList(items)`。
    #[must_use]
    pub fn get_items(&self) -> &[ItemMetadata] {
        &self.items
    }

    /// 返回条目数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 返回是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 返回所有 candidate types 的集合（去重）。
    #[must_use]
    pub fn types(&self) -> BTreeSet<&str> {
        self.items.iter().map(ItemMetadata::get_type).collect()
    }

    /// 返回所有 stereotype 的集合（去重）。
    #[must_use]
    pub fn stereotypes(&self) -> BTreeSet<&str> {
        let mut all = BTreeSet::new();
        for item in &self.items {
            for stereotype in item.get_stereotypes() {
                all.insert(stereotype.as_str());
            }
        }
        all
    }
}

impl core::fmt::Display for CandidateComponentsMetadata {
    /// 对应 Spring `CandidateComponentsMetadata#toString()`：
    /// `"CandidateComponentsMetadata{items=[...]}"` 格式。
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "CandidateComponentsMetadata{{items={:?}}}", self.items)
    }
}