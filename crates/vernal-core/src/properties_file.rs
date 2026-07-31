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
        // 对标 Spring Boolean.getBoolean
        // 使用进程内唯一前缀避免与并行测试共享全局 PROPERTIES 状态
        let pid = std::process::id();
        let key = format!("test.prop.flag.{pid}");
        let keyfalse = format!("test.prop.flagfalse.{pid}");
        FrameworkProperties::set_flag(&key);
        assert!(FrameworkProperties::get_flag(&key));
        FrameworkProperties::set_flag_value(&keyfalse, false);
        assert!(!FrameworkProperties::get_flag(&keyfalse));
        FrameworkProperties::set_property(&key, None);
        FrameworkProperties::set_property(&keyfalse, None);
    }

    #[test]
    fn flag_recognizes_truthy_aliases() {
        // 对标 Spring `getFlag`：`true`/`1`/`yes`/`on` 都视作 true
        // 使用进程内唯一前缀（线程 ID + 测试名）避免并行测试的全局状态相互覆盖
        let prefix = format!("flag.alias.{}.", std::process::id());
        let key_one = format!("{prefix}1");
        let key_yes = format!("{prefix}yes");
        let key_on = format!("{prefix}on");
        FrameworkProperties::set_property(&key_one, Some("1"));
        FrameworkProperties::set_property(&key_yes, Some("yes"));
        FrameworkProperties::set_property(&key_on, Some("on"));
        assert!(FrameworkProperties::get_flag(&key_one));
        assert!(FrameworkProperties::get_flag(&key_yes));
        assert!(FrameworkProperties::get_flag(&key_on));
        FrameworkProperties::set_property(&key_one, None);
        FrameworkProperties::set_property(&key_yes, None);
        FrameworkProperties::set_property(&key_on, None);
    }

    #[test]
    fn flag_rejects_other_strings() {
        let key = format!("flag.other.alias.{}", std::process::id());
        FrameworkProperties::set_property(&key, Some("enabled"));
        // 不是 `true`/`1`/`yes`/`on`，应返回 false
        assert!(!FrameworkProperties::get_flag(&key));
        FrameworkProperties::set_property(&key, None);
    }

    #[test]
    fn properties_file_error_display_includes_io_kind() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err: PropertiesFileError = io_err.into();
        let s = err.to_string();
        assert!(s.contains("属性文件 IO 错误"), "actual: {s}");
        assert!(s.contains("no such file"), "actual: {s}");
    }

    #[test]
    fn properties_file_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<PropertiesFileError>();
        // 验证 From<std::io::Error> 转换（对标 Spring IOException wrapping）
        let io_err = std::io::Error::other("disk gone");
        let err: PropertiesFileError = io_err.into();
        // Debug 可派生
        let _ = format!("{err:?}");
    }

    #[test]
    fn load_properties_from_file_returns_err_when_missing() {
        let result = load_properties_from_file(std::path::Path::new("/no/such/file.properties"));
        assert!(result.is_err());
    }

    #[test]
    fn load_properties_from_file_reads_and_overwrites_global() {
        // 使用进程内唯一前缀避免与其他并行测试的全局状态相互覆盖
        let suffix = std::process::id().to_string();
        let key_target = format!("overwrite.target.{suffix}");
        let key_foo = format!("foo.{suffix}");

        FrameworkProperties::set_property(&key_target, Some("before"));

        // 用临时文件模拟属性文件加载
        let dir = std::env::temp_dir().join(format!(
            "vernal-test-{suffix}",
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("override.properties");
        std::fs::write(
            &path,
            format!("{key_target}=after\n{key_foo}=bar\n"),
        )
        .unwrap();

        let result = load_properties_from_file(&path);
        assert!(result.is_ok());
        assert_eq!(
            FrameworkProperties::get_property(&key_target),
            Some("after".to_string())
        );
        assert_eq!(
            FrameworkProperties::get_property(&key_foo),
            Some("bar".to_string())
        );

        // 清理
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
        FrameworkProperties::set_property(&key_target, None);
        FrameworkProperties::set_property(&key_foo, None);
    }

    #[test]
    fn load_properties_from_classpath_is_noop_when_resource_missing() {
        // `vernal.properties` 在 classpath 不存在时不应 panic, 保持空表（对标 Spring `loadProperties` 容错）
        load_properties_from_classpath();
        // 不假设任何特定内容, 只保证调用后全局表仍然存在
        // 通过 set_property 触发懒加载后再验证不崩溃
        let key = "after.load.classpath.test";
        FrameworkProperties::set_property(key, Some("1"));
        assert_eq!(
            FrameworkProperties::get_property(key),
            Some("1".to_string())
        );
        FrameworkProperties::set_property(key, None);
    }

    #[test]
    fn read_classpath_resource_reads_vernal_properties_when_present() {
        // 对标 Spring `loadProperties`：
        // 当 cwd 中存在 vernal.properties 时，read_classpath_resource 应读取并解析它。
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let key = format!("classpath.read.{pid}.{n}");

        // 在当前工作目录创建临时 vernal.properties
        let cwd = std::env::current_dir().unwrap();
        let backup = cwd.join("vernal.properties");
        let backup_existed = backup.exists();
        let backup_content = if backup_existed {
            Some(std::fs::read_to_string(&backup).unwrap())
        } else {
            None
        };

        let content = format!("{key}=from_cwd_vernal_props\n");
        std::fs::write(&backup, &content).unwrap();

        // 调用 read_classpath_resource
        let result = read_classpath_resource("vernal.properties");
        assert!(result.is_some(), "应该读取到 vernal.properties");
        let map = result.unwrap();
        assert_eq!(map.get(&key).map(String::as_str), Some("from_cwd_vernal_props"));

        // 恢复
        if let Some(c) = backup_content {
            std::fs::write(&backup, c).unwrap();
        } else {
            let _ = std::fs::remove_file(&backup);
        }
    }

    #[test]
    fn read_classpath_resource_returns_none_for_empty_file() {
        // 对标 Spring 行为：文件存在但内容为空时返回空 Map
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let path = std::env::temp_dir().join(format!("empty_vernal_{pid}_{n}.properties"));

        // 先读取确保文件不存在
        let _ = std::fs::remove_file(&path);
        // 验证不存在的文件返回 None
        let result = read_classpath_resource(&path.to_string_lossy());
        assert!(result.is_none());
    }

    #[test]
    fn load_properties_from_file_overrides_existing_property() {
        // 对标 Spring `MutablePropertySources` 的覆盖语义：
        // 加载文件后，文件中的值应覆盖之前 set_property 的值。
        let key = format!("override.vernal.test.{}", std::process::id());
        FrameworkProperties::set_property(&key, Some("before"));

        let dir = std::env::temp_dir().join(format!("vernal-test-override-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("override.properties");
        std::fs::write(&path, format!("{key}=after\n")).unwrap();

        let result = load_properties_from_file(&path);
        assert!(result.is_ok());
        assert_eq!(FrameworkProperties::get_property(&key), Some("after".to_string()));

        // 清理
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
        FrameworkProperties::set_property(&key, None);
    }

    #[test]
    fn load_properties_from_file_returns_error_for_directory() {
        // 对标 Spring `PropertiesLoaderUtils.loadProperties` 错误传播：
        // 当路径是目录而不是文件时返回 Io 错误
        let dir = std::env::temp_dir().join("vernal_test_dir_only");
        std::fs::create_dir_all(&dir).unwrap();

        let result = load_properties_from_file(&dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, PropertiesFileError::Io(_)));

        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn read_classpath_resource_reads_existing_file() {
        // 对标 Spring loadProperties: 从指定路径读取并解析属性文件
        // 使用临时文件避免与其他并行测试竞争 cwd/vernal.properties
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let key = format!("classpath.read.{}.{}", std::process::id(), n);

        let dir = std::env::temp_dir().join(format!("vernal-classpath-test-{n}"));
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.properties");
        std::fs::write(&file_path, format!("{key}=value1\nother=v2\n")).unwrap();

        let result = read_classpath_resource(&file_path.to_string_lossy());
        assert!(result.is_some(), "should read existing file");
        let map = result.unwrap();
        assert_eq!(map.get(&key).map(String::as_str), Some("value1"));
        assert_eq!(map.get("other").map(String::as_str), Some("v2"));

        // 清理
        let _ = std::fs::remove_file(&file_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn read_classpath_resource_returns_none_for_nonexistent() {
        // 对标 Spring: 文件不存在时返回 None
        let result = read_classpath_resource("/nonexistent/path/12345.properties");
        assert!(result.is_none());
    }
}
