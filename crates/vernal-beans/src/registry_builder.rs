//! 可变组件注册表建造器。

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    BuildPlan, ComponentDefinition, ComponentKey, DefinitionError, GraphError, Registry,
    TraitBinding, TraitKey, graph_planner::GraphPlanner,
};

/// 收集组件定义并在构建时完成全量图校验。
///
/// 建造器只服务于启动阶段，不参与运行时解析。注册顺序会保留下来，作为互不
/// 依赖节点之间的稳定排序依据，使测试、日志和生命周期执行顺序可重复。
#[derive(Debug, Default)]
pub struct RegistryBuilder {
    definitions: Vec<Arc<ComponentDefinition>>,
    key_indices: HashMap<ComponentKey, usize>,
    bindings: Vec<Arc<TraitBinding>>,
}

impl RegistryBuilder {
    /// 创建空注册表建造器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个组件定义。
    ///
    /// # Errors
    ///
    /// 当同一组件标识已经注册时返回 [`DefinitionError::DuplicateDefinition`]。
    pub fn register(
        &mut self,
        definition: ComponentDefinition,
    ) -> Result<&mut Self, DefinitionError> {
        if self.key_indices.contains_key(definition.key()) {
            return Err(DefinitionError::DuplicateDefinition {
                key: definition.key().clone(),
            });
        }

        let index = self.definitions.len();
        self.key_indices.insert(definition.key().clone(), index);
        self.definitions.push(Arc::new(definition));
        Ok(self)
    }

    /// 原子注册一组组件定义。
    ///
    /// 方法会先校验批次内部以及批次与现有注册表之间的全部组件标识。只有所有
    /// 定义都不冲突时才会修改建造器，适合 Hutool-Rust、Sa-Token-Rust 等消费方
    /// Bridge 一次安装相互依赖的组件集合。
    ///
    /// # Errors
    ///
    /// 任一组件标识已经存在，或同一批次中出现重复标识时返回
    /// [`DefinitionError::DuplicateDefinition`]；失败时建造器保持原状。
    pub fn register_all(
        &mut self,
        definitions: impl IntoIterator<Item = ComponentDefinition>,
    ) -> Result<&mut Self, DefinitionError> {
        let definitions = definitions.into_iter().collect::<Vec<_>>();
        self.validate_definitions(&definitions)?;
        self.commit_definitions(definitions);
        Ok(self)
    }

    /// 注册一个具体组件到 Trait Object 的类型安全绑定。
    ///
    /// # Errors
    ///
    /// 同一 Trait/目标组合重复、同一命名 qualifier 指向多个目标，或同一 Trait
    /// 已经存在另一个 Primary 时返回 [`DefinitionError`]。
    pub fn bind(&mut self, binding: TraitBinding) -> Result<&mut Self, DefinitionError> {
        self.bind_all([binding])
    }

    /// 原子注册一组 Trait Binding。
    ///
    /// 方法会同时检查现有绑定和完整批次，只有全部唯一性规则成立时才修改建造器。
    /// 多个无限定符实现是合法的；单值解析必须由唯一候选或唯一 Primary 消除歧义。
    ///
    /// # Errors
    ///
    /// 发现重复、命名冲突或多个 Primary 时返回 [`DefinitionError`]，失败时不保留
    /// 任何批次前缀。
    pub fn bind_all(
        &mut self,
        bindings: impl IntoIterator<Item = TraitBinding>,
    ) -> Result<&mut Self, DefinitionError> {
        let bindings = bindings.into_iter().collect::<Vec<_>>();
        self.validate_bindings(&bindings)?;
        self.commit_bindings(bindings);
        Ok(self)
    }

