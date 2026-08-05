//! ReplaceOverride — Spring 风格的替换方法覆盖。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.ReplaceOverride`。
//!
//! 在 Spring 中，`ReplaceOverride` 用于将 Bean 的方法替换为
//! `MethodReplacer` 实现。这通过 XML 的 `<replaced-method>` 配置。
//!
//! ## 使用场景
//!
//! 当需要在不修改源码的情况下替换 Bean 的方法实现时使用，
//! 例如替换算法策略、添加日志等。

use crate::factory::support::method_override::MethodOverride;

/// 替换方法覆盖。
///
/// 对应 Spring 的 `ReplaceOverride`。
///
/// 将指定方法替换为 `MethodReplacer` 实现。
#[derive(Clone, Debug)]
pub struct ReplaceOverride {
    /// 被替换的方法名
    method_name: String,
    /// 替换器的 Bean 名称
    replacer_name: String,
    /// 方法参数类型标识（用于方法重载匹配）
    type_identifiers: Vec<String>,
}

impl ReplaceOverride {
    /// 创建新的替换方法覆盖。
    ///
    /// # 参数
    /// - `method_name` — 被替换的方法名
    /// - `replacer_name` — 替换器的 Bean 名称
    pub fn new(method_name: impl Into<String>, replacer_name: impl Into<String>) -> Self {
        Self {
            method_name: method_name.into(),
            replacer_name: replacer_name.into(),
            type_identifiers: Vec::new(),
        }
    }

    /// 添加类型标识符（用于方法重载匹配）。
    ///
    /// 对应 Spring 的 `<arg-type>` 子元素。
    pub fn add_type_identifier(&mut self, identifier: impl Into<String>) {
        self.type_identifiers.push(identifier.into());
    }

    /// 获取被替换的方法名。
    pub fn get_method_name(&self) -> &str {
        &self.method_name
    }

    /// 获取替换器的 Bean 名称。
    pub fn get_replacer_name(&self) -> &str {
        &self.replacer_name
    }

    /// 获取类型标识符列表。
    pub fn type_identifiers(&self) -> &[String] {
        &self.type_identifiers
    }

    /// 是否有类型标识符。
    pub fn has_type_identifiers(&self) -> bool {
        !self.type_identifiers.is_empty()
    }
}

impl MethodOverride for ReplaceOverride {
    fn get_method_name(&self) -> &str {
        &self.method_name
    }
    fn is_applicable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::support::method_override::MethodOverride;

    #[test]
    fn new_replace_override() {
        let ro = ReplaceOverride::new("compute", "computeReplacer");
        assert_eq!(ro.get_method_name(), "compute");
        assert_eq!(ro.get_replacer_name(), "computeReplacer");
        assert!(!ro.has_type_identifiers());
    }

    #[test]
    fn add_type_identifiers() {
        let mut ro = ReplaceOverride::new("process", "processReplacer");
        ro.add_type_identifier("java.lang.String");
        ro.add_type_identifier("int");

        assert!(ro.has_type_identifiers());
        assert_eq!(ro.type_identifiers().len(), 2);
        assert_eq!(ro.type_identifiers()[0], "java.lang.String");
    }

    #[test]
    fn method_override_trait() {
        let ro = ReplaceOverride::new("doWork", "replacer");
        assert_eq!(MethodOverride::get_method_name(&ro), "doWork");
        assert!(ro.is_applicable());
    }

    #[test]
    fn clone_preserves_state() {
        let mut ro = ReplaceOverride::new("test", "replacer");
        ro.add_type_identifier("String");
        let cloned = ro.clone();
        assert_eq!(cloned.get_method_name(), "test");
        assert_eq!(cloned.type_identifiers().len(), 1);
    }
}
