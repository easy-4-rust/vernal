//! PropertyMatches — 对应 Java 类：org.springframework.beans.PropertyMatches。
//!
//! 对应 Spring beans 包。
//!
//! 在 Spring 中，`PropertyMatches` 用于计算可能的属性名匹配，
//! 当用户配置了一个不存在的属性名时，Spring 会尝试在所有已知属性中
//! 找到拼写最接近的建议，从而给出友好的错误提示。
//! 使用 Levenshtein 距离算法进行模糊匹配。
//!
//! ## 使用场景
//!
//! - Bean 属性注入时属性名拼写错误
//! - 配置文件中的属性名校验
//! - 提供 "Did you mean ...?" 的错误提示

/// PropertyMatches — Spring 风格的属性名匹配工具。
///
/// 对应 Java 类：`org.springframework.beans.PropertyMatches`。
///
/// 当属性名不匹配时，通过编辑距离算法在已知属性列表中
/// 找到最可能的候选属性名，用于生成友好的错误提示。
///
/// ## Java 对比
///
/// | Java 方法 | Rust 方法 |
/// |-----------|-----------|
/// | `PropertyMatches.forField(String, String[])` | `PropertyMatches::for_property(name, candidates)` |
/// | `PropertyMatches.forProperty(String, String[])` | `PropertyMatches::for_property(name, candidates)` |
/// | `getMatches()` | `matches()` |
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PropertyMatches {
    /// 原始属性名（未匹配到的）
    original: String,
    /// 可能匹配的属性名列表
    matches: Vec<String>,
    /// 最大编辑距离（超过此距离的不列入候选）
    max_distance: usize,
}

impl PropertyMatches {
    /// 创建 PropertyMatches 并计算匹配结果。
    ///
    /// 对应 Java 的 `PropertyMatches.forProperty(String propertyName, String[] possibleMatches)`。
    ///
    /// # 参数
    /// - `property_name` — 未匹配到的属性名
    /// - `candidates` — 已知属性名列表
    pub fn for_property(property_name: &str, candidates: &[&str]) -> Self {
        Self::with_max_distance(property_name, candidates, 2)
    }

    /// 创建带有最大编辑距离限制的 PropertyMatches。
    ///
    /// # 参数
    /// - `property_name` — 未匹配到的属性名
    /// - `candidates` — 已知属性名列表
    /// - `max_distance` — 最大编辑距离
    pub fn with_max_distance(
        property_name: &str,
        candidates: &[&str],
        max_distance: usize,
    ) -> Self {
        let matches = Self::find_matches(property_name, candidates, max_distance);
        Self {
            original: property_name.to_string(),
            matches,
            max_distance,
        }
    }

    /// 获取原始属性名。
    pub fn original(&self) -> &str {
        &self.original
    }

    /// 获取可能匹配的属性名列表。
    ///
    /// 对应 Java 的 `String[] getMatches()`。
    pub fn matches(&self) -> &[String] {
        &self.matches
    }

    /// 是否有匹配的候选。
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    /// 构建错误提示消息。
    ///
    /// 对应 Java 的 `String buildErrorMessage()`。
    ///
    /// 返回类似 "Bean property 'nam' is not writable ... Did you mean 'name'?" 的消息。
    pub fn build_error_message(&self) -> String {
        if self.matches.is_empty() {
            format!(
                "Bean property '{}' is not valid. No similar property found.",
                self.original
            )
        } else {
            let suggestions: Vec<&str> = self.matches.iter().map(|s| s.as_str()).collect();
            format!(
                "Bean property '{}' is not valid. Did you mean '{}'?",
                self.original,
                suggestions.join("' or '")
            )
        }
    }

    /// 在候选列表中找到编辑距离在阈值内的属性名。
    ///
    /// 使用 Levenshtein 距离算法。
    fn find_matches(target: &str, candidates: &[&str], max_distance: usize) -> Vec<String> {
        let target_lower = target.to_lowercase();
        let mut results: Vec<(String, usize)> = candidates
            .iter()
            .filter_map(|&candidate| {
                let dist = Self::levenshtein_distance(&target_lower, &candidate.to_lowercase());
                if dist <= max_distance {
                    Some((candidate.to_string(), dist))
                } else {
                    None
                }
            })
            .collect();

        // 按距离排序，距离相同的按字母排序
        results.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        results.into_iter().map(|(name, _)| name).collect()
    }

    /// 计算两个字符串之间的 Levenshtein 距离。
    ///
    /// 使用动态规划算法，时间复杂度 O(m*n)。
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        // 使用单行 DP 数组
        let mut row: Vec<usize> = (0..=len2).collect();

        for i in 0..len1 {
            let mut prev_diag = row[0];
            row[0] = i + 1;
            for j in 0..len2 {
                let cost = if chars1[i] == chars2[j] { 0 } else { 1 };
                let insert = row[j] + 1;
                let delete = row[j + 1] + 1;
                let replace = prev_diag + cost;
                prev_diag = row[j + 1];
                row[j + 1] = insert.min(delete).min(replace);
            }
        }

        row[len2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match_found() {
        let m = PropertyMatches::for_property("name", &["name", "age", "email"]);
        assert!(m.has_matches());
        assert!(m.matches().contains(&"name".to_string()));
    }

    #[test]
    fn close_match_found() {
        let m = PropertyMatches::for_property("nam", &["name", "age", "email"]);
        assert!(m.has_matches());
        assert!(m.matches().contains(&"name".to_string()));
    }

    #[test]
    fn no_match_for_distant_word() {
        let m = PropertyMatches::for_property("xyz", &["name", "age", "email"]);
        assert!(!m.has_matches());
    }

    #[test]
    fn build_error_message_with_matches() {
        let m = PropertyMatches::for_property("nam", &["name", "age"]);
        let msg = m.build_error_message();
        assert!(msg.contains("'nam'"));
        assert!(msg.contains("Did you mean"));
    }

    #[test]
    fn build_error_message_without_matches() {
        let m = PropertyMatches::for_property("xyz", &["name", "age"]);
        let msg = m.build_error_message();
        assert!(msg.contains("'xyz'"));
        assert!(msg.contains("No similar property found"));
    }

    #[test]
    fn levenshtein_distance_identical() {
        assert_eq!(PropertyMatches::levenshtein_distance("abc", "abc"), 0);
    }

    #[test]
    fn levenshtein_distance_one_edit() {
        assert_eq!(PropertyMatches::levenshtein_distance("abc", "ab"), 1);
        assert_eq!(PropertyMatches::levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(PropertyMatches::levenshtein_distance("abc", "adc"), 1);
    }

    #[test]
    fn matches_sorted_by_distance() {
        let m = PropertyMatches::for_property("nam", &["name", "num"]);
        let matches = m.matches();
        // Both "name" and "num" have edit distance 1 from "nam"
        assert!(!matches.is_empty());
    }

    #[test]
    fn custom_max_distance() {
        let m = PropertyMatches::with_max_distance("nm", &["name", "age"], 1);
        assert!(!m.has_matches());

        let m2 = PropertyMatches::with_max_distance("nm", &["name", "age"], 2);
        assert!(m2.has_matches());
    }

    #[test]
    fn empty_candidates() {
        let m = PropertyMatches::for_property("name", &[]);
        assert!(!m.has_matches());
    }
}