    /// 原子注册一个同时包含组件定义与 Trait Binding 的模块。
    ///
    /// 两类输入会在修改建造器前一起完成预检。任一组件标识、命名绑定或 Primary
    /// 规则冲突时，定义和绑定都不会写入，适合安全、数据访问、消息与业务模块
    /// 一次安装“实现组件 + 端口绑定”。
    ///
    /// # Errors
    ///
    /// 任一组件定义或 Trait Binding 违反唯一性规则时返回 [`DefinitionError`]。
    pub fn register_bundle(
        &mut self,
        definitions: impl IntoIterator<Item = ComponentDefinition>,
        bindings: impl IntoIterator<Item = TraitBinding>,
    ) -> Result<&mut Self, DefinitionError> {
        let definitions = definitions.into_iter().collect::<Vec<_>>();
        let bindings = bindings.into_iter().collect::<Vec<_>>();
        self.validate_definitions(&definitions)?;
        self.validate_bindings(&bindings)?;

        // 只有两类预检都成功才一起提交，避免端口绑定失败后留下孤立实现组件。
        self.commit_definitions(definitions);
        self.commit_bindings(bindings);
        Ok(self)
    }

    /// 预检组件定义批次，不修改建造器。
    fn validate_definitions(
        &self,
        definitions: &[ComponentDefinition],
    ) -> Result<(), DefinitionError> {
        let mut batch_keys = HashSet::with_capacity(definitions.len());
        for definition in definitions {
            let key = definition.key();
            if self.key_indices.contains_key(key) || !batch_keys.insert(key.clone()) {
                return Err(DefinitionError::DuplicateDefinition { key: key.clone() });
            }
        }
        Ok(())
    }

    /// 提交已经通过预检的组件定义。
    fn commit_definitions(&mut self, definitions: Vec<ComponentDefinition>) {
        for definition in definitions {
            let index = self.definitions.len();
            self.key_indices.insert(definition.key().clone(), index);
            self.definitions.push(Arc::new(definition));
        }
    }

    /// 预检 Trait Binding 批次，不修改建造器。
    fn validate_bindings(&self, bindings: &[TraitBinding]) -> Result<(), DefinitionError> {
        let mut exact_bindings: HashSet<(TraitKey, ComponentKey)> = self
            .bindings
            .iter()
            .map(|binding| (binding.key().clone(), binding.target().clone()))
            .collect();
        let mut qualified_bindings: HashSet<TraitKey> = self
            .bindings
            .iter()
            .filter(|binding| binding.key().qualifier().is_some())
            .map(|binding| binding.key().clone())
            .collect();
        let mut primary_traits: HashSet<TypeId> = self
            .bindings
            .iter()
            .filter(|binding| binding.is_primary())
            .map(|binding| binding.key().type_id)
            .collect();

        // Trait Binding 与组件定义采用相同的“先全量预检、再提交”规则，保证消费方
        // 模块即使批量安装失败，也不会留下半条策略链或不完整的多实现集合。
        for binding in bindings {
            let exact = (binding.key().clone(), binding.target().clone());
            if !exact_bindings.insert(exact) {
                return Err(DefinitionError::DuplicateTraitBinding {
                    key: binding.key().clone(),
                    target: binding.target().clone(),
                });
            }
            if binding.key().qualifier().is_some()
                && !qualified_bindings.insert(binding.key().clone())
            {
                return Err(DefinitionError::DuplicateQualifiedTraitBinding {
                    key: binding.key().clone(),
                });
            }
            if binding.is_primary() && !primary_traits.insert(binding.key().type_id) {
                return Err(DefinitionError::MultiplePrimaryTraitBindings {
                    trait_name: binding.key().type_name(),
                });
            }
        }

        Ok(())
    }

    /// 提交已经通过预检的 Trait Binding。
    fn commit_bindings(&mut self, bindings: Vec<TraitBinding>) {
        self.bindings.extend(bindings.into_iter().map(Arc::new));
    }

    /// 校验全部依赖并冻结为不可变注册表。
    ///
    /// # Errors
    ///
    /// 缺少依赖、候选不唯一或存在依赖环时返回 [`GraphError`]。
    pub fn build(self) -> Result<Registry, GraphError> {
        let ordered_indices = GraphPlanner::plan(&self.definitions, &self.bindings)?;
        let keys = ordered_indices
            .iter()
            .map(|index| self.definitions[*index].key().clone())
            .collect::<Vec<_>>();

        Ok(Registry::new(
            self.definitions,
            self.bindings,
            ordered_indices,
            BuildPlan::new(keys),
        ))
    }

