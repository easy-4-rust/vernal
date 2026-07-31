//! CustomScopeConfigurer — 对应 Spring `org.springframework.beans.factory.config.CustomScopeConfigurer`。
//!
//! 自定义作用域配置器。

use std::collections::HashMap;

/// 自定义作用域配置器。
///
/// 对应 Java 类：`org.springframework.beans.factory.config.CustomScopeConfigurer`。
///
/// 用于注册自定义作用域。
#[derive(Debug, Default)]
pub struct CustomScopeConfigurer {
    scopes: HashMap<String, String>,
}

impl CustomScopeConfigurer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_scope(&mut self, name: impl Into<String>, scope_type: impl Into<String>) {
        self.scopes.insert(name.into(), scope_type.into());
    }

    pub fn scope_names(&self) -> Vec<String> {
        self.scopes.keys().cloned().collect()
    }

    pub fn has_scope(&self, name: &str) -> bool {
        self.scopes.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configurer() {
        let mut configurer = CustomScopeConfigurer::new();
        assert!(!configurer.has_scope("session"));

        configurer.register_scope("session", "SessionScope");
        assert!(configurer.has_scope("session"));
        assert_eq!(configurer.scope_names().len(), 1);
    }
}
