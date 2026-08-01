//! 框架常量。
//!
//! 对标 Spring `org.springframework.core.Constants` 类。
//!
//! Spring `Constants` 是一个通用的"常量名↔值"双向映射工具,主要用于
//! 解析 `@Value("#{T(java.lang.Math).PI}")` 这类 `SpEL` 表达式中的常量引用。
//!
//! vernal-core 用一个简单的 `HashMap<&'static str, i64>` 表达等价语义。

use std::collections::HashMap;

/// 框架常量注册表。
///
/// 对应 Java: org.springframework.core.Constants
/// 对标 Spring `Constants`。
///
/// # 示例
///
/// ```rust
/// use vernal_core::constants::Constants;
///
/// let mut constants = Constants::new();
/// constants.register("MAX_VALUE", i64::MAX);
/// constants.register("MIN_VALUE", i64::MIN);
///
/// assert_eq!(constants.get("MAX_VALUE"), Some(i64::MAX));
/// assert_eq!(constants.get("MISSING"), None);
/// ```
#[derive(Debug, Clone)]
pub struct Constants {
    /// 常量名 → 值映射
    values: HashMap<&'static str, i64>,
    /// 常量名 → 描述映射(可选)
    descriptions: HashMap<&'static str, &'static str>,
}

impl Constants {
    /// 创建空的常量注册表。
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            descriptions: HashMap::new(),
        }
    }

    /// 注册一个常量。
    pub fn register(&mut self, name: &'static str, value: i64) {
        self.values.insert(name, value);
    }

    /// 注册一个带描述的常量。
    pub fn register_with_description(
        &mut self,
        name: &'static str,
        value: i64,
        description: &'static str,
    ) {
        self.values.insert(name, value);
        self.descriptions.insert(name, description);
    }

    /// 获取常量值。
    #[must_use]
    pub fn get(&self, name: &str) -> Option<i64> {
        self.values.get(name).copied()
    }

    /// 获取常量描述。
    #[must_use]
    pub fn description(&self, name: &str) -> Option<&'static str> {
        self.descriptions.get(name).copied()
    }

    /// 检查常量是否存在。
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// 获取所有常量名。
    pub fn names(&self) -> impl Iterator<Item = &&'static str> {
        self.values.keys()
    }

    /// 常量数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// 创建包含常用数学常量的注册表。
    #[must_use]
    pub fn with_math_constants() -> Self {
        let mut c = Self::new();
        c.register_with_description("PI", 3, "圆周率的整数近似(3)");
        c.register_with_description("E", 2, "自然常数的整数近似(2)");
        c.register_with_description("MAX_INT", i64::MAX, "i64 最大值");
        c.register_with_description("MIN_INT", i64::MIN, "i64 最小值");
        c
    }
}

impl Default for Constants {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get() {
        let mut c = Constants::new();
        c.register("A", 1);
        c.register("B", 2);
        assert_eq!(c.get("A"), Some(1));
        assert_eq!(c.get("B"), Some(2));
        assert_eq!(c.get("C"), None);
    }

    #[test]
    fn register_with_description() {
        let mut c = Constants::new();
        c.register_with_description("PI", 3, "圆周率近似");
        assert_eq!(c.get("PI"), Some(3));
        assert_eq!(c.description("PI"), Some("圆周率近似"));
    }

    #[test]
    fn contains_basic() {
        let mut c = Constants::new();
        c.register("A", 1);
        assert!(c.contains("A"));
        assert!(!c.contains("B"));
    }

    #[test]
    fn len_and_is_empty() {
        let mut c = Constants::new();
        assert!(c.is_empty());
        assert_eq!(c.len(), 0);
        c.register("A", 1);
        assert_eq!(c.len(), 1);
        assert!(!c.is_empty());
    }

    #[test]
    fn names_returns_all_keys() {
        let mut c = Constants::new();
        c.register("A", 1);
        c.register("B", 2);
        let names: Vec<&&str> = c.names().collect();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn with_math_constants_basic() {
        let c = Constants::with_math_constants();
        assert!(c.contains("PI"));
        assert!(c.contains("E"));
        assert!(c.contains("MAX_INT"));
        assert!(c.contains("MIN_INT"));
        assert_eq!(c.get("MAX_INT"), Some(i64::MAX));
    }

    #[test]
    fn default_is_empty() {
        let c = Constants::default();
        assert!(c.is_empty());
    }

    #[test]
    fn description_returns_none_for_missing() {
        let c = Constants::new();
        assert!(c.description("MISSING").is_none());
    }

    #[test]
    fn register_overwrites_previous() {
        let mut c = Constants::new();
        c.register("A", 1);
        c.register("A", 2);
        assert_eq!(c.get("A"), Some(2));
    }
}
