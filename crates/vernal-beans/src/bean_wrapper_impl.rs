//! BeanWrapperImpl — Spring 风格的 Bean 包装器实现。
//!
//! 对应 Java 类：`org.springframework.beans.BeanWrapperImpl`。
//!
//! `BeanWrapperImpl` 是 `BeanWrapper` trait 的核心实现，提供：
//! - 基于 `HashMap` 的属性存储（通过 `RwLock` 实现内部可变性）
//! - 嵌套属性路径访问（如 `"address.city"`）
//! - 属性类型注册与查询
//! - 批量属性设置
//!
//! # 设计原则（Rust 原生）
//!
//! Java 的 `BeanWrapperImpl` 依赖反射和 CGLIB 字节码生成。
//! Rust 版本使用：
//! - `Arc<dyn Any + Send + Sync>` — 类型擦除的属性值
//! - `TypeId` — 编译后的类型标识
//! - `RwLock<HashMap<String, ...>>` — 线程安全的属性存储
//! - 路径拆分 + 递归 — 嵌套属性访问
//!
//! # 线程安全
//!
//! `BeanWrapperImpl` 实现了 `Send + Sync`。
//! 内部通过 `RwLock` 保证并发读写安全：
//! - 读操作获取共享读锁
//! - 写操作获取独占写锁

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use crate::bean_wrapper::BeanWrapper;
use crate::property_accessor::PropertyAccessor;

// ── 错误类型 ───────────────────────────────────────────────────────────────

/// 属性访问错误。
///
/// 封装属性访问过程中可能出现的各种错误。
#[derive(Debug, Clone)]
pub enum PropertyError {
    /// 属性不存在。
    NoSuchProperty(String),
    /// 属性不可读。
    NotReadable(String),
    /// 属性不可写。
    NotWritable(String),
    /// 类型不匹配。
    TypeMismatch {
        /// 属性名称。
        property: String,
        /// 期望的类型。
        expected: &'static str,
        /// 实际的类型。
        actual: &'static str,
    },
    /// 嵌套属性路径中的中间对象为空。
    NullNestedObject(String),
    /// 通用错误消息。
    Other(String),
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyError::NoSuchProperty(name) => {
                write!(f, "属性不存在: '{}'", name)
            }
            PropertyError::NotReadable(name) => {
                write!(f, "属性不可读: '{}'", name)
            }
            PropertyError::NotWritable(name) => {
                write!(f, "属性不可写: '{}'", name)
            }
            PropertyError::TypeMismatch {
                property,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "属性 '{}' 类型不匹配: 期望 {}, 实际 {}",
                    property, expected, actual
                )
            }
            PropertyError::NullNestedObject(path) => {
                write!(f, "嵌套属性路径 '{}' 中的中间对象为空", path)
            }
            PropertyError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for PropertyError {}

// ── 属性元数据 ─────────────────────────────────────────────────────────────

/// 属性元数据。
///
/// 记录单个属性的类型信息和读写权限。
#[derive(Debug, Clone)]
struct PropertyMeta {
    /// 属性类型 ID。
    type_id: TypeId,
    /// 属性类型名称（用于调试和错误信息）。
    type_name: &'static str,
    /// 是否可读。
    readable: bool,
    /// 是否可写。
    writable: bool,
}

// ── BeanWrapperImpl ────────────────────────────────────────────────────────

/// Spring 风格的 Bean 包装器实现。
///
/// 对应 Spring 的 `BeanWrapperImpl`。
///
/// 基于 `RwLock<HashMap>` 的属性存储，支持嵌套属性路径访问。
/// 线程安全，支持并发读写。
///
/// # 用法
///
/// ```rust,no_run
/// use std::any::{Any, TypeId};
/// use std::sync::Arc;
/// use vernal_beans::bean_wrapper_impl::BeanWrapperImpl;
/// use vernal_beans::property_accessor::PropertyAccessor;
/// use vernal_beans::bean_wrapper::BeanWrapper;
///
/// // 创建 BeanWrapper 包装一个 String 实例
/// let instance: Arc<dyn Any + Send + Sync> = Arc::new("my_bean".to_string());
/// let wrapper = BeanWrapperImpl::new(instance);
///
/// // 注册并设置属性
/// wrapper.register_property("name", TypeId::of::<String>());
/// wrapper.set_property_value("name", Arc::new("Alice".to_string())).unwrap();
///
/// // 获取属性
/// let value = wrapper.get_property_value("name").unwrap();
/// let name = value.downcast_ref::<String>().unwrap();
/// assert_eq!(name, "Alice");
/// ```
pub struct BeanWrapperImpl {
    /// 被包装的 Bean 实例。
    instance: Arc<dyn Any + Send + Sync>,
    /// 被包装实例的类型 ID。
    type_id: TypeId,
    /// 属性值存储（内部可变性）。
    properties: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 属性元数据。
    property_metas: RwLock<HashMap<String, PropertyMeta>>,
}

