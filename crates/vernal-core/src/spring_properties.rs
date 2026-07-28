//! 框架级属性持有者。
//!
//! 对标 Spring `org.springframework.core.SpringProperties`。
//!
//! Spring 从 classpath 根目录读取 `spring.properties` 文件，作为框架级本地属性的
//! 静态持有者。Rust 无 classpath 概念，改为从当前工作目录或可执行文件同级目录
//! 读取 `spring.properties`，并回退到环境变量（对标 Spring 回退到 JVM 系统属性）。
//!
//! 该模块用 `RwLock<HashMap>` 保证线程安全，对标 Spring 的静态 `Properties` 字段。

use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

/// `spring.properties` 文件的默认查找路径。
///
/// 对标 Spring `SpringProperties.PROPERTIES_RESOURCE_LOCATION = "spring.properties"`。
pub const PROPERTIES_RESOURCE_LOCATION: &str = "spring.properties";

/// 框架级本地属性的线程安全持有者。
///
/// 对标 Spring `org.springframework.core.SpringProperties`。
///
/// 优先级：本地属性 > 环境变量（对标 Spring：localProperties > System.getProperty）。
pub struct SpringProperties;

struct Inner {
    local: HashMap<String, String>,
}

fn inner() -> &'static RwLock<Inner> {
    static INSTANCE: OnceLock<RwLock<Inner>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let mut local = HashMap::new();
        // 尝试从文件加载（对标 Spring static 初始化块）
        if let Some(content) = load_from_file() {
            parse_properties(&content, &mut local);
        }
        RwLock::new(Inner { local })
    })
}

/// 尝试从 `spring.properties` 文件加载内容。
///
/// 查找顺序：当前工作目录 → 可执行文件同级目录。
fn load_from_file() -> Option<String> {
    // 当前工作目录
    let cwd_path = PathBuf::from(PROPERTIES_RESOURCE_LOCATION);
    if let Ok(mut file) = std::fs::File::open(&cwd_path) {
        let mut content = String::new();
        if file.read_to_string(&mut content).is_ok() {
            return Some(content);
        }
    }
    // 可执行文件同级目录
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let exe_path = dir.join(PROPERTIES_RESOURCE_LOCATION);
            if let Ok(mut file) = std::fs::File::open(&exe_path) {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    return Some(content);
                }
            }
        }
    }
    None
}

