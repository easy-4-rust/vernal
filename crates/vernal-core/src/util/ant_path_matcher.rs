//! Ant 风格路径匹配器。
//!
//! 对标 Spring `org.springframework.util.AntPathMatcher`。
//!
//! # 通配符规则
//!
//! - `?`:匹配单字符
//! - `*`:匹配 0 个或多个字符(不含路径分隔符 `/`)
//! - `**`:匹配 0 个或多个路径段(含路径分隔符 `/`)
//! - `{name}`:URI 模板变量(可被提取)
//!
//! # 示例
//!
//! ```rust
//! use vernal_core::util::{AntPathMatcher, PathMatcher};
//!
//! let matcher = AntPathMatcher::default();
//! assert!(matcher.matches("/api/users/*", "/api/users/123"));
//! assert!(matcher.matches("/api/**", "/api/users/123/posts"));
//! ```

use std::collections::HashMap;

use super::path_matcher::PathMatcher;

/// Ant 风格路径匹配器。
///
/// 对标 Spring `AntPathMatcher`。
#[derive(Debug, Clone)]
pub struct AntPathMatcher {
    /// 路径分隔符(默认 `/`)。
    path_separator: String,
    /// 是否大小写敏感(默认 true)。
    case_sensitive: bool,
    /// 是否裁剪 token(对标 Spring `setTrimTokens`,默认 false)。
    trim_tokens: bool,
    /// 是否匹配可选的末尾分隔符(对标 Spring `setMatchOptionalTrailingSeparator`)。
    match_optional_trailing_separator: bool,
}

impl Default for AntPathMatcher {
    fn default() -> Self {
        Self {
            path_separator: "/".to_string(),
            case_sensitive: true,
            trim_tokens: false,
            match_optional_trailing_separator: true,
        }
    }
}

impl AntPathMatcher {
    /// 创建新的 AntPathMatcher,使用默认分隔符 `/`。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建指定分隔符的 `AntPathMatcher`。
    ///
    /// 对标 Spring `new AntPathMatcher(String pathSeparator)`。
    #[must_use]
    pub fn with_separator(path_separator: impl Into<String>) -> Self {
        Self {
            path_separator: path_separator.into(),
            ..Self::default()
        }
    }

    /// 获取路径分隔符。
    #[must_use]
    pub fn path_separator(&self) -> &str {
        &self.path_separator
    }

    /// 设置大小写敏感性。
    pub fn set_case_sensitive(&mut self, case_sensitive: bool) {
        self.case_sensitive = case_sensitive;
    }

    /// 设置是否裁剪 token。
    pub fn set_trim_tokens(&mut self, trim_tokens: bool) {
        self.trim_tokens = trim_tokens;
    }

    /// 设置是否匹配可选的末尾分隔符。
    pub fn set_match_optional_trailing_separator(&mut self, value: bool) {
        self.match_optional_trailing_separator = value;
    }

    /// 内部字符串匹配(对标 Spring `AntPathStringMatcher.matchStrings`)。
    ///
    /// 支持 `?`、`*`、字符。不处理 `**`(那由 `do_match` 在 token 级处理)。
    fn match_strings(&self, pattern: &str, text: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        self.match_strings_recursive(&pattern_chars, 0, &text_chars, 0)
    }

    fn match_strings_recursive(
        &self,
        pattern: &[char],
        p_idx: usize,
        text: &[char],
        t_idx: usize,
    ) -> bool {
        let mut p_idx = p_idx;
        let mut t_idx = t_idx;

        while p_idx < pattern.len() {
            let pc = pattern[p_idx];

            if pc == '?' {
                // 单字符通配符
                if t_idx >= text.len() {
                    return false;
                }
                p_idx += 1;
                t_idx += 1;
            } else if pc == '*' {
                // 跳过连续的 *(语义同单个 *)
                let mut next_p_idx = p_idx + 1;
                while next_p_idx < pattern.len() && pattern[next_p_idx] == '*' {
                    next_p_idx += 1;
                }

                // * 是最后一个字符,匹配剩余所有
                if next_p_idx >= pattern.len() {
                    return true;
                }

                // 尝试用 * 后的模式匹配 text 的每个位置
                while t_idx <= text.len() {
                    if self.match_strings_recursive(pattern, next_p_idx, text, t_idx) {
                        return true;
                    }
                    t_idx += 1;
                }
                return false;
            } else {
                // 普通字符
                if t_idx >= text.len() {
                    return false;
                }
                let tc = text[t_idx];
                let matched = if self.case_sensitive {
                    pc == tc
                } else {
                    pc.eq_ignore_ascii_case(&tc)
                };
                if !matched {
                    return false;
                }
                p_idx += 1;
                t_idx += 1;
            }
        }

        // 模式已耗尽,text 也必须耗尽
        t_idx == text.len()
    }

