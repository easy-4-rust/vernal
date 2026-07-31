//! AbstractNestablePropertyAccessor — 支持嵌套属性路径的属性访问器。
//!
//! 对应 Java 类：`org.springframework.beans.AbstractNestablePropertyAccessor`。
//!
//! 扩展 `AbstractPropertyAccessor`，增加对嵌套属性路径的支持。
//! 例如 `"address.city"` 会自动拆分为 `"address"` + `"city"`，
//! 然后递归访问。
//!
//! # 设计原则（Rust 原生）
//!
//! Java 的 `AbstractNestablePropertyAccessor` 使用反射遍历嵌套对象。
//! Rust 中通过以下方式实现：
//! - 属性值使用 `Arc<dyn Any + Send + Sync>` 类型擦除
//! - 嵌套对象必须是 `HashMap<String, Arc<dyn Any + Send + Sync>>` 类型
//! - 路径拆分 + 递归访问实现嵌套导航
//!
//! # 嵌套路径语法
//!
//! - `"name"` — 简单属性
//! - `"address.city"` — 两级嵌套
//! - `"address.location.zip"` — 多级嵌套
//!
//! # 限制
//!
//! 当前实现要求嵌套对象为 `HashMap<String, Arc<dyn Any + Send + Sync>>` 类型。
//! 不支持通过 struct 字段直接访问（Rust 没有运行时反射）。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::property_accessor::PropertyAccessor;

/// 支持嵌套属性路径的属性访问器。
///
/// 对应 Spring 的 `AbstractNestablePropertyAccessor`。
///
/// 组合 `HashMap` 属性存储 + 嵌套路径递归访问。
///
/// # 用法
///
/// ```rust,no_run
/// use std::any::{Any, TypeId};
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use vernal_beans::abstract_nestable_property_accessor::AbstractNestablePropertyAccessor;
/// use vernal_beans::property_accessor::PropertyAccessor;
///
/// let accessor = AbstractNestablePropertyAccessor::new();
///
/// // 注册并设置嵌套属性
/// accessor.register_property("address", TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
/// let mut address: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
/// address.insert("city".to_string(), Arc::new("Beijing".to_string()));
/// accessor.set_property_value("address", Arc::new(address)).unwrap();
///
/// // 嵌套路径访问
/// let value = accessor.get_property_value("address.city").unwrap();
/// let city = value.downcast_ref::<String>().unwrap();
/// assert_eq!(city, "Beijing");
/// ```
pub struct AbstractNestablePropertyAccessor {
    /// 属性值存储。
    properties: RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 属性类型映射。
    property_types: RwLock<HashMap<String, TypeId>>,
    /// 可读属性映射。
    readable: RwLock<HashMap<String, bool>>,
    /// 可写属性映射。
    writable: RwLock<HashMap<String, bool>>,
}

