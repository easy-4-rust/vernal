//! BeanRegistry — 对应 Spring `org.springframework.beans.factory.BeanRegistry`。
//!
//! Bean 注册表接口。

/// Bean 注册表接口。
///
/// 对应 Java 接口：`org.springframework.beans.factory.BeanRegistry`。
///
/// 定义了 Bean 注册表的基本操作。
pub trait BeanRegistry: Send + Sync {
    /// 注册一个新的 Bean 定义。
    fn register_bean_definition(
        &mut self,
        bean_name: &str,
        definition: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 移除指定名称的 Bean 定义。
    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 获取指定名称的 Bean 定义。
    fn get_bean_definition(&self, bean_name: &str) -> Option<&dyn std::any::Any>;

    /// 检查是否包含指定名称的 Bean 定义。
    fn contains_bean_definition(&self, bean_name: &str) -> bool;

    /// 获取所有 Bean 定义名称。
    fn bean_definition_names(&self) -> Vec<String>;

    /// 获取 Bean 定义数量。
    fn bean_definition_count(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct SimpleBeanRegistry {
        definitions: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    }

    impl SimpleBeanRegistry {
        fn new() -> Self {
            Self {
                definitions: HashMap::new(),
            }
        }
    }

    impl BeanRegistry for SimpleBeanRegistry {
        fn register_bean_definition(
            &mut self,
            bean_name: &str,
            definition: Box<dyn std::any::Any + Send + Sync>,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.definitions.insert(bean_name.to_string(), definition);
            Ok(())
        }

        fn remove_bean_definition(
            &mut self,
            bean_name: &str,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            self.definitions.remove(bean_name);
            Ok(())
        }

        fn get_bean_definition(&self, bean_name: &str) -> Option<&dyn std::any::Any> {
            self.definitions
                .get(bean_name)
                .map(|v| v.as_ref() as &dyn std::any::Any)
        }

        fn contains_bean_definition(&self, bean_name: &str) -> bool {
            self.definitions.contains_key(bean_name)
        }

        fn bean_definition_names(&self) -> Vec<String> {
            self.definitions.keys().cloned().collect()
        }

        fn bean_definition_count(&self) -> usize {
            self.definitions.len()
        }
    }

    #[test]
    fn test_bean_registry() {
        let mut registry = SimpleBeanRegistry::new();
        assert_eq!(registry.bean_definition_count(), 0);

        registry
            .register_bean_definition("test", Box::new(String::from("value")))
            .unwrap();
        assert_eq!(registry.bean_definition_count(), 1);
        assert!(registry.contains_bean_definition("test"));
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn test_bean_registry_multiple() {
        let mut registry = SimpleBeanRegistry::new();
        registry
            .register_bean_definition("bean1", Box::new(String::from("v1")))
            .unwrap();
        registry
            .register_bean_definition("bean2", Box::new(42i32))
            .unwrap();
        assert_eq!(registry.bean_definition_count(), 2);
        assert!(registry.contains_bean_definition("bean1"));
        assert!(registry.contains_bean_definition("bean2"));
    }

    #[test]
    fn test_bean_registry_remove() {
        let mut registry = SimpleBeanRegistry::new();
        registry
            .register_bean_definition("bean1", Box::new(String::from("v1")))
            .unwrap();
        assert_eq!(registry.bean_definition_count(), 1);
        registry.remove_bean_definition("bean1").unwrap();
        assert_eq!(registry.bean_definition_count(), 0);
        assert!(!registry.contains_bean_definition("bean1"));
    }

    #[test]
    fn test_bean_registry_get() {
        let mut registry = SimpleBeanRegistry::new();
        registry
            .register_bean_definition("bean1", Box::new(String::from("v1")))
            .unwrap();
        let def = registry.get_bean_definition("bean1");
        assert!(def.is_some());
    }

    #[test]
    fn test_bean_registry_get_not_found() {
        let registry = SimpleBeanRegistry::new();
        let def = registry.get_bean_definition("missing");
        assert!(def.is_none());
    }

    #[test]
    fn test_bean_registry_names() {
        let mut registry = SimpleBeanRegistry::new();
        registry
            .register_bean_definition("bean1", Box::new(String::from("v1")))
            .unwrap();
        registry
            .register_bean_definition("bean2", Box::new(42i32))
            .unwrap();
        let names = registry.bean_definition_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"bean1".to_string()));
        assert!(names.contains(&"bean2".to_string()));
    }

    #[test]
    fn test_bean_registry_empty_names() {
        let registry = SimpleBeanRegistry::new();
        let names = registry.bean_definition_names();
        assert!(names.is_empty());
    }

    #[test]
    fn test_bean_registry_empty_get() {
        let registry = SimpleBeanRegistry::new();
        assert!(registry.get_bean_definition("missing").is_none());
    }

    #[test]
    fn test_bean_registry_empty_contains() {
        let registry = SimpleBeanRegistry::new();
        assert!(!registry.contains_bean_definition("missing"));
    }
}
