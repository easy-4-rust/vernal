//! SortDefinition — Spring 风格的排序定义接口。
//!
//! 对应 Java 类：`org.springframework.beans.support.SortDefinition`。
//!
//! 在 Spring 中，`SortDefinition` 定义了排序的基本契约：
//! - 获取排序属性名
//! - 是否忽略大小写
//! - 是否升序
//!
//! ## 设计说明
//!
//! 在 vernal 中，`SortDefinition` 作为 trait 定义排序契约，
//! `MutableSortDefinition` 是其可变实现。

use std::fmt;

/// 排序定义接口。
///
/// 对应 Spring 的 `SortDefinition`。
///
/// 定义排序的基本属性。
pub trait SortDefinition: Send + Sync + fmt::Debug {
    /// 获取排序属性名。
    fn property(&self) -> String;

    /// 是否忽略大小写。
    fn ignore_case(&self) -> bool;

    /// 是否升序。
    fn ascending(&self) -> bool;

    /// 获取排序方向描述。
    fn direction_description(&self) -> &str {
        if self.ascending() { "ASC" } else { "DESC" }
    }
}

/// 简单排序定义。
///
/// 不可变的排序定义实现。
#[derive(Debug, Clone)]
pub struct SimpleSortDefinition {
    property: String,
    ignore_case: bool,
    ascending: bool,
}

impl SimpleSortDefinition {
    /// 创建简单排序定义。
    pub fn new(property: impl Into<String>, ignore_case: bool, ascending: bool) -> Self {
        Self {
            property: property.into(),
            ignore_case,
            ascending,
        }
    }

    /// 创建升序排序定义。
    pub fn ascending(property: impl Into<String>) -> Self {
        Self::new(property, true, true)
    }

    /// 创建降序排序定义。
    pub fn descending(property: impl Into<String>) -> Self {
        Self::new(property, true, false)
    }
}

impl SortDefinition for SimpleSortDefinition {
    fn property(&self) -> String {
        self.property.clone()
    }

    fn ignore_case(&self) -> bool {
        self.ignore_case
    }

    fn ascending(&self) -> bool {
        self.ascending
    }
}

impl fmt::Display for SimpleSortDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.property,
            if self.ascending { "ASC" } else { "DESC" },
            if self.ignore_case {
                "(ignore case)"
            } else {
                "(case sensitive)"
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascending_sort_definition() {
        let sort = SimpleSortDefinition::ascending("name");
        assert_eq!(sort.property(), "name");
        assert!(sort.ascending());
        assert!(sort.ignore_case());
    }

    #[test]
    fn descending_sort_definition() {
        let sort = SimpleSortDefinition::descending("date");
        assert_eq!(sort.property(), "date");
        assert!(!sort.ascending());
    }

    #[test]
    fn display_format() {
        let sort = SimpleSortDefinition::new("id", false, true);
        let display = format!("{}", sort);
        assert!(display.contains("id"));
        assert!(display.contains("ASC"));
        assert!(display.contains("case sensitive"));
    }

    #[test]
    fn direction_description() {
        let asc = SimpleSortDefinition::ascending("a");
        let desc = SimpleSortDefinition::descending("b");
        assert_eq!(asc.direction_description(), "ASC");
        assert_eq!(desc.direction_description(), "DESC");
    }
}