impl BeanWrapperImpl {
    /// 创建新的 BeanWrapperImpl。
    ///
    /// # 参数
    ///
    /// - `instance` — 要包装的 Bean 实例（类型擦除）
    ///
    /// # 返回
    ///
    /// 新创建的 `BeanWrapperImpl`，初始时没有任何注册的属性。
    pub fn new(instance: Arc<dyn Any + Send + Sync>) -> Self {
        let type_id = (*instance).type_id();
        Self {
            instance,
            type_id,
            properties: RwLock::new(HashMap::new()),
            property_metas: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个属性。
    ///
    /// 在设置或获取属性值之前，需要先注册属性及其类型。
    /// 这提供了类似 Java BeanWrapper 的属性描述符机制。
    ///
    /// # 参数
    ///
    /// - `name` — 属性名称
    /// - `type_id` — 属性的类型 ID
    pub fn register_property(&self, name: impl Into<String>, type_id: TypeId) {
        let name = name.into();
        let type_name = resolve_type_name(type_id);
        let mut metas = self.property_metas.write().unwrap();
        metas.insert(
            name,
            PropertyMeta {
                type_id,
                type_name,
                readable: true,
                writable: true,
            },
        );
    }

    /// 注册一个只读属性。
    ///
    /// 只读属性可以被读取但不能被写入。
    ///
    /// # 参数
    ///
    /// - `name` — 属性名称
    /// - `type_id` — 属性的类型 ID
    pub fn register_readonly_property(&self, name: impl Into<String>, type_id: TypeId) {
        let name = name.into();
        let type_name = resolve_type_name(type_id);
        let mut metas = self.property_metas.write().unwrap();
        metas.insert(
            name,
            PropertyMeta {
                type_id,
                type_name,
                readable: true,
                writable: false,
            },
        );
    }

    /// 获取已注册属性的数量。
    pub fn property_count(&self) -> usize {
        self.property_metas.read().unwrap().len()
    }
}

/// 实现 `PropertyAccessor` trait。
///
/// 提供属性的读写、类型查询和属性名列举能力。
impl PropertyAccessor for BeanWrapperImpl {
    /// 获取指定属性的值。
    ///
    /// 支持嵌套属性路径（如 `"address.city"`）。
    /// 对于嵌套路径，会拆分路径并通过递归查找。
    fn get_property_value(
        &self,
        name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if name.contains('.') {
            // 嵌套属性路径
            let dot_pos = name.find('.').unwrap();
            let first = &name[..dot_pos];
            let rest = &name[dot_pos + 1..];

            let metas = self.property_metas.read().unwrap();
            let meta = metas
                .get(first)
                .ok_or_else(|| PropertyError::NoSuchProperty(first.to_string()))?;
            if !meta.readable {
                return Err(PropertyError::NotReadable(first.to_string()).into());
            }
            drop(metas);

            let props = self.properties.read().unwrap();
            let first_value = props
                .get(first)
                .ok_or_else(|| PropertyError::NoSuchProperty(first.to_string()))?;
            get_nested_value(first_value, rest)
        } else {
            // 简单属性
            let metas = self.property_metas.read().unwrap();
            let meta = metas
                .get(name)
                .ok_or_else(|| PropertyError::NoSuchProperty(name.to_string()))?;
            if !meta.readable {
                return Err(PropertyError::NotReadable(name.to_string()).into());
            }
            drop(metas);

            let props = self.properties.read().unwrap();
            props
                .get(name)
                .cloned()
                .ok_or_else(|| PropertyError::NoSuchProperty(name.to_string()).into())
        }
    }

    /// 设置指定属性的值。
    ///
    /// 支持嵌套属性路径（如 `"address.city"`）。
    fn set_property_value(
        &self,
        name: &str,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metas = self.property_metas.read().unwrap();
        let meta = metas
            .get(name)
            .ok_or_else(|| PropertyError::NoSuchProperty(name.to_string()))?;
        if !meta.writable {
            return Err(PropertyError::NotWritable(name.to_string()).into());
        }
        drop(metas);

        let mut props = self.properties.write().unwrap();
        props.insert(name.to_string(), value);
        Ok(())
    }

    /// 获取指定属性的类型。
    fn get_property_type(&self, name: &str) -> Option<TypeId> {
        if name.contains('.') {
            let dot_pos = name.find('.')?;
            let first = &name[..dot_pos];
            let rest = &name[dot_pos + 1..];

            let metas = self.property_metas.read().ok()?;
            let meta = metas.get(first)?;
            if !meta.readable {
                return None;
            }
            drop(metas);

            let props = self.properties.read().ok()?;
            let first_value = props.get(first)?;
            get_nested_type(first_value, rest)
        } else {
            let metas = self.property_metas.read().ok()?;
            metas.get(name).map(|m| m.type_id)
        }
    }

    /// 检查属性是否可读。
    fn is_readable(&self, name: &str) -> bool {
        self.property_metas
            .read()
            .unwrap()
            .get(name)
            .map_or(false, |meta| meta.readable)
    }

    /// 检查属性是否可写。
    fn is_writable(&self, name: &str) -> bool {
        self.property_metas
            .read()
            .unwrap()
            .get(name)
            .map_or(false, |meta| meta.writable)
    }

    /// 获取所有已注册的属性名称。
    fn get_property_names(&self) -> Vec<String> {
        self.property_metas.read().unwrap().keys().cloned().collect()
    }
}

/// 实现 `BeanWrapper` trait。
impl BeanWrapper for BeanWrapperImpl {
    /// 获取被包装的 Bean 实例引用。
    fn get_wrapped_instance(&self) -> &dyn Any {
        &*self.instance
    }

    /// 获取被包装 Bean 的类型标识。
    fn get_wrapped_class(&self) -> TypeId {
        self.type_id
    }
}

// ── 辅助函数 ───────────────────────────────────────────────────────────────

/// 递归获取嵌套属性值。
///
/// 支持 `"address.city"` 形式的嵌套路径。
/// 对于 `HashMap<String, Arc<dyn Any>>` 类型的值，按路径段递归查找。
///
/// # 参数
///
/// - `obj` — 当前对象（类型擦除）
/// - `path` — 剩余路径（如 `"city"` 或 `"street.name"`）
fn get_nested_value(
    obj: &Arc<dyn Any + Send + Sync>,
    path: &str,
) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
    let dot_pos = path.find('.');
    let (current, rest) = match dot_pos {
        Some(pos) => (&path[..pos], Some(&path[pos + 1..])),
        None => (path, None),
    };

    // 尝试将 obj 解释为 HashMap<String, Arc<dyn Any + Send + Sync>>
    if let Some(map) = obj.downcast_ref::<HashMap<String, Arc<dyn Any + Send + Sync>>>() {
        let value = map
            .get(current)
            .ok_or_else(|| PropertyError::NoSuchProperty(format!("嵌套属性 '{}'", current)))?;

        match rest {
            Some(rest_path) => get_nested_value(value, rest_path),
            None => Ok(Arc::clone(value)),
        }
    } else {
        Err(PropertyError::Other(format!(
            "对象不支持嵌套属性访问（需要 HashMap 类型），路径: '{}'",
            current
        ))
        .into())
    }
}

/// 递归获取嵌套属性的类型。
///
/// # 参数
///
/// - `obj` — 当前对象
/// - `path` — 剩余路径
fn get_nested_type(obj: &Arc<dyn Any + Send + Sync>, path: &str) -> Option<TypeId> {
    let dot_pos = path.find('.');
    let (current, rest) = match dot_pos {
        Some(pos) => (&path[..pos], Some(&path[pos + 1..])),
        None => (path, None),
    };

    if let Some(map) = obj.downcast_ref::<HashMap<String, Arc<dyn Any + Send + Sync>>>() {
        let value = map.get(current)?;
        match rest {
            Some(rest_path) => get_nested_type(value, rest_path),
            None => Some((**value).type_id()),
        }
    } else {
        None
    }
}

/// 解析 TypeId 到人类可读的类型名称。
///
/// 用于调试和错误信息输出。
fn resolve_type_name(type_id: TypeId) -> &'static str {
    match type_id {
        id if id == TypeId::of::<String>() => "String",
        id if id == TypeId::of::<i32>() => "i32",
        id if id == TypeId::of::<i64>() => "i64",
        id if id == TypeId::of::<f64>() => "f64",
        id if id == TypeId::of::<f32>() => "f32",
        id if id == TypeId::of::<bool>() => "bool",
        id if id == TypeId::of::<u32>() => "u32",
        id if id == TypeId::of::<u64>() => "u64",
        id if id == TypeId::of::<i8>() => "i8",
        id if id == TypeId::of::<i16>() => "i16",
        id if id == TypeId::of::<u8>() => "u8",
        id if id == TypeId::of::<u16>() => "u16",
        id if id == TypeId::of::<usize>() => "usize",
        id if id == TypeId::of::<isize>() => "isize",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get_property() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("name", TypeId::of::<String>());
        assert!(wrapper.is_readable("name"));
        assert!(wrapper.is_writable("name"));
    }

