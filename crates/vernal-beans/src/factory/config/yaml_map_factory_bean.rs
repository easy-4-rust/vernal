//! YamlMapFactoryBean — 对应 Spring `org.springframework.beans.factory.config.YamlMapFactoryBean`。
//!
//! YAML Map 工厂 Bean。

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// YAML Map 工厂 Bean。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.YamlMapFactoryBean`。
///
/// 用于从 YAML 文件创建 Map 集合。
///
/// ## 使用场景
///
/// - 加载 YAML 配置为 Map
/// - YAML 文档处理
/// - 配置数据结构化
#[derive(Debug, Default)]
pub struct YamlMapFactoryBean {
    /// 资源位置。
    resources: Vec<String>,
    /// YAML 文档匹配状态。
    match_status: MatchStatus,
}

/// YAML 文档匹配策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchStatus {
    /// 匹配所有文档。
    All,
    /// 只匹配第一个文档。
    First,
    /// 只匹配最后一个文档。
    Last,
}

impl Default for MatchStatus {
    fn default() -> Self {
        Self::All
    }
}

impl YamlMapFactoryBean {
    /// 创建新的 YamlMapFactoryBean。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置资源位置。
    pub fn set_resources(&mut self, resources: Vec<String>) {
        self.resources = resources;
    }

    /// 添加资源位置。
    pub fn add_resource(&mut self, resource: impl Into<String>) {
        self.resources.push(resource.into());
    }

    /// 获取资源位置。
    pub fn resources(&self) -> &[String] {
        &self.resources
    }

    /// 设置 YAML 文档匹配状态。
    pub fn set_match_status(&mut self, match_status: MatchStatus) {
        self.match_status = match_status;
    }

    /// 获取 YAML 文档匹配状态。
    pub fn match_status(&self) -> MatchStatus {
        self.match_status
    }

    /// 解析 YAML 文本为 Map。
    pub fn parse_yaml(&self, yaml_text: &str) -> HashMap<String, String> {
        let mut result = HashMap::new();
        let mut current_prefix = String::new();

        for line in yaml_text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim();
                let value = trimmed[colon_pos + 1..].trim();

                if value.is_empty() {
                    if current_prefix.is_empty() {
                        current_prefix = key.to_string();
                    } else {
                        current_prefix = format!("{}.{}", current_prefix, key);
                    }
                } else {
                    let full_key = if current_prefix.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", current_prefix, key)
                    };
                    result.insert(full_key, value.to_string());
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_map_factory_bean_new() {
        let factory = YamlMapFactoryBean::new();
        assert!(factory.resources().is_empty());
        assert_eq!(factory.match_status(), MatchStatus::All);
    }

    #[test]
    fn test_yaml_map_factory_bean_parse() {
        let factory = YamlMapFactoryBean::new();
        let yaml = "database:\n  host: localhost\n  port: 5432";
        let map = factory.parse_yaml(yaml);

        assert_eq!(map.get("database.host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("database.port"), Some(&"5432".to_string()));
    }

    #[test]
    fn test_yaml_map_factory_bean_match_status() {
        let mut factory = YamlMapFactoryBean::new();
        factory.set_match_status(MatchStatus::First);
        assert_eq!(factory.match_status(), MatchStatus::First);
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn default_trait_creates_correctly() {
        let factory = YamlMapFactoryBean::default();
        assert!(factory.resources().is_empty());
        assert_eq!(factory.match_status(), MatchStatus::All);
    }

    #[test]
    fn set_resources_bulk() {
        let mut factory = YamlMapFactoryBean::new();
        factory.set_resources(vec!["a.yml".to_string(), "b.yml".to_string()]);
        assert_eq!(factory.resources().len(), 2);
    }

    #[test]
    fn add_resource_single() {
        let mut factory = YamlMapFactoryBean::new();
        factory.add_resource("config.yml");
        factory.add_resource("other.yml".to_string());
        assert_eq!(factory.resources().len(), 2);
    }

    #[test]
    fn match_status_all_variants() {
        let mut factory = YamlMapFactoryBean::new();

        factory.set_match_status(MatchStatus::All);
        assert_eq!(factory.match_status(), MatchStatus::All);

        factory.set_match_status(MatchStatus::First);
        assert_eq!(factory.match_status(), MatchStatus::First);

        factory.set_match_status(MatchStatus::Last);
        assert_eq!(factory.match_status(), MatchStatus::Last);
    }

    #[test]
    fn match_status_default_is_all() {
        assert_eq!(MatchStatus::default(), MatchStatus::All);
    }

    #[test]
    fn parse_yaml_simple_key_value() {
        let factory = YamlMapFactoryBean::new();
        let yaml = "host: localhost\nport: 8080";
        let map = factory.parse_yaml(yaml);
        assert_eq!(map.get("host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("port"), Some(&"8080".to_string()));
    }

    #[test]
    fn parse_yaml_nested_keys() {
        let factory = YamlMapFactoryBean::new();
        let yaml = "server:\n  host: localhost\n  port: 8080";
        let map = factory.parse_yaml(yaml);
        assert_eq!(map.get("server.host"), Some(&"localhost".to_string()));
        assert_eq!(map.get("server.port"), Some(&"8080".to_string()));
    }

    #[test]
    fn parse_yaml_skips_comments() {
        let factory = YamlMapFactoryBean::new();
        let yaml = "# comment\nhost: localhost\n# another comment\nport: 8080";
        let map = factory.parse_yaml(yaml);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("host"));
        assert!(map.contains_key("port"));
    }

    #[test]
    fn parse_yaml_skips_empty_lines() {
        let factory = YamlMapFactoryBean::new();
        let yaml = "host: localhost\n\n\nport: 8080";
        let map = factory.parse_yaml(yaml);
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn parse_yaml_empty_input() {
        let factory = YamlMapFactoryBean::new();
        let map = factory.parse_yaml("");
        assert!(map.is_empty());
    }

    #[test]
    fn parse_yaml_only_comments() {
        let factory = YamlMapFactoryBean::new();
        let yaml = "# comment1\n# comment2";
        let map = factory.parse_yaml(yaml);
        assert!(map.is_empty());
    }

    #[test]
    fn debug_format() {
        let factory = YamlMapFactoryBean::new();
        let debug = format!("{:?}", factory);
        assert!(debug.contains("YamlMapFactoryBean"));
    }

    #[test]
    fn match_status_debug_clone() {
        let status = MatchStatus::All;
        let cloned = status;
        assert_eq!(status, cloned);
        let debug = format!("{:?}", status);
        assert!(debug.contains("All"));
    }
}