/// 解析 .properties 格式（简化版，仅 `=` 分隔 + `#`/`!` 注释）。
fn parse_properties(content: &str, map: &mut HashMap<String, String>) {
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

impl SpringProperties {
    /// 编程式设置本地属性，覆盖 `spring.properties` 文件中的条目。
    ///
    /// 对标 Spring `setProperty(key, value)`。
    /// `value` 为 `None` 时移除该属性（对标 Java `null` 重置）。
    pub fn set_property(key: &str, value: Option<&str>) {
        let mut guard = inner().write().unwrap();
        match value {
            Some(v) => {
                guard.local.insert(key.to_string(), v.to_string());
            }
            None => {
                guard.local.remove(key);
            }
        }
    }

    /// 检索属性值，先查本地属性，再回退到环境变量。
    ///
    /// 对标 Spring `getProperty(key)`。
    #[must_use]
    pub fn get_property(key: &str) -> Option<String> {
        // 先查本地属性
        {
            let guard = inner().read().unwrap();
            if let Some(v) = guard.local.get(key) {
                return Some(v.clone());
            }
        }
        // 回退到环境变量（对标 Spring 回退到 System.getProperty）
        std::env::var(key).ok()
    }

    /// 设置标志位为 `"true"`。
    ///
    /// 对标 Spring `setFlag(key)`。
    pub fn set_flag(key: &str) {
        Self::set_property(key, Some("true"));
    }

    /// 设置标志位为指定布尔值。
    ///
    /// 对标 Spring `setFlag(key, value)`（since 6.2.6）。
    pub fn set_flag_value(key: &str, value: bool) {
        Self::set_property(key, Some(if value { "true" } else { "false" }));
    }

    /// 检索标志位，值为 `"true"`（忽略大小写）时返回 `true`。
    ///
    /// 对标 Spring `getFlag(key)`。
    #[must_use]
    pub fn get_flag(key: &str) -> bool {
        Self::get_property(key)
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }

    /// 检索标志位，未设置时返回 `None`。
    ///
    /// 对标 Spring `checkFlag(key)`（since 6.2.6）。
    #[must_use]
    pub fn check_flag(key: &str) -> Option<bool> {
        Self::get_property(key).map(|v| v.eq_ignore_ascii_case("true"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_property() {
        SpringProperties::set_property("test.springprop.key", Some("value123"));
        assert_eq!(
            SpringProperties::get_property("test.springprop.key"),
            Some("value123".to_string())
        );
    }

    #[test]
    fn set_property_none_removes() {
        SpringProperties::set_property("test.springprop.remove", Some("x"));
        assert!(SpringProperties::get_property("test.springprop.remove").is_some());
        SpringProperties::set_property("test.springprop.remove", None);
        assert!(SpringProperties::get_property("test.springprop.remove").is_none());
    }

    #[test]
    fn get_property_missing_returns_none() {
        assert!(SpringProperties::get_property("nonexistent.springprop.key").is_none());
    }

    #[test]
    fn set_and_get_flag() {
        SpringProperties::set_flag("test.springprop.flag");
        assert!(SpringProperties::get_flag("test.springprop.flag"));
    }

    #[test]
    fn set_flag_value_false() {
        SpringProperties::set_flag_value("test.springprop.flagfalse", false);
        assert!(!SpringProperties::get_flag("test.springprop.flagfalse"));
    }

    #[test]
    fn check_flag_returns_none_when_unset() {
        assert!(SpringProperties::check_flag("test.springprop.unset").is_none());
    }

    #[test]
    fn check_flag_returns_some_when_set() {
        SpringProperties::set_flag_value("test.springprop.checkflag", true);
        assert_eq!(
            SpringProperties::check_flag("test.springprop.checkflag"),
            Some(true)
        );
    }

    #[test]
    fn flag_case_insensitive_true() {
        SpringProperties::set_property("test.springprop.casetest", Some("TRUE"));
        assert!(SpringProperties::get_flag("test.springprop.casetest"));
    }

    #[test]
    fn fallback_to_env_var() {
        // PATH 环境变量应该存在
        assert!(SpringProperties::get_property("PATH").is_some());
    }

    #[test]
    fn local_overrides_env() {
        // PATH 是系统环境变量，应该可以读取
        let env_val = SpringProperties::get_property("PATH");
        assert!(env_val.is_some(), "PATH 应该存在于系统环境中");
        // 用 set_property 覆盖 PATH，验证本地属性优先
        SpringProperties::set_property("PATH", Some("overridden"));
        assert_eq!(SpringProperties::get_property("PATH"), Some("overridden".to_string()));
        // 清理
        SpringProperties::set_property("PATH", None);
    }

    #[test]
    fn properties_resource_location_constant() {
        assert_eq!(PROPERTIES_RESOURCE_LOCATION, "spring.properties");
    }

    #[test]
    fn parse_properties_function() {
        let mut map = std::collections::HashMap::new();
        parse_properties("key1=value1
# comment
key2=value2

! c
k3=v3", &mut map);
        assert_eq!(map.get("key1").map(String::as_str), Some("value1"));
        assert_eq!(map.get("key2").map(String::as_str), Some("value2"));
        assert_eq!(map.get("k3").map(String::as_str), Some("v3"));
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn parse_properties_trim() {
        let mut map = std::collections::HashMap::new();
        parse_properties("  key  =  value  ", &mut map);
        assert_eq!(map.get("key").map(String::as_str), Some("value"));
    }

    #[test]
    fn parse_properties_empty() {
        let mut map = std::collections::HashMap::new();
        parse_properties("", &mut map);
        assert!(map.is_empty());
    }


}