//! 简单模式匹配工具。
//!
//! 对标 Spring `org.springframework.util.PatternMatchUtils`，支持 `xxx*`、`*xxx`、
//! `*xxx*`、`xxx*yyy`（任意数量模式段）风格。

/// 简单模式匹配工具。
///
/// 对标 Spring `org.springframework.util.PatternMatchUtils`。
/// 所有方法均为关联函数（对标 Java `static`），不持有状态。
pub struct PatternMatchUtils;

impl PatternMatchUtils {
    /// 将字符串与单个模式匹配，支持 `xxx*`、`*xxx`、`*xxx*`、`xxx*yyy`。
    ///
    /// 对标 Spring `simpleMatch(String pattern, String str)`。
    /// 当 pattern 或 str 为空时返回 `false`（对标 Java `null` → `false`）。
    #[must_use]
    pub fn simple_match(pattern: &str, s: &str) -> bool {
        Self::do_simple_match(pattern, s, false)
    }

    /// `simple_match` 的忽略大小写变体。
    ///
    /// 对标 Spring `simpleMatchIgnoreCase(String pattern, String str)`（since 6.1.20）。
    #[must_use]
    pub fn simple_match_ignore_case(pattern: &str, s: &str) -> bool {
        Self::do_simple_match(pattern, s, true)
    }

    /// 将字符串与多个模式匹配，任一命中即返回 `true`。
    ///
    /// 对标 Spring `simpleMatch(String[] patterns, String str)`。
    #[must_use]
    pub fn simple_match_any(patterns: &[&str], s: &str) -> bool {
        patterns.iter().any(|p| Self::simple_match(p, s))
    }

    /// `simple_match_any` 的忽略大小写变体。
    ///
    /// 对标 Spring `simpleMatchIgnoreCase(String[] patterns, String str)`（since 6.1.20）。
    #[must_use]
    pub fn simple_match_any_ignore_case(patterns: &[&str], s: &str) -> bool {
        patterns.iter().any(|p| Self::simple_match_ignore_case(p, s))
    }

    /// 核心递归匹配逻辑，严格翻译 Spring 私有方法 `simpleMatch(pattern, str, ignoreCase)`。
    fn do_simple_match(pattern: &str, s: &str, ignore_case: bool) -> bool {
        let first_star = pattern.find('*');

        // 无通配符：直接比较
        let Some(first_index) = first_star else {
            return if ignore_case {
                pattern.eq_ignore_ascii_case(s)
            } else {
                pattern == s
            };
        };

        // 模式以 * 开头
        if first_index == 0 {
            // 模式只有一个 *：匹配任意
            if pattern.len() == 1 {
                return true;
            }
            let after_first_star = &pattern[1..]; // UTF-8 安全：* 是单字节
            let next_star = after_first_star.find('*');

            // 只有一个 * 前缀：检查 str 是否以剩余部分结尾
            if next_star.is_none() {
                let part = after_first_star;
                return if ignore_case {
                    s.len() >= part.len()
                        && s[s.len() - part.len()..].eq_ignore_ascii_case(part)
                } else {
                    s.ends_with(part)
                };
            }

            let next_index = next_star.unwrap();
            let part = &after_first_star[..next_index];
            let rest_pattern = &after_first_star[next_index..];

            // 空段：跳过，递归匹配剩余模式
            if part.is_empty() {
                return Self::do_simple_match(rest_pattern, s, ignore_case);
            }

            // 在 str 中查找 part 的所有出现位置，逐个递归尝试
            let mut search_start = 0usize;
            while let Some(part_index) = Self::index_of(s, part, search_start, ignore_case) {
                let rest_str = &s[part_index + part.len()..];
                if Self::do_simple_match(rest_pattern, rest_str, ignore_case) {
                    return true;
                }
                search_start = part_index + 1;
            }
            return false;
        }

        // 模式不以 * 开头：检查 str 是否以 pattern[0..first_index] 开头
        // 注意：first_index 是 * 的字节位置，pattern[0..first_index] 是前缀
        let prefix_len = first_index;
        s.len() >= prefix_len
            && Self::starts_with(pattern, &s[..prefix_len], ignore_case)
            && Self::do_simple_match(
                &pattern[first_index..],
                &s[prefix_len..],
                ignore_case,
            )
    }

    /// 检查 pattern 是否以 part 开头（可忽略大小写）。
    ///
    /// 对标 Spring 私有方法 `checkStartsWith`。
    fn starts_with(pattern: &str, part: &str, ignore_case: bool) -> bool {
        if ignore_case {
            pattern.len() >= part.len() && pattern[..part.len()].eq_ignore_ascii_case(part)
        } else {
            pattern.starts_with(part)
        }
    }