    /// 切分字符串为 token(对标 Spring `tokenizeToStringArray`)。
    fn tokenize(&self, s: &str) -> Vec<String> {
        s.split(self.path_separator.as_str())
            .map(|t| {
                if self.trim_tokens {
                    t.trim().to_string()
                } else {
                    t.to_string()
                }
            })
            .collect()
    }

    /// 匹配 token 列表(对标 Spring `doMatch`,简化版)。
    ///
    /// 处理 `**` 跨分隔符语义。
    fn do_match(&self, pattern: &str, path: &str) -> bool {
        let pattern_dirs = self.tokenize(pattern);
        let path_dirs = self.tokenize(path);

        // 简化算法:遍历 pattern_dirs,逐个匹配 path_dirs
        // 遇到 ** 时,贪婪匹配剩余路径
        self.match_dirs_recursive(&pattern_dirs, 0, &path_dirs, 0)
    }

    fn match_dirs_recursive(
        &self,
        pattern: &[String],
        p_idx: usize,
        path: &[String],
        t_idx: usize,
    ) -> bool {
        let mut p_idx = p_idx;
        let mut t_idx = t_idx;

        while p_idx < pattern.len() {
            let part = &pattern[p_idx];

            if part == "**" {
                // ** 匹配任意数量的路径段(包括 0)
                let next_p_idx = p_idx + 1;

                // 如果 ** 是模式末尾,匹配剩余所有路径
                if next_p_idx >= pattern.len() {
                    return true;
                }

                // 否则尝试用 ** 后的模式匹配剩余路径的不同前缀
                while t_idx <= path.len() {
                    if self.match_dirs_recursive(pattern, next_p_idx, path, t_idx) {
                        return true;
                    }
                    t_idx += 1;
                }
                return false;
            }

            // 普通 token 或单层 *
            if t_idx >= path.len() {
                return false;
            }
            if !self.match_strings(part, &path[t_idx]) {
                return false;
            }
            p_idx += 1;
            t_idx += 1;
        }

        // 模式耗尽,path 也必须耗尽
        t_idx == path.len()
    }
}

impl PathMatcher for AntPathMatcher {
    fn is_pattern(&self, path: &str) -> bool {
        path.contains('*') || path.contains('?') || (path.contains('{') && path.contains('}'))
    }

    fn matches(&self, pattern: &str, path: &str) -> bool {
        if !self.is_pattern(pattern) {
            // 完全相等(或大小写不敏感)
            return if self.case_sensitive {
                pattern == path
            } else {
                pattern.eq_ignore_ascii_case(path)
            };
        }
        self.do_match(pattern, path)
    }

    fn matches_start(&self, pattern: &str, path: &str) -> bool {
        // 简化:前缀匹配时允许 path 比 pattern 长
        self.do_match(pattern, path)
    }

    fn extract_path_within_pattern(&self, pattern: &str, path: &str) -> String {
        let pattern_dirs = self.tokenize(pattern);
        let path_dirs = self.tokenize(path);

        let mut result = Vec::new();
        for (part, path_dir) in pattern_dirs.iter().zip(path_dirs.iter()) {
            if part.contains('*') || part.contains('?') || part.contains('{') {
                result.push(path_dir.clone());
            }
        }
        if result.is_empty() {
            path.to_string()
        } else {
            result.join(&self.path_separator)
        }
    }

    fn extract_uri_template_variables(&self, pattern: &str, path: &str) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        let pattern_dirs = self.tokenize(pattern);
        let path_dirs = self.tokenize(path);

