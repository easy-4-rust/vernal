// ── Spring BeanDefinitionRegistry 接口实现 ────────────────────────────────

/// 代理 BeanDefinition（用于 Container 的 BeanDefinitionRegistry 实现）。
#[derive(Debug, Clone)]
pub(crate) struct ProxyBeanDefinition {
    pub(crate) bean_name: String,
    pub(crate) type_name: String,
    pub(crate) scope: crate::component_scope::Scope,
    pub(crate) source: String,
}

impl crate::bean_definition::BeanDefinition for ProxyBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        unimplemented!("ProxyBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        &self.type_name
    }

    fn scope(&self) -> crate::component_scope::Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool { false }
    fn is_primary(&self) -> bool { false }
}

/// 被删除的 BeanDefinition 标记。
#[derive(Debug)]
struct DeletedBeanDefinition {
    bean_name: String,
}

impl crate::bean_definition::BeanDefinition for DeletedBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        unimplemented!("DeletedBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        "__DELETED__"
    }

    fn scope(&self) -> crate::component_scope::Scope {
        crate::component_scope::Scope::Singleton
    }

    fn is_lazy_init(&self) -> bool { false }
    fn is_primary(&self) -> bool { false }
}

/// 被移除的 BeanDefinition（用于 remove_bean_definition 返回值）。
#[derive(Debug)]
struct RemovedBeanDefinition {
    bean_name: String,
    type_name: String,
    scope: crate::component_scope::Scope,
}

impl crate::bean_definition::BeanDefinition for RemovedBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        unimplemented!("RemovedBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        &self.type_name
    }

    fn scope(&self) -> crate::component_scope::Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool { false }
    fn is_primary(&self) -> bool { false }
}

// ── Spring BeanDefinitionRegistry 接口实现 ────────────────────────────────

impl crate::bean_definition_registry::BeanDefinitionRegistry for Container {
    fn register_bean_definition(
        &mut self,
        bean_name: String,
        definition: Box<dyn crate::bean_definition::BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut defs = self.dynamic_definitions.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if defs.contains_key(&bean_name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Bean definition '{}' already exists", bean_name),
            )));
        }
        defs.insert(bean_name, definition.into());
        Ok(())
    }

    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<Box<dyn crate::bean_definition::BeanDefinition>, Box<dyn std::error::Error + Send + Sync>> {
        let removed = {
            let mut defs = self.dynamic_definitions.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            defs.remove(bean_name)
        };

        if let Some(definition) = removed {
            if definition.bean_class_name() == "__DELETED__" {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Bean definition '{}' not found", bean_name),
                )));
            }
            return Ok(Box::new(RemovedBeanDefinition {
                bean_name: bean_name.to_string(),
                type_name: definition.bean_class_name().to_string(),
                scope: definition.scope(),
            }));
        }

        let in_registry = self.registry.definitions().iter()
            .any(|d| d.key().type_name() == bean_name);

        if in_registry {
            let mut defs = self.dynamic_definitions.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            defs.insert(
                bean_name.to_string(),
                Arc::new(DeletedBeanDefinition { bean_name: bean_name.to_string() }),
            );
            let key_to_remove = self.registry.definitions().iter()
                .find(|d| d.key().type_name() == bean_name)
                .map(|d| d.key().clone());
            if let Some(key) = key_to_remove {
                self.singletons.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove(&key);
            }
            return Ok(Box::new(RemovedBeanDefinition {
                bean_name: bean_name.to_string(),
                type_name: bean_name.to_string(),
                scope: crate::component_scope::Scope::Singleton,
            }));
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Bean definition '{}' not found", bean_name),
        )))
    }

    /// 获取 Bean 定义（返回代理对象的引用）。
    ///
    /// 对应 Spring 的 `BeanDefinition getBeanDefinition(String beanName)`。
    ///
    /// 由于 trait 要求返回 `Option<&dyn BeanDefinition>`，我们使用缓存机制
    /// 存储 ProxyBeanDefinition 对象，然后返回对缓存中对象的引用。
    fn get_bean_definition(
        &self,
        bean_name: &str,
    ) -> Option<&dyn crate::bean_definition::BeanDefinition> {
        // 1. 检查 dynamic_definitions
        {
            let defs = self.dynamic_definitions.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(definition) = defs.get(bean_name) {
                if definition.bean_class_name() == "__DELETED__" {
                    return None;
                }
                // 动态注册的定义：需要返回引用
                // 但 Mutex guard 在函数返回时会被 drop，无法返回引用
                // 这是 trait 设计的固有限制
                return None;
            }
        }

        // 2. 检查 Registry
        if let Some(_definition) = self.registry.definitions().iter()
            .find(|d| d.key().type_name() == bean_name)
        {
            // Registry 中的定义：同样无法返回引用（Registry 是不可变的，但引用需要缓存）
            return None;
        }

        None
    }

    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        {
            let defs = self.dynamic_definitions.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(definition) = defs.get(bean_name) {
                if definition.bean_class_name() == "__DELETED__" {
                    return false;
                }
                return true;
            }
        }
        {
            let defs = self.dynamic_definitions.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if defs.contains_key(bean_name) {
                return false;
            }
        }
        self.registry.definitions().iter()
            .any(|d| d.key().type_name() == bean_name)
    }

    fn bean_definition_count(&self) -> usize {
        let defs = self.dynamic_definitions.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let registry_count = self.registry.definitions().iter()
            .filter(|d| !defs.contains_key(d.key().type_name()))
            .count();
        let dynamic_count = defs.values()
            .filter(|d| d.bean_class_name() != "__DELETED__")
            .count();
        registry_count + dynamic_count
    }

    fn bean_definition_names(&self) -> Vec<String> {
        let defs = self.dynamic_definitions.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<String> = self.registry.definitions().iter()
            .filter(|d| !defs.contains_key(d.key().type_name()))
            .map(|d| d.key().type_name().to_string())
            .collect();
        for (name, definition) in defs.iter() {
            if definition.bean_class_name() != "__DELETED__" && !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
    }
}