    /// 在 str 中从 `start_index` 开始查找 `other_str`。
    ///
    /// 对标 Spring 私有方法 `indexOf(str, otherStr, startIndex, ignoreCase)`。
    /// 返回字节偏移量（因为只处理 ASCII 子串，字节偏移 == 字符偏移）。
    fn index_of(str: &str, other: &str, start: usize, ignore_case: bool) -> Option<usize> {
        if !ignore_case {
            return str[start..].find(other).map(|i| start + i);
        }
        if other.is_empty() {
            return Some(start);
        }
        let str_bytes = str.as_bytes();
        let other_bytes = other.as_bytes();
        if start + other_bytes.len() > str_bytes.len() {
            return None;
        }
        let max = str_bytes.len() - other_bytes.len();
        for i in start..=max {
            // regionMatches(true, ...) — 逐字节忽略大小写比较
            if str_bytes[i..i + other_bytes.len()]
                .iter()
                .zip(other_bytes)
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
            {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 基本匹配（对标 Spring PatternMatchUtilsTests）──

    #[test]
    fn exact_match() {
        assert!(PatternMatchUtils::simple_match("abc", "abc"));
        assert!(!PatternMatchUtils::simple_match("abc", "abd"));
    }

    #[test]
    fn prefix_wildcard() {
        assert!(PatternMatchUtils::simple_match("abc*", "abcdef"));
        assert!(!PatternMatchUtils::simple_match("abc*", "xyz"));
    }

    #[test]
    fn suffix_wildcard() {
        assert!(PatternMatchUtils::simple_match("*def", "abcdef"));
        assert!(!PatternMatchUtils::simple_match("*def", "abc"));
    }

    #[test]
    fn middle_wildcard() {
        assert!(PatternMatchUtils::simple_match("*abc*", "xxabcyy"));
        assert!(PatternMatchUtils::simple_match("*abc*", "abc"));
        assert!(!PatternMatchUtils::simple_match("*abc*", "xyz"));
    }

    #[test]
    fn multi_segment_wildcard() {
        assert!(PatternMatchUtils::simple_match("abc*def", "abcxxxdef"));
        assert!(!PatternMatchUtils::simple_match("abc*def", "abcxxx"));
        assert!(PatternMatchUtils::simple_match("a*b*c", "axxbyyc"));
    }

    #[test]
    fn single_star_matches_all() {
        assert!(PatternMatchUtils::simple_match("*", "anything"));
        assert!(PatternMatchUtils::simple_match("*", ""));
    }

    #[test]
    fn empty_inputs() {
        // 空模式不匹配非空字符串
        assert!(!PatternMatchUtils::simple_match("", "abc"));
        // 非空模式不匹配空字符串（除非模式是 *）
        assert!(!PatternMatchUtils::simple_match("abc", ""));
        // 两个空字符串相等（Spring 语义：非 null 空字符串 equals 为 true）
        assert!(PatternMatchUtils::simple_match("", ""));
    }

    // ── 忽略大小写 ──

    #[test]
    fn ignore_case_exact() {
        assert!(PatternMatchUtils::simple_match_ignore_case("ABC", "abc"));
        assert!(PatternMatchUtils::simple_match_ignore_case("abc", "ABC"));
    }

    #[test]
    fn ignore_case_wildcard() {
        assert!(PatternMatchUtils::simple_match_ignore_case("ABC*", "abcdef"));
        assert!(PatternMatchUtils::simple_match_ignore_case("*DEF", "abcDEF"));
        assert!(PatternMatchUtils::simple_match_ignore_case("*ABC*", "xxABcyy"));
    }

    #[test]
    fn ignore_case_multi_segment() {
        assert!(PatternMatchUtils::simple_match_ignore_case("a*B*c", "AxxByyC"));
    }

    // ── 多模式匹配 ──

    #[test]
    fn any_match() {
        assert!(PatternMatchUtils::simple_match_any(&["foo*", "bar*"], "foobar"));
        assert!(PatternMatchUtils::simple_match_any(&["foo*", "bar*"], "barbaz"));
        assert!(!PatternMatchUtils::simple_match_any(&["foo*", "bar*"], "baz"));
    }

    #[test]
    fn any_match_empty() {
        assert!(!PatternMatchUtils::simple_match_any(&[], "abc"));
    }

    #[test]
    fn any_match_ignore_case() {
        assert!(PatternMatchUtils::simple_match_any_ignore_case(&["FOO*", "BAR*"], "foobar"));
    }

    // ── 边界情况（对标 Spring 测试）──

    #[test]
    fn double_star() {
        // ** 等价于 *（空段被跳过）
        assert!(PatternMatchUtils::simple_match("**", "anything"));
        assert!(PatternMatchUtils::simple_match("a**b", "axxb"));
    }

    #[test]
    fn star_at_end() {
        assert!(PatternMatchUtils::simple_match("abc*", "abc"));
        assert!(PatternMatchUtils::simple_match("abc*", "abcdefghi"));
    }

    #[test]
    fn no_partial_prefix() {
        // str 长度不足前缀长度
        assert!(!PatternMatchUtils::simple_match("abcdef*", "abc"));
    }

    #[test]
    fn simple_match_any_first_pattern_misses_second_hits() {
        // 对标 Spring: simpleMatch(String[], String) 逐个尝试
        assert!(PatternMatchUtils::simple_match_any(&["xyz*", "abc*"], "abcdef"));
    }

    #[test]
    fn simple_match_any_all_miss() {
        assert!(!PatternMatchUtils::simple_match_any(&["foo*", "bar*", "baz*"], "qux"));
    }

    #[test]
    fn ignore_case_suffix_with_uppercase_pattern() {
        // 对标 Spring: simpleMatchIgnoreCase("*DEF", "abcdef")
        assert!(PatternMatchUtils::simple_match_ignore_case("*DEF", "abcdef"));
        assert!(!PatternMatchUtils::simple_match_ignore_case("*XYZ", "abcdef"));
    }

    #[test]
    fn ignore_case_prefix_with_mixed_case() {
        assert!(PatternMatchUtils::simple_match_ignore_case("HeLLo*", "hElLo world"));
    }

    // ── 覆盖 index_of 私有函数分支 ──

    #[test]
    fn index_of_ignore_case_empty_other_returns_start() {
        // 对标 Spring: indexOf(str, "", startIndex, true) 应返回 startIndex
        // 覆盖行 132: other.is_empty() && ignore_case → Some(start)
        assert_eq!(
            PatternMatchUtils::index_of("hello", "", 3, true),
            Some(3)
        );
        assert_eq!(
            PatternMatchUtils::index_of("hello", "", 0, true),
            Some(0)
        );
    }

    #[test]
    fn index_of_ignore_case_remaining_too_short_returns_none() {
        // 对标 Spring: remaining string too short for pattern
        // 覆盖行 137: start + other_bytes.len() > str_bytes.len() → None
        assert_eq!(
            PatternMatchUtils::index_of("ab", "abc", 0, true),
            None
        );
    }

    #[test]
    fn index_of_ignore_case_no_match_returns_none() {
        // 对标 Spring: pattern not found with ignoreCase
        // 覆盖行 150: for loop exhausted → None
        assert_eq!(
            PatternMatchUtils::index_of("hello", "xyz", 0, true),
            None
        );
        assert_eq!(
            PatternMatchUtils::index_of("abc", "def", 1, true),
            None
        );
    }

    // ── 覆盖多段通配符回溯搜索（行 94-95）──

    #[test]
    fn multi_segment_backtrack_first_occurrence_fails_second_succeeds() {
        // 对标 Spring: 多段模式匹配中，中间段第一次出现时后续匹配失败，
        // 需要回溯到第二次出现才成功。
        // 覆盖行 93-95: while 循环中的 return true 和 search_start = part_index + 1
        // pattern: *a*a  str: "xaxba"
        //   第一个 "a" 在 pos=1, rest="xba", "*a" 不匹配 "xba"（不以 "a" 结尾）
        //   回溯到第二个 "a" 在 pos=3, rest="a", "*a" 匹配 "a"（以 "a" 结尾）
        assert!(PatternMatchUtils::simple_match("*a*a", "xaxba"));
    }

    #[test]
    fn multi_segment_backtrack_all_occurrences_fail() {
        // 对标 Spring: 多段模式匹配中，所有中间段出现位置都导致后续匹配失败
        // 覆盖行 95: search_start = part_index + 1（多次执行）
        // pattern: *a*a  str: "xaxbx"
        //   "a" at pos=1, rest="xbx", 不以 "a" 结尾
        //   "a" not found from pos=2 → return false
        assert!(!PatternMatchUtils::simple_match("*a*a", "xaxbx"));
    }
}