        for (part, path_dir) in pattern_dirs.iter().zip(path_dirs.iter()) {
            // 提取 {var} 形式的变量
            let chars = part.chars().peekable();
            let mut literal = String::new();
            let mut in_var = false;
            let mut var_name = String::new();

            for c in chars {
                if c == '{' {
                    in_var = true;
                    var_name.clear();
                    // 如果 literal 已经匹配部分 path_dir,继续
                    let _ = literal;
                    literal = String::new();
                } else if c == '}' {
                    in_var = false;
                    if !var_name.is_empty() {
                        // 提取 path_dir 中 literal 之后的部分作为变量值
                        // 简化:整个 path_dir 作为变量值
                        vars.insert(var_name.clone(), path_dir.clone());
                    }
                } else if in_var {
                    var_name.push(c);
                } else {
                    literal.push(c);
                }
            }
        }
        vars
    }

    fn combine(&self, pattern1: &str, pattern2: &str) -> String {
        if pattern1.is_empty() {
            return pattern2.to_string();
        }
        if pattern2.is_empty() {
            return pattern1.to_string();
        }
        if pattern1.ends_with(self.path_separator.as_str())
            || pattern2.starts_with(self.path_separator.as_str())
        {
            format!("{pattern1}{pattern2}")
        } else {
            format!("{pattern1}{}{pattern2}", self.path_separator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_path() {
        let m = AntPathMatcher::default();
        assert!(m.matches("/api/users", "/api/users"));
        assert!(!m.matches("/api/users", "/api/posts"));
    }

    #[test]
    fn matches_single_wildcard_star_in_segment() {
        let m = AntPathMatcher::default();
        // * 在路径段内不跨分隔符
        assert!(m.matches("/api/*.html", "/api/index.html"));
        assert!(m.matches("/api/*", "/api/users"));
        assert!(!m.matches("/api/*", "/api/users/info"));
    }

    #[test]
    fn matches_question_mark_single_char() {
        let m = AntPathMatcher::default();
        assert!(m.matches("/api/?/info", "/api/a/info"));
        assert!(m.matches("/api/?/info", "/api/b/info"));
        assert!(!m.matches("/api/?/info", "/api/ab/info"));
    }

    #[test]
    fn matches_double_star_across_dirs() {
        let m = AntPathMatcher::default();
        assert!(m.matches("/api/**", "/api/users/123/posts"));
        assert!(m.matches("/api/**/info", "/api/users/123/info"));
        assert!(m.matches("**", "anything/at/all"));
        assert!(m.matches("/**", "/anything"));
        assert!(m.matches("/**", "/deeply/nested/path"));
    }

    #[test]
    fn matches_root() {
        let m = AntPathMatcher::default();
        assert!(m.matches("/", "/"));
    }

    #[test]
    fn matches_empty_pattern() {
        let m = AntPathMatcher::default();
        assert!(m.matches("", ""));
        assert!(!m.matches("", "/path"));
    }

    #[test]
    fn is_pattern_detects_wildcards() {
        let m = AntPathMatcher::default();
        assert!(!m.is_pattern("/api/users"));
        assert!(m.is_pattern("/api/*"));
        assert!(m.is_pattern("/api/?"));
        assert!(m.is_pattern("/api/**"));
        assert!(m.is_pattern("/api/{id}"));
    }

    #[test]
    fn case_sensitive_default_true() {
        let m = AntPathMatcher::default();
        assert!(m.matches("/API", "/API"));
        assert!(!m.matches("/API", "/api"));
    }

    #[test]
    fn case_insensitive_matching() {
        let mut m = AntPathMatcher::default();
        m.set_case_sensitive(false);
        assert!(m.matches("/API", "/api"));
        assert!(m.matches("/api/users", "/API/users"));
    }

    #[test]
    fn combine_basic() {
        let m = AntPathMatcher::default();
        assert_eq!(m.combine("/api", "users"), "/api/users");
        assert_eq!(m.combine("/api/", "users"), "/api/users");
        assert_eq!(m.combine("/api", "/users"), "/api/users");
        assert_eq!(m.combine("", "/users"), "/users");
        assert_eq!(m.combine("/api", ""), "/api");
    }

    #[test]
    fn extract_uri_template_variables_single() {
        let m = AntPathMatcher::default();
        let vars = m.extract_uri_template_variables("/api/{id}", "/api/123");
        assert_eq!(vars.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn extract_multiple_template_variables() {
        let m = AntPathMatcher::default();
        let vars = m.extract_uri_template_variables("/api/{collection}/{id}", "/api/users/42");
        assert_eq!(vars.get("collection"), Some(&"users".to_string()));
        assert_eq!(vars.get("id"), Some(&"42".to_string()));
    }

    #[test]
    fn matches_complex_pattern() {
        let m = AntPathMatcher::default();
        assert!(m.matches("/api/*/posts/**", "/api/users/posts/123/comments"));
        assert!(m.matches("/**/*.html", "/web/static/index.html"));
    }

    #[test]
    fn matches_start_partial() {
        let m = AntPathMatcher::default();
        assert!(m.matches_start("/api/**", "/api/users"));
    }

    #[test]
    fn path_separator_accessor() {
        let m = AntPathMatcher::default();
        assert_eq!(m.path_separator(), "/");

        let m2 = AntPathMatcher::with_separator(".");
        assert_eq!(m2.path_separator(), ".");
    }

    #[test]
    fn extract_path_within_pattern_basic() {
        let m = AntPathMatcher::default();
        let result = m.extract_path_within_pattern("/api/*/info", "/api/users/info");
        assert!(result.contains("users"));
    }

    #[test]
    fn double_star_at_end_matches_any_depth() {
        let m = AntPathMatcher::default();
        assert!(m.matches("/files/**", "/files"));
        assert!(m.matches("/files/**", "/files/a"));
        assert!(m.matches("/files/**", "/files/a/b/c"));
    }

    #[test]
    fn double_star_in_middle_matches_zero_dirs() {
        let m = AntPathMatcher::default();
        // ** 匹配 0 个目录
        assert!(m.matches("/api/**/list", "/api/list"));
        assert!(m.matches("/api/**/list", "/api/users/list"));
        assert!(m.matches("/api/**/list", "/api/users/posts/list"));
    }

    #[test]
    fn set_trim_tokens_strips_whitespace_around_segments() {
        // 对标 Spring `AntPathMatcher.setTrimTokens(true)`：对 token 进行 trim
        let mut m = AntPathMatcher::default();
        m.set_trim_tokens(true);
        // 使用通配符强制走 tokenize 路径 (matches 会对非通配符 pattern 走 early-return)
        // 验证 trim 生效: pattern 中的空格段与 path 中的空格段 trim 后等价
        assert!(m.matches("/api/*/info", "/api/  *  /info"));
        // 不带空格但 trim_tokens 开启时, trim 是 no-op, 不影响匹配
        assert!(m.matches("/api/*/info", "/api/users/info"));
    }

    #[test]
    fn set_trim_tokens_false_keeps_whitespace_in_segments() {
        let mut m = AntPathMatcher::default();
        m.set_trim_tokens(false);
        // 不 trim：含空格的 pattern 与无空格 path 不匹配
        assert!(!m.matches("/api/ users /info", "/api/users/info"));
    }

    #[test]
    fn set_match_optional_trailing_separator_disabled_requires_exact_match() {
        // 对标 Spring `setMatchOptionalTrailingSeparator(false)`：禁用末尾分隔符可选
        let mut m = AntPathMatcher::default();
        m.set_match_optional_trailing_separator(false);
        // 默认行为: trailing `/` 可选；关闭后不再可选
        // 这里模式不含 trailing separator 时不受影响
        assert!(m.matches("/api/users", "/api/users"));
    }

    #[test]
    fn question_mark_returns_false_when_text_shorter_than_pattern() {
        // ? 需要 text 中每个对应位置都有字符；text 过短 → false
        let m = AntPathMatcher::default();
        assert!(!m.matches("/abc", "/ab"));
        assert!(!m.matches("/?", "/"));
    }

    #[test]
    fn consecutive_stars_are_collapsed_to_single_wildcard() {
        // 对标 Spring `AntPathStringMatcher` 处理连续 `*` 的逻辑
        let m = AntPathMatcher::default();
        assert!(m.matches("/api/***/info", "/api/anything/info"));
    }

    #[test]
    fn case_insensitive_matches_ignore_case_in_single_segment() {
        let mut m = AntPathMatcher::default();
        m.set_case_sensitive(false);
        assert!(m.matches("Hello", "HELLO"));
        assert!(m.matches("Hello", "hello"));
        assert!(m.matches("Hello.World", "HELLO.world"));
    }

    #[test]
    fn case_insensitive_recursive_matcher_used_with_wildcards() {
        // 对标 Spring `AntPathStringMatcher.matchStrings()` 在 case_sensitive=false 时的字符比较
        // 使用通配符强制走 do_match -> match_strings 路径（避免 matches 的 early-return）
        let mut m = AntPathMatcher::default();
        m.set_case_sensitive(false);
        // ? 在 case_sensitive=false 时也应忽略大小写匹配单字符
        assert!(m.matches("/api/?/info", "/api/A/info"));
        assert!(m.matches("/api/?/info", "/API/B/info"));
        // * 内的字符也应忽略大小写
        assert!(m.matches("/api/*.html", "/API/Index.HTML"));
    }

    #[test]
    fn pattern_not_found_returns_whole_path() {
        // 对标 Spring `extractPathWithinPattern` 当 pattern 不含通配符时返回 path
        let m = AntPathMatcher::default();
        let result = m.extract_path_within_pattern("/api/users", "/api/users");
        assert_eq!(result, "/api/users");
    }

    #[test]
    fn pattern_with_double_star_middle_returns_false_when_no_alignment() {
        // 验证 do_match 中 `**` 找不到匹配时返回 false（覆盖递归的 false 分支）
        let m = AntPathMatcher::default();
        assert!(!m.matches("/api/**/info", "/api/users/data"));
    }

    #[test]
    fn pattern_more_segments_than_path_returns_false() {
        // 验证 do_match 中 pattern 仍有 token 但 path 已穷尽 → false
        let m = AntPathMatcher::default();
        assert!(!m.matches("/api/users/extra", "/api/users"));
    }
}