impl AbstractNestablePropertyAccessor {
    /// 创建空的 AbstractNestablePropertyAccessor。
    pub fn new() -> Self {
        Self {
            properties: RwLock::new(HashMap::new()),
            property_types: RwLock::new(HashMap::new()),
            readable: RwLock::new(HashMap::new()),
            writable: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个属性。
    ///
    /// # 参数
    ///
    /// - `name` — 属性名称（不包含嵌套路径分隔符）
    /// - `type_id` — 属性类型 ID
    pub fn register_property(&self, name: impl Into<String>, type_id: TypeId) {
        let name = name.into();
        self.property_types
            .write()
            .unwrap()
            .insert(name.clone(), type_id);
        self.readable.write().unwrap().insert(name.clone(), true);
        self.writable.write().unwrap().insert(name, true);
    }

    /// 注册一个只读属性。
    pub fn register_readonly_property(&self, name: impl Into<String>, type_id: TypeId) {
        let name = name.into();
        self.property_types
            .write()
            .unwrap()
            .insert(name.clone(), type_id);
        self.readable.write().unwrap().insert(name.clone(), true);
        self.writable.write().unwrap().insert(name, false);
    }

    /// 检查属性是否已注册。
    pub fn has_property(&self, name: &str) -> bool {
        // 只检查根属性（不含 '.')
        let root = name.split('.').next().unwrap_or(name);
        self.property_types.read().unwrap().contains_key(root)
    }

    /// 获取已注册属性的数量。
    pub fn property_count(&self) -> usize {
        self.property_types.read().unwrap().len()
    }

    /// 检查是否为嵌套属性路径。
    fn is_nested_path(name: &str) -> bool {
        name.contains('.')
    }
}

impl Default for AbstractNestablePropertyAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyAccessor for AbstractNestablePropertyAccessor {
    /// 获取指定属性的值。
    ///
    /// 支持嵌套属性路径（如 `"address.city"`）。
    /// 对于嵌套路径，会拆分路径段并递归查找。
    ///
    /// # 实现
    ///
    /// 1. 如果路径包含 `'.'`，拆分为首段和剩余路径
    /// 2. 从 properties 中获取首段的值
    /// 3. 将首段值解释为 `HashMap<String, Arc<dyn Any>>`
    /// 4. 在 HashMap 中查找剩余路径（递归）
    fn get_property_value(
        &self,
        name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        if Self::is_nested_path(name) {
            let dot_pos = name.find('.').unwrap();
            let first = &name[..dot_pos];
            let rest = &name[dot_pos + 1..];

            // 检查首段属性是否注册
            if !self.has_property(first) {
                return Err(format!("属性不存在: '{}'", first).into());
            }

            // 检查首段是否可读
            {
                let readable = self.readable.read().unwrap();
                if !readable.get(first).copied().unwrap_or(false) {
                    return Err(format!("属性不可读: '{}'", first).into());
                }
            }

            // 获取首段值
            let first_value = {
                let props = self.properties.read().unwrap();
                props
                    .get(first)
                    .cloned()
                    .ok_or_else(|| format!("属性值未设置: '{}'", first))?
            };

            // 递归访问嵌套路径
            resolve_nested(&first_value, rest)
        } else {
            // 简单属性
            if !self.has_property(name) {
                return Err(format!("属性不存在: '{}'", name).into());
            }

            {
                let readable = self.readable.read().unwrap();
                if !readable.get(name).copied().unwrap_or(false) {
                    return Err(format!("属性不可读: '{}'", name).into());
                }
            }

            let props = self.properties.read().unwrap();
            props
                .get(name)
                .cloned()
                .ok_or_else(|| format!("属性值未设置: '{}'", name).into())
        }
    }

    /// 设置指定属性的值。
    ///
    /// 不支持嵌套路径设置（只能设置根属性）。
    /// 嵌套属性应先设置完整的根属性值。
    fn set_property_value(
        &self,
        name: &str,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 不支持嵌套路径的写入
        if Self::is_nested_path(name) {
            return Err(
                format!("不支持通过嵌套路径设置属性: '{}'，请先设置根属性", name).into(),
            );
        }

        if !self.has_property(name) {
            return Err(format!("属性不存在: '{}'", name).into());
        }

        {
            let writable = self.writable.read().unwrap();
            if !writable.get(name).copied().unwrap_or(false) {
                return Err(format!("属性不可写: '{}'", name).into());
            }
        }

        let mut props = self.properties.write().unwrap();
        props.insert(name.to_string(), value);
        Ok(())
    }

    fn get_property_type(&self, name: &str) -> Option<TypeId> {
        if Self::is_nested_path(name) {
            let dot_pos = name.find('.')?;
            let first = &name[..dot_pos];
            let rest = &name[dot_pos + 1..];

            let props = self.properties.read().ok()?;
            let first_value = props.get(first)?;
            resolve_nested_type(first_value, rest)
        } else {
            self.property_types.read().unwrap().get(name).copied()
        }
    }

    fn is_readable(&self, name: &str) -> bool {
        if Self::is_nested_path(name) {
            // 对于嵌套路径，只检查首段
            let root = name.split('.').next().unwrap_or(name);
            self.readable
                .read()
                .unwrap()
                .get(root)
                .copied()
                .unwrap_or(false)
        } else {
            self.readable
                .read()
                .unwrap()
                .get(name)
                .copied()
                .unwrap_or(false)
        }
    }

    fn is_writable(&self, name: &str) -> bool {
        if Self::is_nested_path(name) {
            // 嵌套路径不支持写入
            false
        } else {
            self.writable
                .read()
                .unwrap()
                .get(name)
                .copied()
                .unwrap_or(false)
        }
    }

    fn get_property_names(&self) -> Vec<String> {
        self.property_types.read().unwrap().keys().cloned().collect()
    }
}

// ── 辅助函数 ───────────────────────────────────────────────────────────────

/// 递归解析嵌套属性路径，获取最终值。
///
/// # 参数
///
/// - `obj` — 当前对象（类型擦除）
/// - `path` — 剩余路径（如 `"city"` 或 `"street.name"`）
///
/// # 实现
///
/// 要求中间对象为 `HashMap<String, Arc<dyn Any + Send + Sync>>` 类型。
fn resolve_nested(
    obj: &Arc<dyn Any + Send + Sync>,
    path: &str,
) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
    let dot_pos = path.find('.');
    let (current, rest) = match dot_pos {
        Some(pos) => (&path[..pos], Some(&path[pos + 1..])),
        None => (path, None),
    };

    // 尝试将 obj 解释为 HashMap
    if let Some(map) = obj.downcast_ref::<HashMap<String, Arc<dyn Any + Send + Sync>>>() {
        let value = map
            .get(current)
            .ok_or_else(|| format!("嵌套属性不存在: '{}'", current))?;

        match rest {
            Some(rest_path) => resolve_nested(value, rest_path),
            None => Ok(Arc::clone(value)),
        }
    } else {
        Err(format!("对象不支持嵌套属性访问（需要 HashMap 类型），路径: '{}'", current).into())
    }
}

/// 递归解析嵌套属性类型。
fn resolve_nested_type(
    obj: &Arc<dyn Any + Send + Sync>,
    path: &str,
) -> Option<TypeId> {
    let dot_pos = path.find('.');
    let (current, rest) = match dot_pos {
        Some(pos) => (&path[..pos], Some(&path[pos + 1..])),
        None => (path, None),
    };

    if let Some(map) = obj.downcast_ref::<HashMap<String, Arc<dyn Any + Send + Sync>>>() {
        let value = map.get(current)?;
        match rest {
            Some(rest_path) => resolve_nested_type(value, rest_path),
            None => Some((**value).type_id()),
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_property() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("name", TypeId::of::<String>());
        accessor
            .set_property_value("name", Arc::new("Alice".to_string()))
            .unwrap();

        let value = accessor.get_property_value("name").unwrap();
        assert_eq!(*value.downcast_ref::<String>().unwrap(), "Alice");
    }

    #[test]
    fn test_nested_property_two_levels() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property(
            "address",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );

        let mut address: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        address.insert("city".to_string(), Arc::new("Shanghai".to_string()));
        accessor
            .set_property_value("address", Arc::new(address))
            .unwrap();

        let value = accessor.get_property_value("address.city").unwrap();
        assert_eq!(*value.downcast_ref::<String>().unwrap(), "Shanghai");
    }

    #[test]
    fn test_nested_property_three_levels() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property(
            "address",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );

        let mut location: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        location.insert("zip".to_string(), Arc::new("100000".to_string()));

        let mut address: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        address.insert("location".to_string(), Arc::new(location));
        accessor
            .set_property_value("address", Arc::new(address))
            .unwrap();

        let value = accessor
            .get_property_value("address.location.zip")
            .unwrap();
        assert_eq!(*value.downcast_ref::<String>().unwrap(), "100000");
    }

