//! YamlProcessor — 对应 Spring `org.springframework.beans.factory.config.YamlProcessor`。
//!
//! YAML 处理器基类。

use std::collections::HashMap;

/// YAML 处理器基类。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.YamlProcessor`。
///
/// 提供 YAML 文档解析的基础功能。
///
/// ## 使用场景
///
/// - 解析 YAML 配置文件
/// - YAML 文档处理
/// - 配置加载
#[derive(Debug, Default)]
pub struct YamlProcessor {
    /// 资源位置。
    resources: Vec<String>,
    /// 是否解析为列表。
    parse_as_list: bool,
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

impl YamlProcessor {
    /// 创建新的 YamlProcessor。
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

    /// 设置是否解析为列表。
    pub fn set_parse_as_list(&mut self, parse_as_list: bool) {
        self.parse_as_list = parse_as_list;
    }

    /// 是否解析为列表。
    pub fn parse_as_list(&self) -> bool {
        self.parse_as_list
    }

    /// 解析 YAML 文本为属性集合。
    ///
    /// 这是一个简化的实现，将 YAML 键值对解析为扁平化的属性。
    pub fn parse_yaml_to_properties(&self, yaml_text: &str) -> HashMap<String, String> {
        let mut properties = HashMap::new();
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
                    // 嵌套对象
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
                    properties.insert(full_key, value.to_string());
                }
            }
        }

        properties
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_processor_new() {
        let processor = YamlProcessor::new();
        assert!(processor.resources().is_empty());
        assert!(!processor.parse_as_list());
    }

    #[test]
    fn test_yaml_processor_parse_simple() {
        let processor = YamlProcessor::new();
        let yaml = "name: MyApp\nversion: 1.0.0\nport: 8080";
        let properties = processor.parse_yaml_to_properties(yaml);

        assert_eq!(properties.get("name"), Some(&"MyApp".to_string()));
        assert_eq!(properties.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(properties.get("port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_yaml_processor_parse_nested() {
        let processor = YamlProcessor::new();
        let yaml = "server:\n  host: localhost\n  port: 8080";
        let properties = processor.parse_yaml_to_properties(yaml);

        assert_eq!(
            properties.get("server.host"),
            Some(&"localhost".to_string())
        );
        assert_eq!(properties.get("server.port"), Some(&"8080".to_string()));
    }

    #[test]
    fn test_yaml_processor_default() {
        let processor = YamlProcessor::default();
        assert!(processor.resources().is_empty());
        assert!(!processor.parse_as_list());
    }

    #[test]
    fn test_set_resources() {
        let mut processor = YamlProcessor::new();
        processor.set_resources(vec!["config.yaml".to_string(), "app.yaml".to_string()]);
        assert_eq!(processor.resources().len(), 2);
        assert_eq!(processor.resources()[0], "config.yaml");
    }

    #[test]
    fn test_add_resource() {
        let mut processor = YamlProcessor::new();
        processor.add_resource("first.yaml");
        processor.add_resource("second.yaml");
        assert_eq!(processor.resources().len(), 2);
    }

    #[test]
    fn test_set_parse_as_list() {
        let mut processor = YamlProcessor::new();
        assert!(!processor.parse_as_list());
        processor.set_parse_as_list(true);
        assert!(processor.parse_as_list());
    }

    #[test]
    fn test_parse_empty_yaml() {
        let processor = YamlProcessor::new();
        let properties = processor.parse_yaml_to_properties("");
        assert!(properties.is_empty());
    }

    #[test]
    fn test_parse_comments_skipped() {
        let processor = YamlProcessor::new();
        let yaml = "# This is a comment\nname: value\n# Another comment";
        let properties = processor.parse_yaml_to_properties(yaml);
        assert_eq!(properties.len(), 1);
        assert_eq!(properties.get("name"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_empty_lines_skipped() {
        let processor = YamlProcessor::new();
        let yaml = "name: value\n\n\nother: data";
        let properties = processor.parse_yaml_to_properties(yaml);
        assert_eq!(properties.len(), 2);
    }

    #[test]
    fn test_match_status_default() {
        assert_eq!(MatchStatus::default(), MatchStatus::All);
    }

    #[test]
    fn test_match_status_variants() {
        assert_ne!(MatchStatus::All, MatchStatus::First);
        assert_ne!(MatchStatus::First, MatchStatus::Last);
        assert_ne!(MatchStatus::All, MatchStatus::Last);
    }

    #[test]
    fn test_match_status_clone() {
        let status = MatchStatus::First;
        let cloned = status;
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_yaml_with_values_containing_colons() {
        let processor = YamlProcessor::new();
        let yaml = "url: http://example.com";
        let properties = processor.parse_yaml_to_properties(yaml);
        // The value after first colon is " http://example.com"
        assert_eq!(
            properties.get("url"),
            Some(&"http://example.com".to_string())
        );
    }

    #[test]
    fn test_debug_format() {
        let processor = YamlProcessor::new();
        let debug = format!("{:?}", processor);
        assert!(debug.contains("YamlProcessor"));
    }

    #[test]
    fn test_match_status_debug() {
        assert_eq!(format!("{:?}", MatchStatus::All), "All");
        assert_eq!(format!("{:?}", MatchStatus::First), "First");
        assert_eq!(format!("{:?}", MatchStatus::Last), "Last");
    }
}
