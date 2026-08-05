//! 框架属性门面。
//!
//! 对标 Spring `org.springframework.core.SpringProperties`。

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::sync::RwLock;

use crate::properties_file::parse_properties;

/// 框架属性文件位置常量（对标 Spring 的 `spring.properties`，vernal 使用
/// `vernal.properties`）。
pub const PROPERTIES_RESOURCE_LOCATION: &str = "vernal.properties";

/// 框架属性门面。
///
/// 对应 Java: org.springframework.core.SpringProperties
///
/// Spring 语义：`System.getProperties` 之上的门面——启动时从类路径根
/// 加载 `vernal.properties`（对标 `spring.properties`），提供静态的
/// `getProperty` / `setProperty` / `getFlag` 访问；系统属性优先于
/// 文件属性（vernal 中以显式 `set_property` 承担系统属性语义）。
pub struct VernalProperties;

impl VernalProperties {
    /// 获取属性值。
    ///
    /// 对应 Java: `SpringProperties.getProperty(String)`
    #[must_use]
    pub fn get_property(key: &str) -> Option<String> {
        PROPERTIES
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().and_then(|map| map.get(key).cloned()))
    }

    /// 设置属性值（传 `None` 表示删除）。
    ///
    /// 对应 Java: `SpringProperties.setProperty(String, String)`
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
    /// 对应 Java: `SpringProperties.getFlag(String)`
    #[must_use]
    pub fn get_flag(key: &str) -> bool {
        matches!(
            Self::get_property(key).as_deref(),
            Some("true" | "1" | "yes" | "on")
        )
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
/// 如果文件不存在则保持空表。
///
/// 对应 Java: `SpringProperties.loadProperties(Resource)`
pub fn load_properties_from_classpath() {
    if let Ok(mut guard) = PROPERTIES.write() {
        *guard = Some(read_classpath_resource(PROPERTIES_RESOURCE_LOCATION).unwrap_or_default());
    }
}

/// 从指定路径加载属性文件。
///
/// 对应 Java: `SpringProperties.loadProperties(Resource)`
///
/// # Errors
///
/// 当文件不存在或不可读时返回 `Err`。
pub fn load_properties_from_file(path: &Path) -> Result<(), PropertiesFileError> {
    let content = std::fs::read_to_string(path)?;
    let mut map = HashMap::new();
    parse_properties(&content, &mut map);
    if let Ok(mut guard) = PROPERTIES.write() {
        *guard = Some(map);
    }
    Ok(())
}

/// 属性文件错误（对标 `java.io.IOException`）。
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

/// 读取类路径资源（最简实现）。
fn read_classpath_resource(name: &str) -> Option<HashMap<String, String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let Ok(mut file) = std::fs::File::open(path) else {
        return None;
    };
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return None;
    }
    let mut map = HashMap::new();
    parse_properties(&content, &mut map);
    Some(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_set_property() {
        // A 类（合同对齐）：对标 Spring `getProperty`/`setProperty`
        VernalProperties::set_property("vernal.test.key", Some("value"));
        assert_eq!(
            VernalProperties::get_property("vernal.test.key"),
            Some("value".to_string())
        );
        VernalProperties::set_property("vernal.test.key", None);
        assert_eq!(VernalProperties::get_property("vernal.test.key"), None);
    }

    #[test]
    fn missing_property_returns_none() {
        // B 类（边界行为）：未设置属性返回 None
        assert_eq!(VernalProperties::get_property("vernal.no.such.key"), None);
    }

    #[test]
    fn flag_semantics() {
        // A 类（合同对齐）：对标 Spring `getFlag`
        VernalProperties::set_flag("vernal.flag.on");
        assert!(VernalProperties::get_flag("vernal.flag.on"));
        VernalProperties::set_flag_value("vernal.flag.off", false);
        assert!(!VernalProperties::get_flag("vernal.flag.off"));
        VernalProperties::set_property("vernal.flag.one", Some("1"));
        assert!(VernalProperties::get_flag("vernal.flag.one"));
        assert!(!VernalProperties::get_flag("vernal.flag.missing"));
    }

    #[test]
    fn file_loading_roundtrip() {
        // D 类（生命周期/重构安全）：文件加载走 parse_properties
        let dir = std::env::temp_dir();
        let path = dir.join(format!("vernal-test-{}.properties", std::process::id()));
        std::fs::write(&path, "a=1\n# comment\nb=two\n").unwrap();
        load_properties_from_file(&path).unwrap();
        assert_eq!(VernalProperties::get_property("a"), Some("1".to_string()));
        assert_eq!(VernalProperties::get_property("b"), Some("two".to_string()));
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn missing_file_returns_error() {
        // C 类（错误路径）：文件缺失报错
        let result = load_properties_from_file(Path::new("/definitely/not/here.properties"));
        assert!(result.is_err());
    }

    #[test]
    fn resource_location_is_vernal_properties() {
        // A 类（合同对齐）：vernal 不使用 spring 品牌资源名
        assert_eq!(PROPERTIES_RESOURCE_LOCATION, "vernal.properties");
    }
}
