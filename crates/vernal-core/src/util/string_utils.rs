//! 字符串工具。
//!
//! 对标 Spring `org.springframework.util.StringUtils`。
//!
//! Spring `StringUtils` 有 100+ 静态方法,许多与 Rust std 重复(`trim` / `to_lowercase` 等)。
//! 本模块只实现 **Rust std 中没有但 Spring 中常用** 的方法。
//!
//! # 不实现的方法
//!
//! 以下 Spring 方法在 Rust std 已有等价物,不重复实现:
//!
//! - `trim()` / `trimLeading()` / `trimTrailing()` → Rust `str::trim()` / `trim_start()` / `trim_end()`
//! - `toUpperCase()` / `toLowerCase()` → Rust `to_uppercase()` / `to_lowercase()`
//! - `split(String)` → Rust `split()`
//! - `replace()` → Rust `replace()`
//! - `startsWith()` / `endsWith()` → Rust `starts_with()` / `ends_with()`
//! - `contains()` → Rust `contains()`
//! - `substring()` → Rust slicing `&s[range]`
//! - `length()` → Rust `s.len()`

use std::collections::VecDeque;

/// 字符串工具。
///
/// 对标 Spring `StringUtils`。
pub struct StringUtils;

impl StringUtils {
    /// 检查字符串是否包含非空白字符。
    ///
    /// 对标 Spring `StringUtils.hasText(CharSequence)`。
    ///
    /// 等价于 `!str.trim().is_empty()`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use vernal_core::util::StringUtils;
    ///
    /// assert!(!StringUtils::has_text(""));
    /// assert!(!StringUtils::has_text("   "));
    /// assert!(StringUtils::has_text("hello"));
    /// assert!(StringUtils::has_text("  hello  "));
    /// ```
    #[must_use]
    pub fn has_text(s: &str) -> bool {
        !s.trim().is_empty()
    }

    /// 检查字符串是否非空(包含任何字符,包括空白)。
    ///
    /// 对标 Spring `StringUtils.hasLength(CharSequence)`。
    #[must_use]
    pub fn has_length(s: &str) -> bool {
        !s.is_empty()
    }

    /// 检查字符串是否为空白(包括 null/empty/纯空白)。
    ///
    /// 对标 Spring `!StringUtils.hasText(CharSequence)`。
    #[must_use]
    pub fn is_blank(s: &str) -> bool {
        s.trim().is_empty()
    }

