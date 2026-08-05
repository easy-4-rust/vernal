//! PropertyAccessorUtils — 对应 Java 类：org.springframework.beans.PropertyAccessorUtils。
//!
//! 对应 Spring beans 包。
//!
//! 在 Spring 中，`PropertyAccessorUtils` 是一个工具类，提供属性路径解析和操作的
//! 静态方法。主要用于处理嵌套属性路径（如 `"address.city.name"`），
//! 解析索引属性（如 `"list[0]"`），以及属性名称的规范化。
//!
//! ## 核心功能
//!
//! - 属性路径解析（获取首段、末段、嵌套路径）
//! - 属性名规范化（去除索引标记）
//! - 路径判断（是否嵌套、是否可嵌套）

use std::fmt;

/// PropertyAccessorUtils — Spring 风格的属性访问工具类。
///
/// 对应 Java 类：`org.springframework.beans.PropertyAccessorUtils`。
///
/// 提供静态工具方法，用于处理属性路径的解析、规范化和判断。
/// 所有方法都是无状态的纯函数。
///
/// ## Java 对比
///
/// | Java 方法 | Rust 方法 |
/// |-----------|-----------|
/// | `getPropertyName(String)` | `property_name(path)` |
/// | `getFirstNestedPropertySeparatorIndex(String)` | `first_nested_separator_index(path)` |
/// | `matchesProperty(String[], String)` | `matches_property(required, candidate)` |
pub struct PropertyAccessorUtils;

impl PropertyAccessorUtils {
    /// 获取属性路径中的属性名（去除索引标记）。
    ///
    /// 对应 Java 的 `String getPropertyName(String propertyPath)`。
    ///
    /// # 示例
    ///
    /// ```
    /// use vernal_beans::property_accessor_utils::PropertyAccessorUtils;
    ///
    /// assert_eq!(PropertyAccessorUtils::property_name("list[0]"), "list");
    /// assert_eq!(PropertyAccessorUtils::property_name("name"), "name");
    /// assert_eq!(PropertyAccessorUtils::property_name("map[key]"), "map");
    /// ```
    pub fn property_name(path: &str) -> &str {
        match path.find('[') {
            Some(pos) => &path[..pos],
            None => path,
        }
    }

    /// 获取属性路径中第一个嵌套分隔符的位置。
    ///
    /// 对应 Java 的 `int getFirstNestedPropertySeparatorIndex(String)`。
    ///
    /// 嵌套分隔符为 `.`，但不包括 `.` 后紧跟 `[` 的情况（那是索引属性）。
    ///
    /// # 返回
    ///
    /// - `Some(index)` — 找到嵌套分隔符
    /// - `None` — 无嵌套路径
    pub fn first_nested_separator_index(path: &str) -> Option<usize> {
        let bytes = path.as_bytes();
        for i in 0..bytes.len() {
            if bytes[i] == b'.' {
                // 排除 ".[" 的情况（索引访问紧跟点号）
                if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                    continue;
                }
                return Some(i);
            }
        }
        None
    }

    /// 获取属性路径中最后一个嵌套分隔符的位置。
    ///
    /// 对应 Java 的 `int getLastNestedPropertySeparatorIndex(String)`。
    pub fn last_nested_separator_index(path: &str) -> Option<usize> {
        let bytes = path.as_bytes();
        for i in (0..bytes.len()).rev() {
            if bytes[i] == b'.' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                    continue;
                }
                return Some(i);
            }
        }
        None
    }

    /// 判断属性路径是否是嵌套路径（包含 `.` 分隔符）。
    ///
    /// 对应 Java 的判断逻辑。
    pub fn is_nested_path(path: &str) -> bool {
        Self::first_nested_separator_index(path).is_some()
    }

    /// 获取嵌套属性路径的首段。
    ///
    /// 对于 `"address.city.name"` 返回 `"address"`。
    /// 对于 `"address"` 返回 `"address"`。
    pub fn first_segment(path: &str) -> &str {
        match Self::first_nested_separator_index(path) {
            Some(pos) => &path[..pos],
            None => Self::property_name(path),
        }
    }

    /// 获取嵌套属性路径的尾部（首段之后的部分）。
    ///
    /// 对于 `"address.city.name"` 返回 `"city.name"`。
    /// 对于 `"address"` 返回 `None`。
    pub fn nested_path_after_first(path: &str) -> Option<&str> {
        match Self::first_nested_separator_index(path) {
            Some(pos) => {
                let rest = &path[pos + 1..];
                if rest.is_empty() { None } else { Some(rest) }
            }
            None => None,
        }
    }

    /// 规范化属性名（去除首尾空白和索引标记）。
    pub fn canonical_property_name(name: &str) -> String {
        let trimmed = name.trim();
        Self::property_name(trimmed).to_string()
    }

    /// 检查属性名是否匹配候选名（支持模糊匹配）。
    ///
    /// 对应 Java 的 `String matchesProperty(String[] required, String candidate)`。
    ///
    /// # 参数
    /// - `required` — 需要匹配的属性名数组
    /// - `candidate` — 候选属性名
    ///
    /// # 返回
    ///
    /// - `true` — 候选名在 required 数组中或 required 为空
    pub fn matches_property(required: &[&str], candidate: &str) -> bool {
        if required.is_empty() {
            return true;
        }
        let canonical = Self::canonical_property_name(candidate);
        required
            .iter()
            .any(|r| Self::canonical_property_name(r) == canonical)
    }

    /// 计算属性路径的嵌套深度。
    ///
    /// - `"name"` => 1
    /// - `"address.city"` => 2
    /// - `"address.city.name"` => 3
    pub fn path_depth(path: &str) -> usize {
        let mut count = 0;
        let mut i = 0;
        let bytes = path.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'.' {
                // Skip ".[" patterns
                if i + 1 < bytes.len() && bytes[i + 1] == b'[' {
                    i += 2;
                    continue;
                }
                count += 1;
            }
            i += 1;
        }
        count + 1
    }

    /// 判断路径是否包含索引表达式。
    pub fn has_index(path: &str) -> bool {
        path.contains('[')
    }
}

