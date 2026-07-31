//! PropertyComparator — Spring 风格的属性比较器。
//!
//! 对应 Java 类：`org.springframework.beans.support.PropertyComparator`。
//!
//! 在 Spring 中，`PropertyComparator` 根据 Bean 的指定属性进行排序比较。
//! 它使用 `BeanWrapper` 获取属性值，然后使用自然排序或自定义比较器排序。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`PropertyComparator` 使用闭包获取属性值进行比较。

use std::cmp::Ordering;
use std::collections::HashMap;

/// 属性比较器。
///
/// 对应 Spring 的 `PropertyComparator`。
///
/// 根据对象的指定属性值进行排序比较。
#[derive(Debug)]
pub struct PropertyComparator {
    /// 排序属性名。
    property_name: String,
    /// 是否升序。
    ascending: bool,
    /// 是否忽略大小写（针对字符串属性）。
    ignore_case: bool,
}

impl PropertyComparator {
    /// 创建属性比较器。
    pub fn new(property_name: impl Into<String>, ascending: bool) -> Self {
        Self {
            property_name: property_name.into(),
            ascending,
            ignore_case: false,
        }
    }

    /// 设置忽略大小写。
    pub fn with_ignore_case(mut self, ignore: bool) -> Self {
        self.ignore_case = ignore;
        self
    }

    /// 获取属性名。
    pub fn property_name(&self) -> &str {
        &self.property_name
    }

    /// 比较两个属性值字符串。
    pub fn compare_values(&self, a: &str, b: &str) -> Ordering {
        let ordering = if self.ignore_case {
            a.to_lowercase().cmp(&b.to_lowercase())
        } else {
            a.cmp(b)
        };

        if self.ascending {
            ordering
        } else {
            ordering.reverse()
        }
    }

    /// 对属性映射列表进行排序。
    ///
    /// # 参数
    /// - `items` — 属性映射列表
    pub fn sort(&self, items: &mut [HashMap<String, String>]) {
        let property = self.property_name.clone();
        let ignore_case = self.ignore_case;
        let ascending = self.ascending;

        items.sort_by(|a, b| {
            let val_a = a.get(&property).map(|s| s.as_str()).unwrap_or("");
            let val_b = b.get(&property).map(|s| s.as_str()).unwrap_or("");

            let ordering = if ignore_case {
                val_a.to_lowercase().cmp(&val_b.to_lowercase())
            } else {
                val_a.cmp(val_b)
            };

            if ascending { ordering } else { ordering.reverse() }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_values_ascending() {
        let cmp = PropertyComparator::new("name", true);
        assert_eq!(cmp.compare_values("apple", "banana"), Ordering::Less);
        assert_eq!(cmp.compare_values("banana", "apple"), Ordering::Greater);
        assert_eq!(cmp.compare_values("same", "same"), Ordering::Equal);
    }

    #[test]
    fn compare_values_descending() {
        let cmp = PropertyComparator::new("name", false);
        assert_eq!(cmp.compare_values("apple", "banana"), Ordering::Greater);
    }

    #[test]
    fn compare_values_ignore_case() {
        let cmp = PropertyComparator::new("name", true).with_ignore_case(true);
        assert_eq!(cmp.compare_values("Apple", "apple"), Ordering::Equal);
    }

    #[test]
    fn sort_items_by_property() {
        let cmp = PropertyComparator::new("name", true);
        let mut items = vec![
            HashMap::from([("name".to_string(), "Charlie".to_string())]),
            HashMap::from([("name".to_string(), "Alice".to_string())]),
            HashMap::from([("name".to_string(), "Bob".to_string())]),
        ];
        cmp.sort(&mut items);
        assert_eq!(items[0].get("name"), Some(&"Alice".to_string()));
        assert_eq!(items[1].get("name"), Some(&"Bob".to_string()));
        assert_eq!(items[2].get("name"), Some(&"Charlie".to_string()));
    }
}