    /// 大小写不敏感地检查是否以指定前缀开头。
    ///
    /// 对标 Spring `StringUtils.startsWithIgnoreCase(CharSequence, CharSequence)`。
    #[must_use]
    pub fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
        if s.len() < prefix.len() {
            return false;
        }
        s[..prefix.len()].eq_ignore_ascii_case(prefix)
    }

    /// 大小写不敏感地检查是否以指定后缀结尾。
    ///
    /// 对标 Spring `StringUtils.endsWithIgnoreCase(CharSequence, CharSequence)`。
    #[must_use]
    pub fn ends_with_ignore_case(s: &str, suffix: &str) -> bool {
        if s.len() < suffix.len() {
            return false;
        }
        s[s.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
    }

    /// 大小写不敏感地检查是否包含子串。
    ///
    /// 对标 Spring `StringUtils.containsIgnoreCase(CharSequence, CharSequence)`。
    #[must_use]
    pub fn contains_ignore_case(s: &str, substr: &str) -> bool {
        if substr.is_empty() {
            return true;
        }
        s.to_lowercase().contains(&substr.to_lowercase())
    }

    /// 把逗号分隔的字符串切分为 Vec<String>(去除空白与空项)。
    ///
    /// 对标 Spring `StringUtils.commaDelimitedListToStringArray(String)`。
    ///
    /// # 注意
    ///
    /// Spring 版本会保留空项(`"a,,b"` → `["a", "", "b"]`),
    /// vernal-core 简化为去除空项以匹配 Rust 习惯。
    /// 如需保留空项,使用 `split(',')`。
    #[must_use]
    pub fn comma_delimited_list_to_vec(s: &str) -> Vec<String> {
        s.split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect()
    }

    /// 把 Vec<String> 用逗号拼接为字符串。
    ///
    /// 对标 Spring `StringUtils.collectionToCommaDelimitedString(Collection)`。
    #[must_use]
    pub fn vec_to_comma_delimited_string(items: &[String]) -> String {
        items.join(",")
    }

    /// 把任意 Iterator<Item = &str> 用逗号拼接为字符串。
    #[must_use]
    pub fn join_comma<'a, I: IntoIterator<Item = &'a str>>(items: I) -> String {
        let items: Vec<&str> = items.into_iter().collect();
        items.join(",")
    }

    /// 把字符串数组用指定分隔符拼接。
    ///
    /// 对标 Spring `StringUtils.arrayToDelimitedString(Object[], String)`。
    #[must_use]
    pub fn join_delimited(items: &[&str], delimiter: &str) -> String {
        items.join(delimiter)
    }

    /// 把字符串首字母大写。
    ///
    /// 对标 Spring `StringUtils.capitalize(String)`。
    ///
    /// 如果首字符已是大写或字符串为空,直接返回原值。
    #[must_use]
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                if first.is_ascii_uppercase() {
                    return s.to_string();
                }
                first.to_ascii_uppercase().to_string() + chars.as_str()
            }
        }
    }

    /// 把字符串首字母小写。
    ///
    /// 对标 Spring `StringUtils.uncapitalize(String)`。
    #[must_use]
    pub fn uncapitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => {
                if first.is_ascii_lowercase() {
                    return s.to_string();
                }
                first.to_ascii_lowercase().to_string() + chars.as_str()
            }
        }
    }

    /// 计算子字符串出现次数。
    ///
    /// 对标 Spring `StringUtils.countOccurrencesOf(String, String)`。
    #[must_use]
    pub fn count_occurrences_of(s: &str, sub: &str) -> usize {
        if sub.is_empty() {
            return 0;
        }
        s.matches(sub).count()
    }

    /// 把驼峰转换为下划线(`helloWorld` → `hello_world`)。
    ///
    /// 对标 Spring `StringUtils.camelCaseToUnderScore(String)`(变体)。
    #[must_use]
    pub fn camel_to_snake(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 4);
        for (i, c) in s.chars().enumerate() {
            if c.is_ascii_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }

    /// 把下划线转换为驼峰(`hello_world` → `helloWorld`)。
    ///
    /// 对标 Spring `StringUtils.underScoreToCamelCase(String)`(变体)。
    #[must_use]
    pub fn snake_to_camel(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut next_upper = false;
        for c in s.chars() {
            if c == '_' {
                next_upper = true;
            } else if next_upper {
                result.push(c.to_ascii_uppercase());
                next_upper = false;
            } else {
                result.push(c);
            }
        }
        result
    }

    /// 删除指定字符的所有出现。
    ///
    /// 对标 Spring `StringUtils.deleteAny(String, String)`(简化版)。
    #[must_use]
    pub fn delete(s: &str, chars_to_delete: &str) -> String {
        s.chars()
            .filter(|c| !chars_to_delete.contains(*c))
            .collect()
    }

    /// 用 `quote` 包装字符串(转义内部 quote)。
    ///
    /// 对标 Spring `StringUtils.quote(String)`。
    #[must_use]
    pub fn quote(s: &str) -> String {
        format!("'{}'", s.replace('\'', "\\'"))
    }

    /// 删除前后空白(对标 Spring `StringUtils.trimWhitespace(String)`)。
    ///
    /// 注意:Rust `str::trim()` 已实现等价行为,本方法仅为兼容性提供。
    #[must_use]
    pub fn trim_whitespace(s: &str) -> &str {
        s.trim()
    }

    /// 删除所有空白字符。
    ///
    /// 对标 Spring `StringUtils.trimAllWhitespace(String)`。
    #[must_use]
    pub fn trim_all_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// 拆分为单词(以空白为分隔符,对标 Spring `StringUtils.tokenizeToStringArray`)。
    #[must_use]
    pub fn tokenize_to_vec(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    /// 拆分为单词队列(对标 Spring `StringUtils.tokenizeToStringArray` 的 Deque 版)。
    #[must_use]
    pub fn tokenize_to_deque(s: &str) -> VecDeque<String> {
        s.split_whitespace().map(String::from).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_text_basic() {
        assert!(StringUtils::has_text("hello"));
        assert!(StringUtils::has_text("  hello  "));
        assert!(!StringUtils::has_text(""));
        assert!(!StringUtils::has_text("   "));
        assert!(!StringUtils::has_text("\t\n"));
    }

    #[test]
    fn has_length_basic() {
        assert!(StringUtils::has_length("hello"));
        assert!(StringUtils::has_length("   ")); // 空白也算有长度
        assert!(!StringUtils::has_length(""));
    }

    #[test]
    fn is_blank_basic() {
        assert!(StringUtils::is_blank(""));
        assert!(StringUtils::is_blank("   "));
        assert!(StringUtils::is_blank("\t\n"));
        assert!(!StringUtils::is_blank("hello"));
    }

    #[test]
    fn starts_with_ignore_case_basic() {
        assert!(StringUtils::starts_with_ignore_case("Hello", "he"));
        assert!(StringUtils::starts_with_ignore_case("HELLO", "he"));
        assert!(StringUtils::starts_with_ignore_case("hello", "He"));
        assert!(!StringUtils::starts_with_ignore_case("hello", "world"));
        assert!(!StringUtils::starts_with_ignore_case("he", "hello")); // prefix 比字符串长
    }

    #[test]
    fn ends_with_ignore_case_basic() {
        assert!(StringUtils::ends_with_ignore_case("Hello", "LO"));
        assert!(StringUtils::ends_with_ignore_case("HELLO", "lo"));
        assert!(!StringUtils::ends_with_ignore_case("hello", "world"));
    }

    #[test]
    fn contains_ignore_case_basic() {
        assert!(StringUtils::contains_ignore_case("hello world", "WORLD"));
        assert!(StringUtils::contains_ignore_case("HELLO", "ello"));
        assert!(StringUtils::contains_ignore_case("hello", "")); // 空子串总包含
        assert!(!StringUtils::contains_ignore_case("hello", "world"));
    }

    #[test]
    fn comma_delimited_list_basic() {
        let v = StringUtils::comma_delimited_list_to_vec("a,b,c");
        assert_eq!(v, vec!["a", "b", "c"]);

        let v = StringUtils::comma_delimited_list_to_vec("  a, b , c  ");
        assert_eq!(v, vec!["a", "b", "c"]);

        let v = StringUtils::comma_delimited_list_to_vec("");
        assert!(v.is_empty());
    }

    #[test]
    fn vec_to_comma_delimited_basic() {
        let v = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(StringUtils::vec_to_comma_delimited_string(&v), "a,b,c");

        let empty: Vec<String> = vec![];
        assert_eq!(StringUtils::vec_to_comma_delimited_string(&empty), "");
    }

    #[test]
    fn join_comma_works_with_iterator() {
        let s = StringUtils::join_comma(["a", "b", "c"]);
        assert_eq!(s, "a,b,c");
    }

    #[test]
    fn join_delimited_basic() {
        let items = ["a", "b", "c"];
        assert_eq!(StringUtils::join_delimited(&items, "; "), "a; b; c");
    }

    #[test]
    fn capitalize_basic() {
        assert_eq!(StringUtils::capitalize("hello"), "Hello");
        assert_eq!(StringUtils::capitalize("Hello"), "Hello"); // 已大写不变
        assert_eq!(StringUtils::capitalize(""), "");
        assert_eq!(StringUtils::capitalize("hELLO"), "HELLO"); // 只改首字符
    }

    #[test]
    fn uncapitalize_basic() {
        assert_eq!(StringUtils::uncapitalize("Hello"), "hello");
        assert_eq!(StringUtils::uncapitalize("hello"), "hello"); // 已小写不变
        assert_eq!(StringUtils::uncapitalize(""), "");
        assert_eq!(StringUtils::uncapitalize("HELLO"), "hELLO");
    }

    #[test]
    fn count_occurrences_of_basic() {
        assert_eq!(StringUtils::count_occurrences_of("ababab", "a"), 3);
        assert_eq!(StringUtils::count_occurrences_of("ababab", "ab"), 3);
        assert_eq!(StringUtils::count_occurrences_of("hello", ""), 0);
        assert_eq!(StringUtils::count_occurrences_of("hello", "x"), 0);
    }

    #[test]
    fn camel_to_snake_basic() {
        assert_eq!(StringUtils::camel_to_snake("helloWorld"), "hello_world");
        assert_eq!(StringUtils::camel_to_snake("HelloWorld"), "hello_world");
        assert_eq!(
            StringUtils::camel_to_snake("already_snake"),
            "already_snake"
        );
        assert_eq!(StringUtils::camel_to_snake(""), "");
    }

    #[test]
    fn snake_to_camel_basic() {
        assert_eq!(StringUtils::snake_to_camel("hello_world"), "helloWorld");
        assert_eq!(StringUtils::snake_to_camel("user_id"), "userId");
        assert_eq!(StringUtils::snake_to_camel(""), "");
    }

    #[test]
    fn delete_basic() {
        assert_eq!(StringUtils::delete("hello world", "lo"), "he wrd");
        assert_eq!(StringUtils::delete("a-b-c", "-"), "abc");
    }

    #[test]
    fn quote_basic() {
        assert_eq!(StringUtils::quote("hello"), "'hello'");
        assert_eq!(StringUtils::quote(""), "''");
    }

    #[test]
    fn trim_whitespace_basic() {
        assert_eq!(StringUtils::trim_whitespace("  hello  "), "hello");
    }

    #[test]
    fn trim_all_whitespace_basic() {
        assert_eq!(
            StringUtils::trim_all_whitespace("  hello  world  "),
            "helloworld"
        );
        assert_eq!(StringUtils::trim_all_whitespace("a\tb\nc"), "abc");
    }

    #[test]
    fn tokenize_to_vec_basic() {
        let v = StringUtils::tokenize_to_vec("hello world foo");
        assert_eq!(v, vec!["hello", "world", "foo"]);

        let v = StringUtils::tokenize_to_vec("  hello   world  ");
        assert_eq!(v, vec!["hello", "world"]);

        let v = StringUtils::tokenize_to_vec("");
        assert!(v.is_empty());
    }

    #[test]
    fn tokenize_to_deque_basic() {
        let d = StringUtils::tokenize_to_deque("hello world");
        assert_eq!(d.len(), 2);
        assert_eq!(d[0], "hello");
        assert_eq!(d[1], "world");
    }

    #[test]
    fn capitalize_handles_unicode() {
        // 非 ASCII 字符首字符不变(对标 Spring 仅处理 ASCII)
        assert_eq!(StringUtils::capitalize("中文"), "中文");
    }
}