impl fmt::Debug for PropertyAccessorUtils {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PropertyAccessorUtils")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_name_strips_index() {
        assert_eq!(PropertyAccessorUtils::property_name("list[0]"), "list");
        assert_eq!(PropertyAccessorUtils::property_name("map[key]"), "map");
        assert_eq!(PropertyAccessorUtils::property_name("name"), "name");
    }

    #[test]
    fn first_nested_separator_index_finds_dot() {
        assert_eq!(
            PropertyAccessorUtils::first_nested_separator_index("address.city"),
            Some(7)
        );
        assert_eq!(
            PropertyAccessorUtils::first_nested_separator_index("address"),
            None
        );
    }

    #[test]
    fn first_nested_separator_index_skips_dot_bracket() {
        // "list.[0].name" — the ".[" at index 4 is not a real nested separator
        // the second dot at index 8 is the actual nested separator
        assert_eq!(
            PropertyAccessorUtils::first_nested_separator_index("list.[0].name"),
            Some(8)
        );
    }

    #[test]
    fn last_nested_separator_index() {
        assert_eq!(
            PropertyAccessorUtils::last_nested_separator_index("a.b.c"),
            Some(3)
        );
        assert_eq!(
            PropertyAccessorUtils::last_nested_separator_index("name"),
            None
        );
    }

    #[test]
    fn is_nested_path() {
        assert!(PropertyAccessorUtils::is_nested_path("address.city"));
        assert!(!PropertyAccessorUtils::is_nested_path("name"));
    }

    #[test]
    fn first_segment() {
        assert_eq!(
            PropertyAccessorUtils::first_segment("address.city.name"),
            "address"
        );
        assert_eq!(PropertyAccessorUtils::first_segment("name"), "name");
    }

    #[test]
    fn nested_path_after_first() {
        assert_eq!(
            PropertyAccessorUtils::nested_path_after_first("address.city.name"),
            Some("city.name")
        );
        assert_eq!(PropertyAccessorUtils::nested_path_after_first("name"), None);
    }

    #[test]
    fn canonical_property_name() {
        assert_eq!(
            PropertyAccessorUtils::canonical_property_name("  name  "),
            "name"
        );
        assert_eq!(
            PropertyAccessorUtils::canonical_property_name("list[0]"),
            "list"
        );
    }

    #[test]
    fn matches_property_empty_matches_any() {
        assert!(PropertyAccessorUtils::matches_property(&[], "anything"));
    }

    #[test]
    fn matches_property_exact() {
        assert!(PropertyAccessorUtils::matches_property(
            &["name", "age"],
            "name"
        ));
        assert!(!PropertyAccessorUtils::matches_property(
            &["name", "age"],
            "other"
        ));
    }

    #[test]
    fn path_depth() {
        assert_eq!(PropertyAccessorUtils::path_depth("name"), 1);
        assert_eq!(PropertyAccessorUtils::path_depth("a.b"), 2);
        assert_eq!(PropertyAccessorUtils::path_depth("a.b.c"), 3);
    }

    #[test]
    fn has_index() {
        assert!(PropertyAccessorUtils::has_index("list[0]"));
        assert!(!PropertyAccessorUtils::has_index("name"));
    }
}
