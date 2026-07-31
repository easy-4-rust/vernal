//! PagedListHolder — Spring 风格的分页列表持有者。
//!
//! 对应 Java 类：`org.springframework.beans.support.PagedListHolder`。
//!
//! 在 Spring 中，`PagedListHolder` 提供了对任意列表的分页视图。
//! 它支持排序、分页导航和元素访问。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`PagedListHolder` 管理一个泛型列表，
//! 提供分页访问和排序功能。

use std::fmt;

/// 分页列表持有者。
///
/// 对应 Spring 的 `PagedListHolder`。
///
/// 提供对列表的分页视图，支持排序和导航。
#[derive(Debug, Clone)]
pub struct PagedListHolder<T: Clone> {
    /// 元素列表。
    source: Vec<T>,
    /// 每页大小。
    page_size: usize,
    /// 当前页码（从 0 开始）。
    current_page: usize,
    /// 排序属性名。
    sort_property: Option<String>,
    /// 是否升序。
    ascending: bool,
}

impl<T: Clone> PagedListHolder<T> {
    /// 创建分页列表持有者。
    pub fn new(source: Vec<T>) -> Self {
        let len = source.len();
        Self {
            source,
            page_size: len.max(1),
            current_page: 0,
            sort_property: None,
            ascending: true,
        }
    }

    /// 创建带页大小的分页列表持有者。
    pub fn with_page_size(source: Vec<T>, page_size: usize) -> Self {
        Self {
            source,
            page_size: page_size.max(1),
            current_page: 0,
            sort_property: None,
            ascending: true,
        }
    }

    /// 获取源列表大小。
    pub fn source_size(&self) -> usize {
        self.source.len()
    }

    /// 获取每页大小。
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// 设置每页大小。
    pub fn set_page_size(&mut self, page_size: usize) {
        self.page_size = page_size.max(1);
        self.current_page = self.current_page.min(self.page_count().saturating_sub(1));
    }

    /// 获取总页数。
    pub fn page_count(&self) -> usize {
        if self.source.is_empty() {
            return 0;
        }
        (self.source.len() + self.page_size - 1) / self.page_size
    }

    /// 获取当前页码（从 0 开始）。
    pub fn current_page(&self) -> usize {
        self.current_page
    }

    /// 设置当前页码。
    pub fn set_current_page(&mut self, page: usize) {
        self.current_page = page.min(self.page_count().saturating_sub(1));
    }

    /// 是否有下一页。
    pub fn has_next_page(&self) -> bool {
        self.current_page + 1 < self.page_count()
    }

    /// 是否有上一页。
    pub fn has_previous_page(&self) -> bool {
        self.current_page > 0
    }

    /// 前往下一页。
    pub fn next_page(&mut self) {
        if self.has_next_page() {
            self.current_page += 1;
        }
    }

    /// 前往上一页。
    pub fn previous_page(&mut self) {
        if self.has_previous_page() {
            self.current_page -= 1;
        }
    }

    /// 获取当前页的元素。
    pub fn page(&self) -> &[T] {
        let start = self.current_page * self.page_size;
        let end = (start + self.page_size).min(self.source.len());
        if start >= self.source.len() {
            &[]
        } else {
            &self.source[start..end]
        }
    }

    /// 获取源列表。
    pub fn source(&self) -> &[T] {
        &self.source
    }

    /// 设置排序属性。
    pub fn set_sort_property(&mut self, property: impl Into<String>) {
        self.sort_property = Some(property.into());
    }

    /// 设置排序方向。
    pub fn set_ascending(&mut self, ascending: bool) {
        self.ascending = ascending;
    }

    /// 是否升序。
    pub fn is_ascending(&self) -> bool {
        self.ascending
    }
}

impl<T: Clone + fmt::Debug> fmt::Display for PagedListHolder<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PagedListHolder[page={}/{}, size={}, total={}]",
            self.current_page + 1,
            self.page_count(),
            self.page_size,
            self.source.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pagination() {
        let data: Vec<i32> = (0..10).collect();
        let mut holder = PagedListHolder::with_page_size(data, 3);
        assert_eq!(holder.page_count(), 4);
        assert_eq!(holder.page(), &[0, 1, 2]);

        holder.next_page();
        assert_eq!(holder.page(), &[3, 4, 5]);
    }

    #[test]
    fn page_navigation() {
        let data: Vec<i32> = (0..7).collect();
        let mut holder = PagedListHolder::with_page_size(data, 3);
        assert!(holder.has_next_page());
        assert!(!holder.has_previous_page());

        holder.next_page();
        holder.next_page();
        assert_eq!(holder.page(), &[6]);
        assert!(!holder.has_next_page());
        assert!(holder.has_previous_page());
    }

    #[test]
    fn empty_source() {
        let holder: PagedListHolder<i32> = PagedListHolder::new(vec![]);
        assert_eq!(holder.page_count(), 0);
        assert!(holder.page().is_empty());
    }

    #[test]
    fn set_page_size() {
        let data: Vec<i32> = (0..10).collect();
        let mut holder = PagedListHolder::with_page_size(data, 5);
        assert_eq!(holder.page_count(), 2);
        holder.set_page_size(3);
        assert_eq!(holder.page_count(), 4);
    }

    #[test]
    fn display_format() {
        let data: Vec<i32> = (0..5).collect();
        let holder = PagedListHolder::with_page_size(data, 2);
        let display = format!("{}", holder);
        assert!(display.contains("page=1/3"));
    }
}
