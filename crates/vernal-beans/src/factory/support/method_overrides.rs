//! MethodOverrides — Spring 风格的方法覆盖集合。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.MethodOverrides`。
//!
//! 在 Spring 中，`MethodOverrides` 持有一个 Bean 定义上的所有方法覆盖。
//! 它用于在 Bean 实例化后应用方法替换。

use crate::factory::support::method_override::MethodOverride;
use std::collections::HashMap;
use std::collections::hash_map::Keys;

/// 方法覆盖集合。
///
/// 对应 Spring 的 `MethodOverrides`。
///
/// 管理一个 Bean 定义上的所有方法覆盖。
#[derive(Debug, Default)]
pub struct MethodOverrides {
    /// 方法名 -> 方法覆盖的映射
    overrides: HashMap<String, Box<dyn MethodOverride>>,
}

impl MethodOverrides {
    /// 创建空的方法覆盖集合。
    pub fn new() -> Self { Self::default() }

    /// 添加方法覆盖。
    ///
    /// 如果已有同名覆盖，会被替换。
    pub fn add(&mut self, override_obj: Box<dyn MethodOverride>) {
        let name = override_obj.get_method_name().to_string();
        self.overrides.insert(name, override_obj);
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool { self.overrides.is_empty() }

    /// 覆盖数量。
    pub fn len(&self) -> usize { self.overrides.len() }

    /// 获取指定方法名的覆盖。
    pub fn get(&self, name: &str) -> Option<&(dyn MethodOverride + 'static)> {
        self.overrides.get(name).map(|b| b.as_ref())
    }

    /// 是否包含指定方法名的覆盖。
    pub fn contains(&self, name: &str) -> bool {
        self.overrides.contains_key(name)
    }

    /// 移除指定方法名的覆盖。
    pub fn remove(&mut self, name: &str) -> Option<Box<dyn MethodOverride>> {
        self.overrides.remove(name)
    }

    /// 获取所有被覆盖的方法名。
    pub fn method_names(&self) -> Keys<'_, String, Box<dyn MethodOverride>> {
        self.overrides.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::support::method_override::SimpleMethodOverride;

    #[test]
    fn empty_overrides() {
        let overrides = MethodOverrides::new();
        assert!(overrides.is_empty());
        assert_eq!(overrides.len(), 0);
    }

    #[test]
    fn add_and_get_override() {
        let mut overrides = MethodOverrides::new();
        overrides.add(Box::new(SimpleMethodOverride::new("doWork")));

        assert!(!overrides.is_empty());
        assert_eq!(overrides.len(), 1);
        assert!(overrides.contains("doWork"));

        let mo = overrides.get("doWork").unwrap();
        assert_eq!(mo.get_method_name(), "doWork");
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let overrides = MethodOverrides::new();
        assert!(overrides.get("missing").is_none());
        assert!(!overrides.contains("missing"));
    }

    #[test]
    fn remove_override() {
        let mut overrides = MethodOverrides::new();
        overrides.add(Box::new(SimpleMethodOverride::new("test")));
        assert!(overrides.contains("test"));

        let removed = overrides.remove("test");
        assert!(removed.is_some());
        assert!(!overrides.contains("test"));
    }

    #[test]
    fn method_names_list() {
        let mut overrides = MethodOverrides::new();
        overrides.add(Box::new(SimpleMethodOverride::new("a")));
        overrides.add(Box::new(SimpleMethodOverride::new("b")));
        overrides.add(Box::new(SimpleMethodOverride::new("c")));

        let mut names: Vec<&String> = overrides.method_names().collect();
        names.sort();
        assert_eq!(names, vec!["a", "b", "c"]);
    }
}