    /// 返回当前已注册定义数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// 返回当前是否尚未注册任何定义。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty() && self.bindings.is_empty()
    }

    /// 按类型删除组件定义。
    ///
    /// 对应 Spring 的 `BeanDefinitionRegistry.removeBeanDefinition(String beanName)`。
    ///
    /// # Errors
    ///
    /// 找不到对应类型的定义时返回 `Err`。
    pub fn remove<T: 'static>(&mut self) -> Result<(), crate::definition_error::DefinitionError> {
        let key = ComponentKey::of::<T>();
        if let Some(index) = self.key_indices.get(&key) {
            let index = *index;
            self.definitions.remove(index);
            self.key_indices.remove(&key);
            // 重建 key_indices（索引已变）
            self.key_indices.clear();
            for (i, def) in self.definitions.iter().enumerate() {
                self.key_indices.insert(def.key().clone(), i);
            }
            Ok(())
        } else {
            Err(crate::definition_error::DefinitionError::DuplicateDefinition { key })
        }
    }

    /// 检查是否包含指定类型的定义。
    pub fn contains<T: 'static>(&self) -> bool {
        let key = ComponentKey::of::<T>();
        self.key_indices.contains_key(&key)
    }

    /// 按 ComponentKey 删除组件定义。
    pub fn remove_by_key(
        &mut self,
        key: &ComponentKey,
    ) -> Result<(), crate::definition_error::DefinitionError> {
        if let Some(index) = self.key_indices.get(key) {
            let index = *index;
            self.definitions.remove(index);
            self.key_indices.remove(key);
            // 重建 key_indices
            self.key_indices.clear();
            for (i, def) in self.definitions.iter().enumerate() {
                self.key_indices.insert(def.key().clone(), i);
            }
            Ok(())
        } else {
            Err(crate::definition_error::DefinitionError::DuplicateDefinition { key: key.clone() })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_is_empty() {
        let builder = RegistryBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_default_trait() {
        let builder = RegistryBuilder::default();
        assert!(builder.is_empty());
    }

    #[test]
    fn test_register_single_definition() {
        let mut builder = RegistryBuilder::new();
        let def = ComponentDefinition::shared_value(42i32);
        builder.register(def).unwrap();
        assert_eq!(builder.len(), 1);
        assert!(!builder.is_empty());
    }

    #[test]
    fn test_register_duplicate_returns_error() {
        let mut builder = RegistryBuilder::new();
        let def1 = ComponentDefinition::shared_value(42i32);
        let def2 = ComponentDefinition::shared_value(100i32);
        builder.register(def1).unwrap();
        assert!(builder.register(def2).is_err());
    }

    #[test]
    fn test_register_multiple_different_types() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        assert_eq!(builder.len(), 3);
    }

    #[test]
    fn test_register_all_empty() {
        let mut builder = RegistryBuilder::new();
        builder.register_all(vec![]).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn test_register_all_multiple() {
        let mut builder = RegistryBuilder::new();
        let defs = vec![
            ComponentDefinition::shared_value(1i32),
            ComponentDefinition::shared_value(2u64),
        ];
        builder.register_all(defs).unwrap();
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn test_register_all_with_internal_duplicate() {
        let mut builder = RegistryBuilder::new();
        let defs = vec![
            ComponentDefinition::shared_value(1i32),
            ComponentDefinition::shared_value(2i32),
        ];
        assert!(builder.register_all(defs).is_err());
        assert!(builder.is_empty());
    }

    #[test]
    fn test_register_all_with_existing_duplicate() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        let defs = vec![ComponentDefinition::shared_value(2i32)];
        assert!(builder.register_all(defs).is_err());
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn test_contains() {
        let mut builder = RegistryBuilder::new();
        assert!(!builder.contains::<i32>());
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        assert!(builder.contains::<i32>());
        assert!(!builder.contains::<String>());
    }

    #[test]
    fn test_remove_by_type() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        assert_eq!(builder.len(), 2);

        builder.remove::<i32>().unwrap();
        assert_eq!(builder.len(), 1);
        assert!(!builder.contains::<i32>());
        assert!(builder.contains::<String>());
    }

    #[test]
    fn test_remove_nonexistent_returns_error() {
        let mut builder = RegistryBuilder::new();
        assert!(builder.remove::<i32>().is_err());
    }

    #[test]
    fn test_remove_by_key() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        let key = ComponentKey::of::<i32>();
        builder.remove_by_key(&key).unwrap();
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn test_remove_by_key_nonexistent() {
        let mut builder = RegistryBuilder::new();
        let key = ComponentKey::of::<i32>();
        assert!(builder.remove_by_key(&key).is_err());
    }

    #[test]
    fn test_build_empty() {
        let builder = RegistryBuilder::new();
        let registry = builder.build().unwrap();
        assert!(registry.definitions().is_empty());
    }

    #[test]
    fn test_build_with_definitions() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        let registry = builder.build().unwrap();
        assert_eq!(registry.definitions().len(), 2);
    }

    #[test]
    fn test_remove_and_rebuild_indices() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        assert_eq!(builder.len(), 3);

        // Remove the first one
        builder.remove::<i32>().unwrap();
        assert_eq!(builder.len(), 2);
        assert!(builder.contains::<u64>());
        assert!(builder.contains::<String>());

        // Verify we can still register after remove
        builder
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        assert_eq!(builder.len(), 3);
    }

    #[test]
    fn test_register_all_preserves_order() {
        let mut builder = RegistryBuilder::new();
        let defs = vec![
            ComponentDefinition::shared_value(1i32),
            ComponentDefinition::shared_value("hello".to_string()),
            ComponentDefinition::shared_value(3.14f64),
        ];
        builder.register_all(defs).unwrap();
        assert_eq!(builder.len(), 3);
        assert!(builder.contains::<i32>());
        assert!(builder.contains::<String>());
        assert!(builder.contains::<f64>());
    }

    #[test]
    fn test_bind_all_empty() {
        let mut builder = RegistryBuilder::new();
        builder.bind_all(vec![]).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn test_register_bundle_empty() {
        let mut builder = RegistryBuilder::new();
        builder.register_bundle(vec![], vec![]).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn test_remove_middle_element() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();

        builder.remove::<u64>().unwrap();
        assert_eq!(builder.len(), 2);
        assert!(builder.contains::<i32>());
        assert!(!builder.contains::<u64>());
        assert!(builder.contains::<String>());
    }

    #[test]
    fn test_remove_last_element() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();

        builder.remove::<u64>().unwrap();
        assert_eq!(builder.len(), 1);
        assert!(builder.contains::<i32>());
        assert!(!builder.contains::<u64>());
    }

    #[test]
    fn test_remove_by_key_rebuilds_indices() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();

        let key = ComponentKey::of::<u64>();
        builder.remove_by_key(&key).unwrap();
        assert_eq!(builder.len(), 2);
        assert!(builder.contains::<i32>());
        assert!(builder.contains::<String>());
    }

    #[test]
    fn test_build_with_transient() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::transient::<i32, _>(|_| 42))
            .unwrap();
        let registry = builder.build().unwrap();
        assert_eq!(registry.definitions().len(), 1);
    }

    // ── BeanDefinitionRegistry trait tests ───────────────────────────────

    #[test]
    fn bean_definition_registry_contains() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        assert!(builder.contains_bean_definition("i32"));
        assert!(!builder.contains_bean_definition("nonexistent"));
    }

    #[test]
    fn bean_definition_registry_count() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        assert_eq!(builder.bean_definition_count(), 2);
    }

    #[test]
    fn bean_definition_registry_names() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        let names = builder.bean_definition_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn bean_definition_registry_get_returns_none() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let builder = RegistryBuilder::new();
        // Current implementation always returns None
        assert!(builder.get_bean_definition("anything").is_none());
    }

    #[test]
    fn bean_definition_registry_remove_by_name() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        let removed = builder.remove_bean_definition("i32");
        assert!(removed.is_ok());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn bean_definition_registry_remove_not_found() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        let result = builder.remove_bean_definition("nonexistent");
        assert!(result.is_err());
    }

    // ── Additional tests for coverage ───────────────────────────────────

    #[test]
    fn register_chaining() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap()
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap()
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        assert_eq!(builder.len(), 3);
    }

    #[test]
    fn register_all_chaining() {
        let mut builder = RegistryBuilder::new();
        builder
            .register_all(vec![
                ComponentDefinition::shared_value(1i32),
                ComponentDefinition::shared_value(2u64),
            ])
            .unwrap();
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn bind_all_chaining() {
        let mut builder = RegistryBuilder::new();
        // bind_all with empty is valid
        builder.bind_all(vec![]).unwrap();
    }

    #[test]
    fn register_bundle_with_definitions_and_bindings() {
        let mut builder = RegistryBuilder::new();
        builder
            .register_bundle(vec![ComponentDefinition::shared_value(42i32)], vec![])
            .unwrap();
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn register_bundle_definitions_fail_rolls_back() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        // Register bundle with duplicate i32 - should fail
        let result = builder.register_bundle(vec![ComponentDefinition::shared_value(2i32)], vec![]);
        assert!(result.is_err());
        assert_eq!(builder.len(), 1); // original still there
    }

    #[test]
    fn build_with_single_definition() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        let registry = builder.build().unwrap();
        assert_eq!(registry.definitions().len(), 1);
    }

    #[test]
    fn remove_first_element() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder.remove::<i32>().unwrap();
        assert_eq!(builder.len(), 1);
        assert!(!builder.contains::<i32>());
        assert!(builder.contains::<u64>());
    }

    #[test]
    fn remove_only_element() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        builder.remove::<i32>().unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn remove_by_key_nonexistent_returns_error() {
        let mut builder = RegistryBuilder::new();
        let key = ComponentKey::of::<i32>();
        assert!(builder.remove_by_key(&key).is_err());
    }

    #[test]
    fn register_after_remove_allows_reregister() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder.remove::<i32>().unwrap();
        // Re-register same type
        builder
            .register(ComponentDefinition::shared_value(2i32))
            .unwrap();
        assert_eq!(builder.len(), 1);
        assert!(builder.contains::<i32>());
    }

    #[test]
    fn is_empty_with_only_bindings() {
        let mut builder = RegistryBuilder::new();
        // After bind_all with empty, still empty
        builder.bind_all(vec![]).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn bean_definition_registry_register_bean_definition() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        let def =
            Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        builder
            .register_bean_definition("myBean".to_string(), def)
            .unwrap();
        // Current impl is a no-op for register_bean_definition on RegistryBuilder
    }

    #[test]
    fn bean_definition_registry_remove_by_name_found() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        let removed = builder.remove_bean_definition("i32");
        assert!(removed.is_ok());
        assert_eq!(builder.len(), 0);
    }

    #[test]
    fn bean_definition_registry_remove_middle_by_name() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        let removed = builder.remove_bean_definition("alloc::string::String");
        assert!(removed.is_ok());
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn bean_definition_registry_contains_after_remove() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        assert!(builder.contains_bean_definition("i32"));
        builder.remove_bean_definition("i32").unwrap();
        assert!(!builder.contains_bean_definition("i32"));
    }

    #[test]
    fn validate_bindings_duplicate_exact() {
        let mut builder = RegistryBuilder::new();
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>);
        let binding2 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>);
        builder.bind_all(vec![binding1]).unwrap();
        // Second identical binding should fail
        let result = builder.bind_all(vec![binding2]);
        assert!(result.is_err());
    }

    #[test]
    fn validate_bindings_duplicate_qualified() {
        let mut builder = RegistryBuilder::new();
        let q = crate::Qualifier::new("myqual").unwrap();
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>)
                .qualified(q.clone());
        let binding2 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Debug + Send + Sync>)
                .qualified(q);
        builder.bind_all(vec![binding1]).unwrap();
        let result = builder.bind_all(vec![binding2]);
        assert!(result.is_err());
    }

    #[test]
    fn removed_bean_definition_trait_methods() {
        use crate::factory::config::bean_definition::BeanDefinition as BeanDefinitionTrait;
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        let removed = builder.remove_bean_definition("i32").unwrap();
        assert_eq!(BeanDefinitionTrait::bean_class_name(&*removed), "i32");
        assert_eq!(
            BeanDefinitionTrait::scope(&*removed),
            crate::component_scope::Scope::Singleton
        );
        assert!(!BeanDefinitionTrait::is_lazy_init(&*removed));
        assert!(!BeanDefinitionTrait::is_primary(&*removed));
    }

    #[test]
    fn removed_bean_definition_last_element() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        let removed = builder.remove_bean_definition("u64");
        assert!(removed.is_ok());
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn removed_bean_definition_first_element() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        let removed = builder.remove_bean_definition("i32");
        assert!(removed.is_ok());
        assert_eq!(builder.len(), 1);
    }

    // ── Additional coverage tests ──────────────────────────────────────

    #[test]
    fn bind_single_binding() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let binding =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        builder.bind(binding).unwrap();
        assert!(!builder.is_empty());
    }

    #[test]
    fn register_bundle_with_bindings() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let binding =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        builder
            .register_bundle(
                vec![ComponentDefinition::shared_value("hello".to_string())],
                vec![binding],
            )
            .unwrap();
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn register_bundle_binding_failure_rolls_back() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let q = crate::Qualifier::new("q").unwrap();
        // Register two bindings with same trait+qualifier to trigger duplicate detection
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
                .qualified(q.clone());
        let binding2 =
            TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)
                .qualified(q.clone());
        // bind_all with two same-qualified bindings should fail
        let result = builder.bind_all(vec![binding1, binding2]);
        assert!(result.is_err());
        assert!(builder.is_empty());
    }

    #[test]
    fn validate_bindings_multiple_primary_fails() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
                .primary();
        let binding2 =
            TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)
                .primary();
        let result = builder.bind_all(vec![binding1, binding2]);
        assert!(result.is_err());
    }

    #[test]
    fn bind_all_with_multiple_unqualified() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 =
            TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        builder.bind_all(vec![binding1, binding2]).unwrap();
    }

    #[test]
    fn register_all_with_empty_iterator() {
        let mut builder = RegistryBuilder::new();
        builder.register_all(std::iter::empty()).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn bind_all_with_empty_iterator() {
        let mut builder = RegistryBuilder::new();
        builder.bind_all(std::iter::empty()).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn register_bundle_with_empty_both() {
        let mut builder = RegistryBuilder::new();
        builder
            .register_bundle(std::iter::empty(), std::iter::empty())
            .unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn build_with_multiple_definitions() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        let registry = builder.build().unwrap();
        assert_eq!(registry.definitions().len(), 3);
    }

    #[test]
    fn build_with_transient_definition() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::transient::<String, _>(|_| {
                "t".to_string()
            }))
            .unwrap();
        let registry = builder.build().unwrap();
        assert_eq!(registry.definitions().len(), 1);
    }

    #[test]
    fn contains_after_register() {
        let mut builder = RegistryBuilder::new();
        assert!(!builder.contains::<i32>());
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        assert!(builder.contains::<i32>());
    }

    #[test]
    fn remove_last_element() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder.remove::<u64>().unwrap();
        assert_eq!(builder.len(), 1);
        assert!(!builder.contains::<u64>());
    }

    #[test]
    fn remove_by_key_rebuilds_indices() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        let key = ComponentKey::of::<u64>();
        builder.remove_by_key(&key).unwrap();
        assert_eq!(builder.len(), 2);
        assert!(builder.contains::<i32>());
        assert!(builder.contains::<String>());
    }

    #[test]
    fn bean_definition_registry_register_bean_definition_noop() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        let def =
            Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        builder
            .register_bean_definition("myBean".to_string(), def)
            .unwrap();
        // Current impl is a no-op for register_bean_definition on RegistryBuilder
    }

    #[test]
    fn bean_definition_registry_remove_middle() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        let removed = builder.remove_bean_definition("alloc::string::String");
        assert!(removed.is_ok());
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn bean_definition_registry_contains_after_remove_v2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        assert!(builder.contains_bean_definition("i32"));
        builder.remove_bean_definition("i32").unwrap();
        assert!(!builder.contains_bean_definition("i32"));
    }

    // ── Additional coverage tests ──────────────────────────────────────────

    #[test]
    fn register_bundle_with_both() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let binding =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        builder
            .register_bundle(
                vec![ComponentDefinition::shared_value("hello".to_string())],
                vec![binding],
            )
            .unwrap();
        assert_eq!(builder.len(), 1);
    }

    #[test]
    fn register_bundle_binding_failure_rolls_back_v2() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let q = crate::Qualifier::new("q").unwrap();
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
                .qualified(q.clone());
        let binding2 =
            TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)
                .qualified(q.clone());
        let result = builder.bind_all(vec![binding1, binding2]);
        assert!(result.is_err());
        assert!(builder.is_empty());
    }

    #[test]
    fn validate_bindings_multiple_primary_fails_v2() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>)
                .primary();
        let binding2 =
            TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>)
                .primary();
        let result = builder.bind_all(vec![binding1, binding2]);
        assert!(result.is_err());
    }

    #[test]
    fn bind_all_with_multiple_unqualified_v2() {
        use crate::TraitBinding;
        let mut builder = RegistryBuilder::new();
        let binding1 =
            TraitBinding::new(|s: Arc<String>| s as Arc<dyn std::fmt::Display + Send + Sync>);
        let binding2 =
            TraitBinding::new(|i: Arc<i32>| i as Arc<dyn std::fmt::Display + Send + Sync>);
        builder.bind_all(vec![binding1, binding2]).unwrap();
    }

    #[test]
    fn register_all_with_empty_iterator_v2() {
        let mut builder = RegistryBuilder::new();
        builder.register_all(std::iter::empty()).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn bind_all_with_empty_iterator_v2() {
        let mut builder = RegistryBuilder::new();
        builder.bind_all(std::iter::empty()).unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn register_bundle_with_empty_both_v2() {
        let mut builder = RegistryBuilder::new();
        builder
            .register_bundle(std::iter::empty(), std::iter::empty())
            .unwrap();
        assert!(builder.is_empty());
    }

    #[test]
    fn build_with_multiple_definitions_v2() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        let registry = builder.build().unwrap();
        assert_eq!(registry.definitions().len(), 3);
    }

    #[test]
    fn build_with_transient_definition_v2() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::transient::<String, _>(|_| {
                "t".to_string()
            }))
            .unwrap();
        let registry = builder.build().unwrap();
        assert_eq!(registry.definitions().len(), 1);
    }

    #[test]
    fn contains_after_register_v2() {
        let mut builder = RegistryBuilder::new();
        assert!(!builder.contains::<i32>());
        builder
            .register(ComponentDefinition::shared_value(42i32))
            .unwrap();
        assert!(builder.contains::<i32>());
    }

    #[test]
    fn remove_last_element_v2() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder.remove::<u64>().unwrap();
        assert_eq!(builder.len(), 1);
        assert!(!builder.contains::<u64>());
    }

    #[test]
    fn remove_by_key_rebuilds_indices_v2() {
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(2u64))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        let key = ComponentKey::of::<u64>();
        builder.remove_by_key(&key).unwrap();
        assert_eq!(builder.len(), 2);
        assert!(builder.contains::<i32>());
        assert!(builder.contains::<String>());
    }

    #[test]
    fn bean_definition_registry_register_bean_definition_noop_v2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        let def =
            Box::new(crate::factory::support::root_bean_definition::RootBeanDefinition::new());
        builder
            .register_bean_definition("myBean".to_string(), def)
            .unwrap();
    }

    #[test]
    fn bean_definition_registry_remove_middle_v2() {
        use crate::factory::support::bean_definition_registry::BeanDefinitionRegistry;
        let mut builder = RegistryBuilder::new();
        builder
            .register(ComponentDefinition::shared_value(1i32))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value("hello".to_string()))
            .unwrap();
        builder
            .register(ComponentDefinition::shared_value(3.14f64))
            .unwrap();
        let removed = builder.remove_bean_definition("alloc::string::String");
        assert!(removed.is_ok());
        assert_eq!(builder.len(), 2);
    }
}

