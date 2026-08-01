//! 方法类键。
//!
//! 对标 Spring `org.springframework.core.MethodClassKey`。

use std::hash::{Hash, Hasher};

/// 方法类键。
///
/// 对应 Java: org.springframework.core.MethodClassKey
///
/// Spring 语义：`(方法名, 类名)` 复合键，用于缓存键（对标 Spring 按方法
/// 名与声明类缓存元数据的键）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodClassKey {
    method_name: String,
    class_name: String,
}

impl MethodClassKey {
    /// 创建方法类键。
    #[must_use]
    pub fn new(method_name: impl Into<String>, class_name: impl Into<String>) -> Self {
        Self {
            method_name: method_name.into(),
            class_name: class_name.into(),
        }
    }

    /// 返回方法名。
    #[must_use]
    pub fn method_name(&self) -> &str {
        &self.method_name
    }

    /// 返回类名。
    #[must_use]
    pub fn class_name(&self) -> &str {
        &self.class_name
    }
}

impl Hash for MethodClassKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.method_name.hash(state);
        self.class_name.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn equal_keys_compare_equal() {
        // A 类（合同对齐）：对标 Spring 键相等性
        let a = MethodClassKey::new("toString", "java.lang.Object");
        let b = MethodClassKey::new("toString", "java.lang.Object");
        assert_eq!(a, b);
    }

    #[test]
    fn different_methods_are_different_keys() {
        // B 类（边界行为）
        let a = MethodClassKey::new("toString", "java.lang.Object");
        let b = MethodClassKey::new("hashCode", "java.lang.Object");
        assert_ne!(a, b);
    }

    #[test]
    fn works_as_hash_map_key() {
        // D 类（重构安全）：对标 Spring 缓存键用途
        let mut cache = HashMap::new();
        cache.insert(MethodClassKey::new("run", "com.example.Task"), 1_usize);
        assert_eq!(
            cache.get(&MethodClassKey::new("run", "com.example.Task")),
            Some(&1)
        );
    }
}
