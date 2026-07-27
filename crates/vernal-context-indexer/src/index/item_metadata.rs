//! 对应 Java 类：`org.springframework.context.index.processor.ItemMetadata`
//!
//! 索引中的一条记录。`type` 定义候选目标的标识（通常为完全限定类名），
//! `stereotypes` 是可用于检索候选的"标记"。典型用例是候选上某个注解的存在。
//!
//! @author Stephane Nicoll
//! @since 5.0

use std::collections::BTreeSet;

/// 索引中的一条记录。
///
/// 对应 Spring `org.springframework.context.index.processor.ItemMetadata`。
///
/// `type` 定义候选目标的标识（通常为完全限定类名），`stereotypes` 是可用于
/// 检索候选的"标记"。典型用例是候选上某个注解的存在。
///
/// 与 [`crate::LinkedComponentEntry`] 的关系：
/// - `LinkedComponentEntry` 是 vernal 链接期 + 运行时通用的条目
/// - `ItemMetadata` 是 Spring `processor/` 子模块的高层镜像
/// - 两者字段完全对应（type + stereotypes）
///
/// # 与 Spring 的对应关系
///
/// | Spring | Vernal |
/// |--------|--------|
/// | `ItemMetadata(String type, Set<String> stereotypes)` | `ItemMetadata::new(type, stereotypes)` |
/// | `getType() : String` | `ItemMetadata::get_type() -> &str` |
/// | `getStereotypes() : Set<String>` | `ItemMetadata::get_stereotypes() -> &BTreeSet<String>` |
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemMetadata {
    /// 候选类型标识（通常为完全限定类名）。
    r#type: String,
    /// 该候选携带的 stereotype 集合。
    stereotypes: BTreeSet<String>,
}

impl ItemMetadata {
    /// 创建一个新的 [`ItemMetadata`]。
    ///
    /// 对应 Spring `ItemMetadata(String type, Set<String> stereotypes)` 构造器。
    /// 注意：Spring 把传入的 `stereotypes` 复制到新的 `HashSet` 中去重；vernal
    /// 使用 `BTreeSet` 同时承担去重 + 排序。
    #[must_use]
    pub fn new(r#type: impl Into<String>, stereotypes: BTreeSet<String>) -> Self {
        Self {
            r#type: r#type.into(),
            stereotypes,
        }
    }

    /// 返回候选类型标识。
    ///
    /// 对应 Spring `ItemMetadata#getType()`。
    #[must_use]
    pub fn get_type(&self) -> &str {
        &self.r#type
    }

    /// 返回该候选携带的 stereotype 集合。
    ///
    /// 对应 Spring `ItemMetadata#getStereotypes()`。
    #[must_use]
    pub fn get_stereotypes(&self) -> &BTreeSet<String> {
        &self.stereotypes
    }
}