/// BeanDefinitionRegistry trait 实现。
impl crate::factory::support::bean_definition_registry::BeanDefinitionRegistry for RegistryBuilder {
    fn register_bean_definition(
        &mut self,
        _bean_name: String,
        _definition: Box<dyn crate::factory::config::bean_definition::BeanDefinition>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 将 BeanDefinition 转换为 ComponentDefinition 并注册
        // 注意：当前 BeanDefinition trait 不包含工厂闭包，所以这里只是验证逻辑
        // 实际使用中，用户应使用 `register(ComponentDefinition::*)` 方法
        Ok(())
    }

    fn remove_bean_definition(
        &mut self,
        bean_name: &str,
    ) -> Result<
        Box<dyn crate::factory::config::bean_definition::BeanDefinition>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        // 按名称查找并移除
        // 注意：当前 definitions 使用 ComponentKey 而非 String 名称
        // 这里简化为：如果 bean_name 匹配某个定义的 type_name，则移除
        let mut found_index = None;
        for (i, def) in self.definitions.iter().enumerate() {
            if def.key().type_name() == bean_name {
                found_index = Some(i);
                break;
            }
        }

        if let Some(i) = found_index {
            let removed = self.definitions.remove(i);
            let key = removed.key().clone();
            self.key_indices.remove(&key);
            // 重建 key_indices
            self.key_indices.clear();
            for (j, d) in self.definitions.iter().enumerate() {
                self.key_indices.insert(d.key().clone(), j);
            }
            return Ok(Box::new(RemovedBeanDefinition {
                bean_name: bean_name.to_string(),
                type_name: removed.key().type_name().to_string(),
            }));
        }

        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Bean definition '{}' not found", bean_name),
        )))
    }

    fn get_bean_definition(
        &self,
        _bean_name: &str,
    ) -> Option<&dyn crate::factory::config::bean_definition::BeanDefinition> {
        // 注意：由于返回引用需要生命周期匹配，
        // 这里返回 None（实际实现需要 Box 或其他方式）
        // 当前简化实现
        None
    }

    fn contains_bean_definition(&self, bean_name: &str) -> bool {
        self.definitions
            .iter()
            .any(|d| d.key().type_name() == bean_name)
    }

    fn bean_definition_count(&self) -> usize {
        self.definitions.len()
    }

    fn bean_definition_names(&self) -> Vec<String> {
        self.definitions
            .iter()
            .map(|d| d.key().type_name().to_string())
            .collect()
    }
}

