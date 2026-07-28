//! 可配置 MIME 文件类型映射 — 对标 `ConfigurableMimeFileTypeMap`。

use std::collections::HashMap;
use std::path::Path;

/// 可配置 MIME 文件类型映射。
///
/// 对标 Spring 的 `ConfigurableMimeFileTypeMap`，从 mime.types 格式的映射中读取
/// 文件扩展名到 MIME 类型的映射。
pub struct ConfigurableMimeFileTypeMap {
    /// 自定义映射（扩展名 -> MIME 类型）
    custom_mappings: HashMap<String, String>,
    /// 默认映射
    default_mappings: HashMap<String, String>,
}

impl ConfigurableMimeFileTypeMap {
    /// 创建默认的 MIME 文件类型映射。
    pub fn new() -> Self {
        let mut default_mappings = HashMap::new();
        // 常见 MIME 类型
        default_mappings.insert("html".to_string(), "text/html".to_string());
        default_mappings.insert("htm".to_string(), "text/html".to_string());
        default_mappings.insert("css".to_string(), "text/css".to_string());
        default_mappings.insert("js".to_string(), "application/javascript".to_string());
        default_mappings.insert("json".to_string(), "application/json".to_string());
        default_mappings.insert("xml".to_string(), "application/xml".to_string());
        default_mappings.insert("txt".to_string(), "text/plain".to_string());
        default_mappings.insert("pdf".to_string(), "application/pdf".to_string());
        default_mappings.insert("zip".to_string(), "application/zip".to_string());
        default_mappings.insert("png".to_string(), "image/png".to_string());
        default_mappings.insert("jpg".to_string(), "image/jpeg".to_string());
        default_mappings.insert("jpeg".to_string(), "image/jpeg".to_string());
        default_mappings.insert("gif".to_string(), "image/gif".to_string());
        default_mappings.insert("svg".to_string(), "image/svg+xml".to_string());
        default_mappings.insert("mp3".to_string(), "audio/mpeg".to_string());
        default_mappings.insert("mp4".to_string(), "video/mp4".to_string());
        default_mappings.insert("doc".to_string(), "application/msword".to_string());
        default_mappings.insert(
            "docx".to_string(),
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        );
        default_mappings.insert("xls".to_string(), "application/vnd.ms-excel".to_string());
        default_mappings.insert(
            "xlsx".to_string(),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        );

        Self {
            custom_mappings: HashMap::new(),
            default_mappings,
        }
    }

    /// 添加自定义 MIME 映射。
    pub fn add_mapping(&mut self, extension: String, mime_type: String) {
        self.custom_mappings.insert(extension, mime_type);
    }

    /// 从 mime.types 格式的字符串加载映射。
    pub fn load_from_mappings(&mut self, mappings: &[String]) {
        for mapping in mappings {
            let parts: Vec<&str> = mapping.split_whitespace().collect();
            if parts.len() >= 2 {
                let mime_type = parts[0].to_string();
                for ext in &parts[1..] {
                    self.custom_mappings
                        .insert(ext.to_string(), mime_type.clone());
                }
            }
        }
    }

    /// 获取文件的 MIME 类型。
    pub fn get_content_type(&self, filename: &str) -> String {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        self.custom_mappings
            .get(&ext)
            .or_else(|| self.default_mappings.get(&ext))
            .cloned()
            .unwrap_or_else(|| "application/octet-stream".to_string())
    }
}

impl Default for ConfigurableMimeFileTypeMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_types() {
        let map = ConfigurableMimeFileTypeMap::new();
        assert_eq!(map.get_content_type("test.html"), "text/html");
        assert_eq!(map.get_content_type("test.pdf"), "application/pdf");
        assert_eq!(map.get_content_type("test.png"), "image/png");
    }

    #[test]
    fn test_custom_mapping() {
        let mut map = ConfigurableMimeFileTypeMap::new();
        map.add_mapping("xyz".to_string(), "application/custom".to_string());
        assert_eq!(map.get_content_type("test.xyz"), "application/custom");
    }

    #[test]
    fn test_unknown_extension() {
        let map = ConfigurableMimeFileTypeMap::new();
        assert_eq!(
            map.get_content_type("test.unknown"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_load_from_mappings() {
        let mut map = ConfigurableMimeFileTypeMap::new();
        map.load_from_mappings(&["text/custom ext1 ext2".to_string()]);
        assert_eq!(map.get_content_type("test.ext1"), "text/custom");
        assert_eq!(map.get_content_type("test.ext2"), "text/custom");
    }
}
