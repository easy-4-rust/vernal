//! 框架属性文件加载器。
//!
//! 提供从类路径加载 `vernal.properties` 文件的功能，
//! 用于读取框架级别的开关配置（如 trace 开关、实验性功能标志）。
//!
//! 设计说明：与 Spring 的 `org.springframework.core.SpringProperties` 等价语义，
//! 但去掉 Spring 品牌，使用框架中性的命名。

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::RwLock;

/// 框架属性文件位置常量。
pub const PROPERTIES_RESOURCE_LOCATION: &str = "vernal.properties";

/// 框架属性文件加载器。
///
/// 启动时从类路径读取 `vernal.properties`，提供静态 getter/setter。
/// 对应语义（Spring 迁移）：`org.springframework.core.SpringProperties`
pub struct FrameworkProperties;

impl FrameworkProperties {
    /// 获取属性值。
    ///
    /// 对应语义（Spring 迁移）：`SpringProperties.getProperty(String)`
    #[must_use]
    pub fn get_property(key: &str) -> Option<String> {
        PROPERTIES.read().ok().and_then(|guard| {
            guard.as_ref().and_then(|map| map.get(key).cloned())
        })
    }

    /// 设置属性值。
    ///
    /// 传 `None` 表示删除该属性。
    /// 对应语义（Spring 迁移）：`SpringProperties.setProperty(String, String)`
    pub fn set_property(key: &str, value: Option<&str>) {
        if let Ok(mut guard) = PROPERTIES.write() {
            let map = guard.get_or_insert_with(HashMap::new);
            match value {
                Some(v) => map.insert(key.to_string(), v.to_string()),
                None => map.remove(key),
            };
        }
    }

    /// 获取布尔标志。
    ///
    /// 对应语义（Spring 迁移）：`SpringProperties.getFlag(String)`
    #[must_use]
    pub fn get_flag(key: &str) -> bool {
        match Self::get_property(key).as_deref() {
            Some("true") | Some("1") | Some("yes") | Some("on") => true,
            _ => false,
        }
    }

    /// 设置布尔标志为 true。
    pub fn set_flag(key: &str) {
        Self::set_flag_value(key, true);
    }

    /// 设置布尔标志为指定值。
    pub fn set_flag_value(key: &str, value: bool) {
        Self::set_property(key, Some(if value { "true" } else { "false" }));
    }
}

/// 全局属性表（懒加载）。
static PROPERTIES: RwLock<Option<HashMap<String, String>>> = RwLock::new(None);

/// 加载类路径下的 `vernal.properties`。
///
/// 如果文件不存在则保持空表。对应语义（Spring 迁移）：
/// `SpringProperties.loadProperties(Resource)`。
pub fn load_properties_from_classpath() {
    if let Ok(mut guard) = PROPERTIES.write() {
        *guard = Some(read_classpath_resource(PROPERTIES_RESOURCE_LOCATION).unwrap_or_default());
    }
}

/// 从指定路径加载属性文件。
///
/// 对应语义（Spring 迁移）：`SpringProperties.loadProperties(Resource)`
///
/// # Errors
///
/// 当文件不存在时返回 `Err`。
pub fn load_properties_from_file(path: &Path) -> Result<(), PropertiesFileError> {
    let content = std::fs::read_to_string(path)?;
    let mut map = HashMap::new();
    parse_properties(&content, &mut map);
    if let Ok(mut guard) = PROPERTIES.write() {
        *guard = Some(map);
    }
    Ok(())
}

/// 属性文件错误。
#[derive(Debug)]
pub enum PropertiesFileError {
    /// IO 错误（对标 `java.io.IOException`）
    Io(std::io::Error),
}

impl std::fmt::Display for PropertiesFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "属性文件 IO 错误: {e}"),
        }
    }
}

impl std::error::Error for PropertiesFileError {}

impl From<std::io::Error> for PropertiesFileError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// 解析 .properties 格式文本。
///
/// 格式：`key=value` 每行一对，`#`/`!` 开头为注释。
pub fn parse_properties(content: &str, map: &mut HashMap<String, String>) {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
}

/// 读取类路径资源（最简实现）。
fn read_classpath_resource(name: &str) -> Option<HashMap<String, String>> {
    // vernal 运行时没有真正的 classpath，使用相对路径打开文件作为简化
    let path = std::path::Path::new(name);
    let mut file = std::fs::File::open(path).ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;
    let mut map = HashMap::new();
    parse_properties(&content, &mut map);
    Some(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_properties_basic() {
        let mut map = HashMap::new();
        parse_properties("key1=value1
# comment
key2=value2

! c2
k3=v3", &mut map);
        assert_eq!(map.get("key1").map(String::as_str), Some("value1"));
        assert_eq!(map.get("key2").map(String::as_str), Some("value2"));
        assert_eq!(map.get("k3").map(String::as_str), Some("v3"));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn parse_properties_trim() {
        let mut map = HashMap::new();
        parse_properties("  key  =  value  ", &mut map);
        assert_eq!(map.get("key").map(String::as_str), Some("value"));
    }

    #[test]
    fn parse_properties_empty() {
        let mut map = HashMap::new();
        parse_properties("", &mut map);
        assert!(map.is_empty());
    }

    #[test]
    fn properties_resource_location_constant() {
        assert_eq!(PROPERTIES_RESOURCE_LOCATION, "vernal.properties");
    }

    #[test]
    fn set_then_get_then_clear() {
        // 先设置值
        FrameworkProperties::set_property("test.prop.local", Some("original"));
        assert_eq!(
            FrameworkProperties::get_property("test.prop.local"),
            Some("original".to_string())
        );
        // 覆盖
        FrameworkProperties::set_property("test.prop.local", Some("overridden"));
        assert_eq!(
            FrameworkProperties::get_property("test.prop.local"),
            Some("overridden".to_string())
        );
        // 清理
        FrameworkProperties::set_property("test.prop.local", None);
        assert!(FrameworkProperties::get_property("test.prop.local").is_none());
    }

    #[test]
    fn set_property_then_get() {
        FrameworkProperties::set_property("test.prop.key", Some("value123"));
        assert_eq!(
            FrameworkProperties::get_property("test.prop.key"),
            Some("value123".to_string())
        );
        FrameworkProperties::set_property("test.prop.key", None);
    }

    #[test]
    fn remove_property() {
        FrameworkProperties::set_property("test.prop.remove", Some("x"));
        assert!(FrameworkProperties::get_property("test.prop.remove").is_some());
        FrameworkProperties::set_property("test.prop.remove", None);
        assert!(FrameworkProperties::get_property("test.prop.remove").is_none());
    }

    #[test]
    fn missing_property_returns_none() {
        assert!(FrameworkProperties::get_property("nonexistent.prop.key").is_none());
    }

    #[test]
    fn flag_set_and_get() {
        FrameworkProperties::set_flag("test.prop.flag");
        assert!(FrameworkProperties::get_flag("test.prop.flag"));
        FrameworkProperties::set_flag_value("test.prop.flagfalse", false);
        assert!(!FrameworkProperties::get_flag("test.prop.flagfalse"));
        FrameworkProperties::set_property("test.prop.flag", None);
        FrameworkProperties::set_property("test.prop.flagfalse", None);
    }
}