    #[test]
    fn test_nested_write_not_supported() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property(
            "address",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );

        let result = accessor.set_property_value("address.city", Arc::new("X".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_nonexistent() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property(
            "address",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );

        let mut address: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        address.insert("city".to_string(), Arc::new("Beijing".to_string()));
        accessor
            .set_property_value("address", Arc::new(address))
            .unwrap();

        let result = accessor.get_property_value("address.country");
        assert!(result.is_err());
    }

    #[test]
    fn test_readonly_property() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_readonly_property("ro", TypeId::of::<i32>());
        assert!(accessor.is_readable("ro"));
        assert!(!accessor.is_writable("ro"));
        assert!(accessor.set_property_value("ro", Arc::new(1i32)).is_err());
    }

    #[test]
    fn test_property_names() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("a", TypeId::of::<i32>());
        accessor.register_property("b", TypeId::of::<String>());
        let mut names = accessor.get_property_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_nested_type_resolution() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property(
            "data",
            TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>(),
        );

        let mut data: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        data.insert("count".to_string(), Arc::new(42i64));
        accessor
            .set_property_value("data", Arc::new(data))
            .unwrap();

        assert_eq!(
            accessor.get_property_type("data.count"),
            Some(TypeId::of::<i64>())
        );
    }

    #[test]
    fn test_default_trait() {
        let accessor = AbstractNestablePropertyAccessor::default();
        assert_eq!(accessor.property_count(), 0);
    }

    #[test]
    fn test_property_count() {
        let accessor = AbstractNestablePropertyAccessor::new();
        assert_eq!(accessor.property_count(), 0);
        accessor.register_property("a", TypeId::of::<i32>());
        assert_eq!(accessor.property_count(), 1);
        accessor.register_property("b", TypeId::of::<String>());
        assert_eq!(accessor.property_count(), 2);
    }

    #[test]
    fn test_has_property_nested() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("address", TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
        assert!(accessor.has_property("address.city"));
        assert!(accessor.has_property("address"));
    }

    #[test]
    fn test_has_property_unregistered() {
        let accessor = AbstractNestablePropertyAccessor::new();
        assert!(!accessor.has_property("unknown"));
    }

    #[test]
    fn test_is_readable_registered() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("name", TypeId::of::<String>());
        assert!(accessor.is_readable("name"));
    }

    #[test]
    fn test_is_readable_unregistered() {
        let accessor = AbstractNestablePropertyAccessor::new();
        assert!(!accessor.is_readable("unknown"));
    }

    #[test]
    fn test_is_writable_registered() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("name", TypeId::of::<String>());
        assert!(accessor.is_writable("name"));
    }

    #[test]
    fn test_is_writable_nested_path_returns_false() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("address", TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
        assert!(!accessor.is_writable("address.city"));
    }

    #[test]
    fn test_is_readable_nested_path() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("address", TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
        assert!(accessor.is_readable("address.city"));
    }

    #[test]
    fn test_set_property_value_not_registered() {
        let accessor = AbstractNestablePropertyAccessor::new();
        let result = accessor.set_property_value("unknown", Arc::new(42i32));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_property_value_not_registered() {
        let accessor = AbstractNestablePropertyAccessor::new();
        let result = accessor.get_property_value("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_property_value_not_set() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("name", TypeId::of::<String>());
        let result = accessor.get_property_value("name");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_property_value_readonly() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_readonly_property("ro", TypeId::of::<i32>());
        accessor.set_property_value("ro", Arc::new(42i32)).unwrap_err();
    }

    #[test]
    fn test_get_property_type_simple() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("count", TypeId::of::<i32>());
        assert_eq!(accessor.get_property_type("count"), Some(TypeId::of::<i32>()));
    }

    #[test]
    fn test_get_property_type_unregistered() {
        let accessor = AbstractNestablePropertyAccessor::new();
        assert!(accessor.get_property_type("unknown").is_none());
    }

    #[test]
    fn test_get_property_type_nested_not_found() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("data", TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());
        let mut data: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        data.insert("key".to_string(), Arc::new("val".to_string()));
        accessor.set_property_value("data", Arc::new(data)).unwrap();
        assert!(accessor.get_property_type("data.nonexistent").is_none());
    }

    #[test]
    fn test_nested_non_hashmap_type() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("name", TypeId::of::<String>());
        accessor.set_property_value("name", Arc::new("hello".to_string())).unwrap();
        let result = accessor.get_property_value("name.inner");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_property_type_nested_non_hashmap() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("name", TypeId::of::<String>());
        accessor.set_property_value("name", Arc::new("hello".to_string())).unwrap();
        assert!(accessor.get_property_type("name.inner").is_none());
    }

    #[test]
    fn test_deeply_nested_three_levels() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_property("root", TypeId::of::<HashMap<String, Arc<dyn Any + Send + Sync>>>());

        let mut level3: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        level3.insert("value".to_string(), Arc::new(42i32));

        let mut level2: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        level2.insert("inner".to_string(), Arc::new(level3));

        let mut level1: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        level1.insert("mid".to_string(), Arc::new(level2));

        accessor.set_property_value("root", Arc::new(level1)).unwrap();

        let val = accessor.get_property_value("root.mid.inner.value").unwrap();
        assert_eq!(*val.downcast_ref::<i32>().unwrap(), 42);
    }

    #[test]
    fn test_register_readonly_then_try_write() {
        let accessor = AbstractNestablePropertyAccessor::new();
        accessor.register_readonly_property("config", TypeId::of::<String>());
        assert!(accessor.is_readable("config"));
        assert!(!accessor.is_writable("config"));
        let result = accessor.set_property_value("config", Arc::new("value".to_string()));
        assert!(result.is_err());
    }
}
