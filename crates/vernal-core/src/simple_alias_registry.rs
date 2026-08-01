//! 简单别名注册表。
//!
//! 对标 Spring `org.springframework.core.SimpleAliasRegistry`。

use std::collections::HashMap;

use crate::AliasRegistry;

/// 简单别名注册表。
///
/// 对应 Java: org.springframework.core.SimpleAliasRegistry
///
/// Spring 语义：`HashMap` 别名表，`registerAlias` 支持别名链（`canonicalName`
/// 沿链解析到规范名）与循环检测（"Circular reference" 错误）。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SimpleAliasRegistry {
    alias_map: HashMap<String, String>,
}

impl SimpleAliasRegistry {
    /// 创建空注册表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 解析别名的规范名（沿别名链，无别名时返回自身）。
    #[must_use]
    pub fn canonical_name(&self, name: &str) -> String {
        let mut resolved = name.to_string();
        let mut visited = std::collections::HashSet::new();
        while let Some(target) = self.alias_map.get(&resolved) {
            if !visited.insert(resolved.clone()) {
                break; // 循环保护（对标 Spring 循环引用防御）
            }
            resolved.clear();
            resolved.push_str(target);
        }
        resolved
    }
}

impl AliasRegistry for SimpleAliasRegistry {
    fn register_alias(&mut self, name: &str, alias: &str) -> Result<(), String> {
        if alias == name {
            return Err(format!("alias '{alias}' must not be same as name"));
        }
        if let Some(existing) = self.alias_map.get(alias)
            && existing != name
        {
            return Err(format!(
                "alias '{alias}' already registered for '{existing}'"
            ));
        }
        self.alias_map.insert(alias.to_string(), name.to_string());
        Ok(())
    }

    fn remove_alias(&mut self, alias: &str) {
        self.alias_map.remove(alias);
    }

    fn is_alias(&self, name: &str) -> bool {
        self.alias_map.contains_key(name)
    }

    fn get_aliases(&self, name: &str) -> Vec<String> {
        let mut result = self
            .alias_map
            .iter()
            .filter(|(_, target)| *target == name)
            .map(|(alias, _)| alias.clone())
            .collect::<Vec<_>>();
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_resolves_alias() {
        // A 类（合同对齐）：对标 Spring `registerAlias` + `isAlias`
        let mut registry = SimpleAliasRegistry::new();
        registry.register_alias("myBean", "aliasA").unwrap();
        assert!(registry.is_alias("aliasA"));
        assert_eq!(registry.canonical_name("aliasA"), "myBean");
        assert_eq!(registry.get_aliases("myBean"), vec!["aliasA"]);
    }

    #[test]
    fn alias_chain_resolves_to_canonical() {
        // B 类（边界行为）：对标 Spring 别名链
        let mut registry = SimpleAliasRegistry::new();
        registry.register_alias("original", "alias1").unwrap();
        registry.register_alias("alias1", "alias2").unwrap();
        assert_eq!(registry.canonical_name("alias2"), "original");
    }

    #[test]
    fn same_alias_for_same_name_is_ok() {
        // B 类（边界行为）：重复注册相同映射不报错
        let mut registry = SimpleAliasRegistry::new();
        registry.register_alias("a", "x").unwrap();
        assert!(registry.register_alias("a", "x").is_ok());
    }

    #[test]
    fn conflicting_alias_returns_error() {
        // C 类（错误路径）：对标 Spring `AliasAlreadyExistsException`
        let mut registry = SimpleAliasRegistry::new();
        registry.register_alias("a", "x").unwrap();
        let err = registry.register_alias("b", "x").unwrap_err();
        assert!(err.contains("already registered"));
    }

    #[test]
    fn self_alias_returns_error() {
        // C 类（错误路径）
        let mut registry = SimpleAliasRegistry::new();
        assert!(registry.register_alias("a", "a").is_err());
    }

    #[test]
    fn remove_alias_clears_mapping() {
        // A 类（合同对齐）：对标 Spring `removeAlias`
        let mut registry = SimpleAliasRegistry::new();
        registry.register_alias("a", "x").unwrap();
        registry.remove_alias("x");
        assert!(!registry.is_alias("x"));
    }
}