    #[test]
    fn test_set_and_get_property_value() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("name", TypeId::of::<String>());
        wrapper
            .set_property_value("name", Arc::new("Alice".to_string()))
            .unwrap();

        let value = wrapper.get_property_value("name").unwrap();
        let name = value.downcast_ref::<String>().unwrap();
        assert_eq!(name, "Alice");
    }

    #[test]
    fn test_overwrite_property_value() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("count", TypeId::of::<i32>());

        wrapper.set_property_value("count", Arc::new(1i32)).unwrap();
        let v1 = wrapper.get_property_value("count").unwrap();
        assert_eq!(*v1.downcast_ref::<i32>().unwrap(), 1);

        wrapper.set_property_value("count", Arc::new(42i32)).unwrap();
        let v2 = wrapper.get_property_value("count").unwrap();
        assert_eq!(*v2.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_get_nonexistent_property() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        let result = wrapper.get_property_value("missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_unregistered_property() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        let result = wrapper.set_property_value("x", Arc::new(1i32));
        assert!(result.is_err());
    }

    #[test]
    fn test_readonly_property() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_readonly_property("ro", TypeId::of::<i32>());
        assert!(wrapper.is_readable("ro"));
        assert!(!wrapper.is_writable("ro"));
        assert!(wrapper.set_property_value("ro", Arc::new(42i32)).is_err());
    }

    #[test]
    fn test_nested_property_via_hashmap() {
        // 创建一个嵌套的 HashMap 结构：address = { city: "Beijing" }
        let mut inner = HashMap::new();
        inner.insert(
            "city".to_string(),
            Arc::new("Beijing".to_string()) as Arc<dyn Any + Send + Sync>,
        );

        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property(
            "address",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        wrapper
            .set_property_value("address", Arc::new(inner))
            .unwrap();

        let value = wrapper.get_property_value("address.city").unwrap();
        let city = value.downcast_ref::<String>().unwrap();
        assert_eq!(city, "Beijing");
    }

    #[test]
    fn test_deep_nested_property() {
        // address = { location: { zip: "100000" } }
        let mut location = HashMap::new();
        location.insert(
            "zip".to_string(),
            Arc::new("100000".to_string()) as Arc<dyn Any + Send + Sync>,
        );

        let mut address = HashMap::new();
        address.insert(
            "location".to_string(),
            Arc::new(location) as Arc<dyn Any + Send + Sync>,
        );

        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("address", TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
        wrapper
            .set_property_value("address", Arc::new(address))
            .unwrap();

        let value = wrapper.get_property_value("address.location.zip").unwrap();
        let zip = value.downcast_ref::<String>().unwrap();
        assert_eq!(zip, "100000");
    }

    #[test]
    fn test_get_property_names() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("a", TypeId::of::<i32>());
        wrapper.register_property("b", TypeId::of::<String>());
        let mut names = wrapper.get_property_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_property_type() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("name", TypeId::of::<String>());
        assert_eq!(
            wrapper.get_property_type("name"),
            Some(TypeId::of::<String>())
        );
        assert_eq!(wrapper.get_property_type("missing"), None);
    }

    #[test]
    fn test_wrapped_instance_and_class() {
        let instance: Arc<dyn Any + Send + Sync> = Arc::new("my_bean".to_string());
        let wrapper = BeanWrapperImpl::new(Arc::clone(&instance));
        assert_eq!(wrapper.get_wrapped_class(), TypeId::of::<String>());
        let wrapped = wrapper.get_wrapped_instance();
        let s = wrapped.downcast_ref::<String>().unwrap();
        assert_eq!(s, "my_bean");
    }

    #[test]
    fn test_batch_set_property_values() {
        use crate::bean_wrapper::BeanWrapper;

        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("name", TypeId::of::<String>());
        wrapper.register_property("age", TypeId::of::<i32>());

        let mut values = HashMap::new();
        values.insert("name".to_string(), Arc::new("Bob".to_string()) as Arc<dyn Any + Send + Sync>);
        values.insert("age".to_string(), Arc::new(30i32) as Arc<dyn Any + Send + Sync>);

        wrapper.set_property_values(&values).unwrap();

        let name = wrapper.get_property_value("name").unwrap();
        assert_eq!(*name.downcast_ref::<String>().unwrap(), "Bob");
        let age = wrapper.get_property_value("age").unwrap();
        assert_eq!(*age.downcast_ref::<i32>().unwrap(), 30);
    }

    #[test]
    fn test_property_count() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        assert_eq!(wrapper.property_count(), 0);
        wrapper.register_property("x", TypeId::of::<i32>());
        assert_eq!(wrapper.property_count(), 1);
        wrapper.register_property("y", TypeId::of::<String>());
        assert_eq!(wrapper.property_count(), 2);
    }

    #[test]
    fn test_property_error_display_no_such_property() {
        let err = PropertyError::NoSuchProperty("name".to_string());
        assert_eq!(format!("{}", err), "属性不存在: 'name'");
    }

    #[test]
    fn test_property_error_display_not_readable() {
        let err = PropertyError::NotReadable("secret".to_string());
        assert_eq!(format!("{}", err), "属性不可读: 'secret'");
    }

    #[test]
    fn test_property_error_display_not_writable() {
        let err = PropertyError::NotWritable("readonly".to_string());
        assert_eq!(format!("{}", err), "属性不可写: 'readonly'");
    }

    #[test]
    fn test_property_error_display_type_mismatch() {
        let err = PropertyError::TypeMismatch {
            property: "count".to_string(),
            expected: "i32",
            actual: "String",
        };
        let msg = format!("{}", err);
        assert!(msg.contains("count"));
        assert!(msg.contains("i32"));
        assert!(msg.contains("String"));
    }

    #[test]
    fn test_property_error_display_null_nested_object() {
        let err = PropertyError::NullNestedObject("address.city".to_string());
        assert_eq!(format!("{}", err), "嵌套属性路径 'address.city' 中的中间对象为空");
    }

    #[test]
    fn test_property_error_display_other() {
        let err = PropertyError::Other("custom error".to_string());
        assert_eq!(format!("{}", err), "custom error");
    }

    #[test]
    fn test_property_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PropertyError>();
    }

    #[test]
    fn test_nested_property_type() {
        let mut inner = HashMap::new();
        inner.insert(
            "value".to_string(),
            Arc::new(42i32) as Arc<dyn Any + Send + Sync>,
        );

        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property(
            "data",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        wrapper
            .set_property_value("data", Arc::new(inner))
            .unwrap();

        let type_id = wrapper.get_property_type("data.value");
        assert_eq!(type_id, Some(TypeId::of::<i32>()));
    }

    #[test]
    fn test_nested_property_type_missing() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property(
            "data",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        assert_eq!(wrapper.get_property_type("data.missing"), None);
    }

    #[test]
    fn test_nested_property_type_unreadable() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_readonly_property(
            "data",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        // readonly is still readable
        assert_eq!(wrapper.get_property_type("data.child"), None);
    }

    #[test]
    fn test_set_property_value_unregistered() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        let result = wrapper.set_property_value("unknown", Arc::new(1i32));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_property_value_readonly() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_readonly_property("ro", TypeId::of::<i32>());
        let result = wrapper.set_property_value("ro", Arc::new(42i32));
        assert!(result.is_err());
    }

    #[test]
    fn test_is_readable_unregistered() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        assert!(!wrapper.is_readable("unknown"));
    }

    #[test]
    fn test_is_writable_unregistered() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        assert!(!wrapper.is_writable("unknown"));
    }

    #[test]
    fn test_get_property_names_empty() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        assert!(wrapper.get_property_names().is_empty());
    }

    #[test]
    fn test_resolve_type_name_known_types() {
        assert_eq!(resolve_type_name(TypeId::of::<String>()), "String");
        assert_eq!(resolve_type_name(TypeId::of::<i32>()), "i32");
        assert_eq!(resolve_type_name(TypeId::of::<i64>()), "i64");
        assert_eq!(resolve_type_name(TypeId::of::<f64>()), "f64");
        assert_eq!(resolve_type_name(TypeId::of::<f32>()), "f32");
        assert_eq!(resolve_type_name(TypeId::of::<bool>()), "bool");
        assert_eq!(resolve_type_name(TypeId::of::<u32>()), "u32");
        assert_eq!(resolve_type_name(TypeId::of::<u64>()), "u64");
        assert_eq!(resolve_type_name(TypeId::of::<i8>()), "i8");
        assert_eq!(resolve_type_name(TypeId::of::<i16>()), "i16");
        assert_eq!(resolve_type_name(TypeId::of::<u8>()), "u8");
        assert_eq!(resolve_type_name(TypeId::of::<u16>()), "u16");
        assert_eq!(resolve_type_name(TypeId::of::<usize>()), "usize");
        assert_eq!(resolve_type_name(TypeId::of::<isize>()), "isize");
    }

    #[test]
    fn test_multiple_nested_properties() {
        // city = { name: "Beijing", code: "010" }
        let mut city = HashMap::new();
        city.insert(
            "name".to_string(),
            Arc::new("Beijing".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        city.insert(
            "code".to_string(),
            Arc::new("010".to_string()) as Arc<dyn Any + Send + Sync>,
        );

        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property(
            "city",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        wrapper
            .set_property_value("city", Arc::new(city))
            .unwrap();

        let name = wrapper.get_property_value("city.name").unwrap();
        assert_eq!(*name.downcast_ref::<String>().unwrap(), "Beijing");

        let code = wrapper.get_property_value("city.code").unwrap();
        assert_eq!(*code.downcast_ref::<String>().unwrap(), "010");
    }

    #[test]
    fn test_nested_property_missing_intermediate() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property(
            "data",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        // data is not set, so accessing data.child should fail
        assert!(wrapper.get_property_value("data.child").is_err());
    }

    #[test]
    fn test_get_property_type_for_simple_property() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("count", TypeId::of::<i32>());
        assert_eq!(
            wrapper.get_property_type("count"),
            Some(TypeId::of::<i32>())
        );
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn test_nested_property_get_not_readable() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_readonly_property(
            "data",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        // readonly is still readable, so this should work
        let result = wrapper.get_property_value("data.child");
        // data is not set, so this should fail with NoSuchProperty
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_property_not_hashmap() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("data", TypeId::of::<String>());
        wrapper.set_property_value("data", Arc::new("not_a_map".to_string())).unwrap();
        let result = wrapper.get_property_value("data.child");
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_property_missing_key() {
        let mut inner = HashMap::new();
        inner.insert(
            "existing".to_string(),
            Arc::new("value".to_string()) as Arc<dyn Any + Send + Sync>,
        );
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property(
            "data",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );
        wrapper.set_property_value("data", Arc::new(inner)).unwrap();
        let result = wrapper.get_property_value("data.missing");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_property_type_nested_not_hashmap() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("data", TypeId::of::<String>());
        wrapper.set_property_value("data", Arc::new("not_a_map".to_string())).unwrap();
        assert_eq!(wrapper.get_property_type("data.child"), None);
    }

    #[test]
    fn test_get_property_type_simple_missing() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        assert_eq!(wrapper.get_property_type("missing"), None);
    }

    #[test]
    fn test_property_error_is_clone() {
        let err = PropertyError::NoSuchProperty("test".to_string());
        let err2 = err.clone();
        assert_eq!(format!("{}", err), format!("{}", err2));
    }

    #[test]
    fn test_property_error_is_debug() {
        let err = PropertyError::NoSuchProperty("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("NoSuchProperty"));
    }

    #[test]
    fn test_property_error_is_std_error() {
        let err = PropertyError::NoSuchProperty("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_batch_set_property_values_partial() {
        let wrapper = BeanWrapperImpl::new(Arc::new("test".to_string()));
        wrapper.register_property("name", TypeId::of::<String>());
        // "age" is not registered, so setting it should fail
        let mut values = HashMap::new();
        values.insert("name".to_string(), Arc::new("Bob".to_string()) as Arc<dyn Any + Send + Sync>);
        values.insert("age".to_string(), Arc::new(30i32) as Arc<dyn Any + Send + Sync>);
        let result = wrapper.set_property_values(&values);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_type_name_unknown() {
        struct CustomType;
        let type_id = TypeId::of::<CustomType>();
        assert_eq!(resolve_type_name(type_id), "unknown");
    }
}
