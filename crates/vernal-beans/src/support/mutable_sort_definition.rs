//! MutableSortDefinition — Spring 风格的可变排序定义。
//!
//! 对应 Java 类：`org.springframework.beans.support.MutableSortDefinition`。
//!
//! 在 Spring 中，`MutableSortDefinition` 是 `SortDefinition` 的可变实现，
//! 允许在运行时修改排序属性。主要用于分页和列表排序场景。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`MutableSortDefinition` 使用 `std::sync::Mutex` 保护可变状态，
//! 支持线程安全的排序属性修改。

use std::fmt;

/// 可变排序定义。
///
/// 对应 Spring 的 `MutableSortDefinition`。
///
/// 可在运行时修改的排序定义。
#[derive(Debug)]
pub struct MutableSortDefinition {
    inner: std::sync::Mutex<SortState>,
}

/// 排序状态。
#[derive(Debug, Clone)]
struct SortState {
    /// 排序属性名。
    property: String,
    /// 是否忽略大小写。
    ignore_case: bool,
    /// 是否升序。
    ascending: bool,
}

impl MutableSortDefinition {
    /// 创建可变排序定义。
    pub fn new(property: impl Into<String>, ignore_case: bool, ascending: bool) -> Self {
        Self {
            inner: std::sync::Mutex::new(SortState {
                property: property.into(),
                ignore_case,
                ascending,
            }),
        }
    }

    /// 创建空的排序定义。
    pub fn empty() -> Self {
        Self {
            inner: std::sync::Mutex::new(SortState {
                property: String::new(),
                ignore_case: true,
                ascending: true,
            }),
        }
    }

    /// 获取排序属性名。
    pub fn property(&self) -> String {
        self.inner.lock().unwrap().property.clone()
    }

    /// 是否忽略大小写。
    pub fn ignore_case(&self) -> bool {
        self.inner.lock().unwrap().ignore_case
    }

    /// 是否升序。
    pub fn ascending(&self) -> bool {
        self.inner.lock().unwrap().ascending
    }

    /// 设置排序属性名。
    pub fn set_property(&self, property: impl Into<String>) {
        self.inner.lock().unwrap().property = property.into();
    }

    /// 设置是否忽略大小写。
    pub fn set_ignore_case(&self, ignore_case: bool) {
        self.inner.lock().unwrap().ignore_case = ignore_case;
    }

    /// 设置是否升序。
    pub fn set_ascending(&self, ascending: bool) {
        self.inner.lock().unwrap().ascending = ascending;
    }

    /// 切换排序方向。
    pub fn toggle_direction(&self) {
        let mut state = self.inner.lock().unwrap();
        state.ascending = !state.ascending;
    }

    /// 是否已设置排序属性。
    pub fn has_property(&self) -> bool {
        !self.inner.lock().unwrap().property.is_empty()
    }
}

impl Clone for MutableSortDefinition {
    fn clone(&self) -> Self {
        let state = self.inner.lock().unwrap().clone();
        Self {
            inner: std::sync::Mutex::new(state),
        }
    }
}

impl fmt::Display for MutableSortDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.inner.lock().unwrap();
        let direction = if state.ascending { "ASC" } else { "DESC" };
        let case = if state.ignore_case {
            "IGNORE CASE"
        } else {
            "CASE SENSITIVE"
        };
        write!(f, "{} {} {}", state.property, direction, case)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sort_definition() {
        let sort = MutableSortDefinition::new("name", false, true);
        assert_eq!(sort.property(), "name");
        assert!(!sort.ignore_case());
        assert!(sort.ascending());
    }

    #[test]
    fn modify_sort_properties() {
        let sort = MutableSortDefinition::new("name", true, true);
        sort.set_property("date");
        sort.set_ascending(false);
        assert_eq!(sort.property(), "date");
        assert!(!sort.ascending());
    }

    #[test]
    fn toggle_direction() {
        let sort = MutableSortDefinition::new("id", true, true);
        assert!(sort.ascending());
        sort.toggle_direction();
        assert!(!sort.ascending());
        sort.toggle_direction();
        assert!(sort.ascending());
    }

    #[test]
    fn empty_sort_definition() {
        let sort = MutableSortDefinition::empty();
        assert!(!sort.has_property());
        assert!(sort.ascending());
    }

    #[test]
    fn display_format() {
        let sort = MutableSortDefinition::new("name", true, false);
        let display = format!("{}", sort);
        assert!(display.contains("name"));
        assert!(display.contains("DESC"));
    }
}