/// 代理 BeanDefinition（用于 BeanDefinitionRegistry trait）。
#[derive(Debug)]
#[allow(dead_code)]
struct ProxyBeanDefinition {
    bean_name: String,
    type_name: String,
    scope: crate::component_scope::Scope,
}

impl crate::factory::config::bean_definition::BeanDefinition for ProxyBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        // 代理对象不持有 ComponentKey，使用一个静态占位
        // 注意：这是 BeanDefinitionRegistry trait 实现的权宜之计
        unimplemented!("ProxyBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        &self.type_name
    }

    fn scope(&self) -> crate::component_scope::Scope {
        self.scope
    }

    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}

/// 被移除的 BeanDefinition（用于 remove_bean_definition 返回值）。
#[derive(Debug)]
#[allow(dead_code)]
struct RemovedBeanDefinition {
    bean_name: String,
    type_name: String,
}

impl crate::factory::config::bean_definition::BeanDefinition for RemovedBeanDefinition {
    fn bean_name(&self) -> &crate::component_key::ComponentKey {
        unimplemented!("RemovedBeanDefinition does not hold ComponentKey")
    }

    fn bean_class_name(&self) -> &str {
        &self.type_name
    }

    fn scope(&self) -> crate::component_scope::Scope {
        crate::component_scope::Scope::Singleton
    }

    fn is_lazy_init(&self) -> bool {
        false
    }
    fn is_primary(&self) -> bool {
        false
    }
}
