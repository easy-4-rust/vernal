//! AbstractBeanFactory — Spring 风格的抽象 Bean 工厂基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanFactory`。
//!
//! 在 Spring 中，这是 BeanFactory 体系的核心抽象基类，
//! 提供了 Bean 定义缓存、别名管理、单例缓存、Scope 注册等核心功能。
//! `DefaultListableBeanFactory` 和 `AbstractAutowireCapableBeanFactory`
//! 都继承自此类。
//!
//! ## 三级缓存
//!
//! Spring 的 `AbstractBeanFactory` 配合 `DefaultSingletonBeanRegistry`
//! 实现了三级缓存来解决循环依赖问题。
//!
//! ## Bean 生命周期
//!
//! 1. 实例化（Instantiation）
//! 2. 属性填充（Populate properties）
//! 3. BeanNameAware / BeanClassLoaderAware / BeanFactoryAware
//! 4. BeanPostProcessor.postProcessBeforeInitialization
//! 5. InitializingBean.afterPropertiesSet / init-method
//! 6. BeanPostProcessor.postProcessAfterInitialization
//! 7. 使用
//! 8. DisposableBean.destroy / destroy-method

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ScopeKey;

/// 抽象 Bean 工厂基类。
///
/// 对应 Spring 的 `AbstractBeanFactory`。
///
/// 提供 BeanFactory 的基础实现，包括：
/// - Bean 定义缓存
/// - 别名管理
/// - 单例缓存
/// - Scope 注册
/// - BeanPostProcessor 管理
pub struct AbstractBeanFactory {
    /// Bean 定义缓存（bean_name -> definition）
    bean_definitions: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 单例缓存（bean_name -> instance）
    singletons: Mutex<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// 别名映射（alias -> bean_name）
    aliases: Mutex<HashMap<String, String>>,
    /// 作用域注册（scope_name -> ScopeKey）
    scopes: Mutex<HashMap<String, ScopeKey>>,
    /// 父 BeanFactory（可选）
    parent: Mutex<Option<String>>,
    /// 配置是否已冻结
    configuration_frozen: Mutex<bool>,
    /// Bean 名称缓存（type_id -> bean_names）
    type_to_names: Mutex<HashMap<std::any::TypeId, Vec<String>>>,
    /// Bean 创建中的名称集合（用于检测循环依赖）
    singletons_currently_in_creation: Mutex<std::collections::HashSet<String>>,
    /// 已注册的 BeanPostProcessor 数量
    post_processor_count: Mutex<usize>,
    /// 是否已经销毁
    destroyed: Mutex<bool>,
}

impl AbstractBeanFactory {
    /// 创建新的抽象 Bean 工厂。
    pub fn new() -> Self {
        Self {
            bean_definitions: Mutex::new(HashMap::new()),
            singletons: Mutex::new(HashMap::new()),
            aliases: Mutex::new(HashMap::new()),
            scopes: Mutex::new(HashMap::new()),
            parent: Mutex::new(None),
            configuration_frozen: Mutex::new(false),
            type_to_names: Mutex::new(HashMap::new()),
            singletons_currently_in_creation: Mutex::new(std::collections::HashSet::new()),
            post_processor_count: Mutex::new(0),
            destroyed: Mutex::new(false),
        }
    }

    /// 注册 Bean 定义。
    pub fn register_bean_definition(&self, name: String, definition: Arc<dyn Any + Send + Sync>) {
        self.bean_definitions.lock().unwrap().insert(name, definition);
    }

    /// 移除 Bean 定义。
    pub fn remove_bean_definition(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.bean_definitions.lock().unwrap().remove(name)
    }

    /// 获取 Bean 定义。
    pub fn get_bean_definition(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.bean_definitions.lock().unwrap().get(name).cloned()
    }

    /// 获取 Bean 定义数量。
    pub fn bean_definition_count(&self) -> usize {
        self.bean_definitions.lock().unwrap().len()
    }

    /// 检查 Bean 定义是否存在。
    pub fn contains_bean_definition(&self, name: &str) -> bool {
        self.bean_definitions.lock().unwrap().contains_key(name)
    }

    /// 获取所有 Bean 定义名称。
    pub fn bean_definition_names(&self) -> Vec<String> {
        self.bean_definitions.lock().unwrap().keys().cloned().collect()
    }

    /// 注册单例。
    pub fn register_singleton(&self, name: String, singleton: Arc<dyn Any + Send + Sync>) {
        self.singletons.lock().unwrap().insert(name, singleton);
    }

    /// 获取单例。
    pub fn get_singleton(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.singletons.lock().unwrap().get(name).map(Arc::clone)
    }

    /// 检查单例是否存在。
    pub fn contains_singleton(&self, name: &str) -> bool {
        self.singletons.lock().unwrap().contains_key(name)
    }

    /// 获取单例数量。
    pub fn singleton_count(&self) -> usize {
        self.singletons.lock().unwrap().len()
    }

    /// 获取所有单例名称。
    pub fn singleton_names(&self) -> Vec<String> {
        self.singletons.lock().unwrap().keys().cloned().collect()
    }

    /// 注册别名。
    ///
    /// 对应 Spring 的 `registerAlias(String, String)`。
    pub fn register_alias(&self, alias: String, bean_name: String) -> Result<(), String> {
        let mut aliases = self.aliases.lock().unwrap();
        if let Some(existing) = aliases.get(&alias) {
            if existing == &bean_name {
                return Ok(());
            }
            return Err(format!("Alias '{}' already points to '{}'", alias, existing));
        }
        aliases.insert(alias, bean_name);
        Ok(())
    }

    /// 解析别名。
    pub fn resolve_alias(&self, alias: &str) -> String {
        self.aliases.lock().unwrap().get(alias).cloned().unwrap_or_else(|| alias.to_string())
    }

    /// 获取别名数量。
    pub fn alias_count(&self) -> usize {
        self.aliases.lock().unwrap().len()
    }

    /// 检查别名是否存在。
    pub fn contains_alias(&self, alias: &str) -> bool {
        self.aliases.lock().unwrap().contains_key(alias)
    }

    /// 注册 Scope。
    pub fn register_scope(&self, name: String, scope: ScopeKey) {
        self.scopes.lock().unwrap().insert(name, scope);
    }

    /// 获取已注册的 Scope 数量。
    pub fn registered_scope_count(&self) -> usize {
        self.scopes.lock().unwrap().len()
    }

    /// 检查 Scope 是否已注册。
    pub fn contains_scope(&self, name: &str) -> bool {
        self.scopes.lock().unwrap().contains_key(name)
    }

    /// 设置父 BeanFactory。
    pub fn set_parent(&self, parent: Option<String>) {
        *self.parent.lock().unwrap() = parent;
    }

    /// 获取父 BeanFactory 名称。
    pub fn parent(&self) -> Option<String> {
        self.parent.lock().unwrap().clone()
    }

    /// 冻结配置。
    pub fn freeze_configuration(&self) {
        *self.configuration_frozen.lock().unwrap() = true;
    }

    /// 解冻配置。
    pub fn defreeze_configuration(&self) {
        *self.configuration_frozen.lock().unwrap() = false;
    }

    /// 检查配置是否已冻结。
    pub fn is_configuration_frozen(&self) -> bool {
        *self.configuration_frozen.lock().unwrap()
    }

    /// 注册类型到名称的映射。
    pub fn register_type_mapping(&self, type_id: std::any::TypeId, bean_name: String) {
        self.type_to_names.lock().unwrap()
            .entry(type_id)
            .or_default()
            .push(bean_name);
    }

    /// 按类型查找 Bean 名称。
    pub fn get_bean_names_for_type(&self, type_id: std::any::TypeId) -> Vec<String> {
        self.type_to_names.lock().unwrap()
            .get(&type_id)
            .cloned()
            .unwrap_or_default()
    }

    /// 标记 Bean 正在创建中。
    ///
    /// 对应 Spring 的 `beforeSingletonCreation`。
    pub fn mark_singleton_in_creation(&self, bean_name: &str) -> bool {
        let mut set = self.singletons_currently_in_creation.lock().unwrap();
        set.insert(bean_name.to_string())
    }

    /// 标记 Bean 创建完成。
    ///
    /// 对应 Spring 的 `afterSingletonCreation`。
    pub fn unmark_singleton_in_creation(&self, bean_name: &str) {
        let mut set = self.singletons_currently_in_creation.lock().unwrap();
        set.remove(bean_name);
    }

    /// 检查 Bean 是否正在创建中。
    ///
    /// 对应 Spring 的 `isSingletonCurrentlyInCreation`。
    pub fn is_singleton_currently_in_creation(&self, bean_name: &str) -> bool {
        self.singletons_currently_in_creation.lock().unwrap().contains(bean_name)
    }

    /// 获取正在创建中的 Bean 数量。
    pub fn singletons_in_creation_count(&self) -> usize {
        self.singletons_currently_in_creation.lock().unwrap().len()
    }

    /// 注册 BeanPostProcessor 数量。
    pub fn set_post_processor_count(&self, count: usize) {
        *self.post_processor_count.lock().unwrap() = count;
    }

    /// 获取 BeanPostProcessor 数量。
    pub fn post_processor_count(&self) -> usize {
        *self.post_processor_count.lock().unwrap()
    }

    /// 检查 Bean 是否存在（定义或单例）。
    ///
    /// 对应 Spring 的 `containsBean(String)`。
    pub fn contains_bean(&self, name: &str) -> bool {
        let resolved = self.resolve_alias(name);
        self.contains_bean_definition(&resolved) || self.contains_singleton(&resolved)
    }

    /// 判断 Bean 是否为单例。
    pub fn is_singleton(&self, name: &str) -> bool {
        let resolved = self.resolve_alias(name);
        self.contains_singleton(&resolved)
    }

    /// 判断 Bean 是否为原型。
    pub fn is_prototype(&self, name: &str) -> bool {
        let resolved = self.resolve_alias(name);
        self.contains_bean_definition(&resolved) && !self.contains_singleton(&resolved)
    }

    /// 销毁所有单例。
    pub fn destroy_singletons(&self) {
        self.singletons.lock().unwrap().clear();
        *self.destroyed.lock().unwrap() = true;
    }

    /// 检查是否已销毁。
    pub fn is_destroyed(&self) -> bool {
        *self.destroyed.lock().unwrap()
    }

    /// 获取所有别名。
    pub fn aliases(&self) -> HashMap<String, String> {
        self.aliases.lock().unwrap().clone()
    }
}

impl Default for AbstractBeanFactory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_factory_is_empty() {
        let factory = AbstractBeanFactory::new();
        assert_eq!(factory.bean_definition_count(), 0);
        assert_eq!(factory.singleton_count(), 0);
        assert_eq!(factory.alias_count(), 0);
    }

    #[test]
    fn register_and_find_bean_definition() {
        let factory = AbstractBeanFactory::new();
        factory.register_bean_definition("myBean".to_string(), Arc::new(42_i32));

        assert!(factory.contains_bean_definition("myBean"));
        assert_eq!(factory.bean_definition_count(), 1);
    }

    #[test]
    fn register_and_find_singleton() {
        let factory = AbstractBeanFactory::new();
        factory.register_singleton("service".to_string(), Arc::new("impl".to_string()));

        assert!(factory.contains_singleton("service"));
        let bean = factory.get_singleton("service").unwrap();
        assert_eq!(bean.downcast_ref::<String>(), Some(&"impl".to_string()));
    }

    #[test]
    fn alias_management() {
        let factory = AbstractBeanFactory::new();
        factory.register_alias("alias1".to_string(), "myBean".to_string()).unwrap();

        assert_eq!(factory.resolve_alias("alias1"), "myBean");
        assert_eq!(factory.resolve_alias("unknown"), "unknown");
        assert_eq!(factory.alias_count(), 1);
    }

    #[test]
    fn duplicate_alias_same_target_ok() {
        let factory = AbstractBeanFactory::new();
        factory.register_alias("a".to_string(), "bean".to_string()).unwrap();
        assert!(factory.register_alias("a".to_string(), "bean".to_string()).is_ok());
    }

    #[test]
    fn duplicate_alias_different_target_errors() {
        let factory = AbstractBeanFactory::new();
        factory.register_alias("a".to_string(), "bean1".to_string()).unwrap();
        assert!(factory.register_alias("a".to_string(), "bean2".to_string()).is_err());
    }

    #[test]
    fn configuration_freeze() {
        let factory = AbstractBeanFactory::new();
        assert!(!factory.is_configuration_frozen());

        factory.freeze_configuration();
        assert!(factory.is_configuration_frozen());
    }

    #[test]
    fn bean_definition_names() {
        let factory = AbstractBeanFactory::new();
        factory.register_bean_definition("a".to_string(), Arc::new(1));
        factory.register_bean_definition("b".to_string(), Arc::new(2));

        let mut names = factory.bean_definition_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn destroy_singletons() {
        let factory = AbstractBeanFactory::new();
        factory.register_singleton("a".to_string(), Arc::new(1));
        factory.register_singleton("b".to_string(), Arc::new(2));
        assert_eq!(factory.singleton_count(), 2);

        factory.destroy_singletons();
        assert_eq!(factory.singleton_count(), 0);
        assert!(factory.is_destroyed());
    }

    #[test]
    fn remove_bean_definition() {
        let factory = AbstractBeanFactory::new();
        factory.register_bean_definition("bean".to_string(), Arc::new(1));
        assert!(factory.contains_bean_definition("bean"));

        let removed = factory.remove_bean_definition("bean");
        assert!(removed.is_some());
        assert!(!factory.contains_bean_definition("bean"));
    }

    #[test]
    fn singleton_in_creation_tracking() {
        let factory = AbstractBeanFactory::new();
        assert!(!factory.is_singleton_currently_in_creation("bean"));

        factory.mark_singleton_in_creation("bean");
        assert!(factory.is_singleton_currently_in_creation("bean"));
        assert_eq!(factory.singletons_in_creation_count(), 1);

        factory.unmark_singleton_in_creation("bean");
        assert!(!factory.is_singleton_currently_in_creation("bean"));
    }

    #[test]
    fn contains_bean_checks_alias() {
        let factory = AbstractBeanFactory::new();
        factory.register_bean_definition("myBean".to_string(), Arc::new(1));
        factory.register_alias("alias1".to_string(), "myBean".to_string()).unwrap();

        assert!(factory.contains_bean("myBean"));
        assert!(factory.contains_bean("alias1"));
        assert!(!factory.contains_bean("unknown"));
    }

    #[test]
    fn is_singleton_and_prototype() {
        let factory = AbstractBeanFactory::new();
        factory.register_bean_definition("proto".to_string(), Arc::new(1));
        factory.register_singleton("single".to_string(), Arc::new(2));

        assert!(factory.is_singleton("single"));
        assert!(!factory.is_singleton("proto"));
        assert!(factory.is_prototype("proto"));
        assert!(!factory.is_prototype("single"));
    }

    #[test]
    fn singleton_names() {
        let factory = AbstractBeanFactory::new();
        factory.register_singleton("a".to_string(), Arc::new(1));
        factory.register_singleton("b".to_string(), Arc::new(2));

        let mut names = factory.singleton_names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn get_bean_definition() {
        let factory = AbstractBeanFactory::new();
        factory.register_bean_definition("bean".to_string(), Arc::new(42_i32));

        let def = factory.get_bean_definition("bean");
        assert!(def.is_some());
        assert_eq!(def.unwrap().downcast_ref::<i32>(), Some(&42));

        assert!(factory.get_bean_definition("missing").is_none());
    }

    #[test]
    fn contains_alias() {
        let factory = AbstractBeanFactory::new();
        factory.register_alias("a".to_string(), "bean".to_string()).unwrap();

        assert!(factory.contains_alias("a"));
        assert!(!factory.contains_alias("b"));
    }

    #[test]
    fn defreeze_configuration() {
        let factory = AbstractBeanFactory::new();
        factory.freeze_configuration();
        assert!(factory.is_configuration_frozen());

        factory.defreeze_configuration();
        assert!(!factory.is_configuration_frozen());
    }

    #[test]
    fn post_processor_count() {
        let factory = AbstractBeanFactory::new();
        assert_eq!(factory.post_processor_count(), 0);

        factory.set_post_processor_count(5);
        assert_eq!(factory.post_processor_count(), 5);
    }

    #[test]
    fn type_mapping() {
        let factory = AbstractBeanFactory::new();
        factory.register_type_mapping(std::any::TypeId::of::<String>(), "bean1".to_string());
        factory.register_type_mapping(std::any::TypeId::of::<String>(), "bean2".to_string());

        let names = factory.get_bean_names_for_type(std::any::TypeId::of::<String>());
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn get_bean_names_for_unknown_type() {
        let factory = AbstractBeanFactory::new();
        let names = factory.get_bean_names_for_type(std::any::TypeId::of::<Vec<i32>>());
        assert!(names.is_empty());
    }

    #[test]
    fn set_and_get_parent() {
        let factory = AbstractBeanFactory::new();
        assert_eq!(factory.parent(), None);

        factory.set_parent(Some("parentFactory".to_string()));
        assert_eq!(factory.parent(), Some("parentFactory".to_string()));

        factory.set_parent(None);
        assert_eq!(factory.parent(), None);
    }

    #[test]
    fn get_singleton_missing() {
        let factory = AbstractBeanFactory::new();
        assert!(factory.get_singleton("nonexistent").is_none());
    }

    #[test]
    fn remove_bean_definition_missing() {
        let factory = AbstractBeanFactory::new();
        assert!(factory.remove_bean_definition("nonexistent").is_none());
    }

    #[test]
    fn register_scope_and_check() {
        let factory = AbstractBeanFactory::new();
        assert_eq!(factory.registered_scope_count(), 0);
        assert!(!factory.contains_scope("request"));

        factory.register_scope("request".to_string(), ScopeKey::of::<String>());
        assert_eq!(factory.registered_scope_count(), 1);
        assert!(factory.contains_scope("request"));
        assert!(!factory.contains_scope("session"));
    }

    #[test]
    fn mark_and_unmark_singleton_in_creation_idempotent() {
        let factory = AbstractBeanFactory::new();
        // First mark returns true (newly inserted)
        assert!(factory.mark_singleton_in_creation("bean"));
        // Second mark returns false (already exists)
        assert!(!factory.mark_singleton_in_creation("bean"));
        assert_eq!(factory.singletons_in_creation_count(), 1);

        factory.unmark_singleton_in_creation("bean");
        assert_eq!(factory.singletons_in_creation_count(), 0);
    }

    #[test]
    fn destroy_singletons_clears_all() {
        let factory = AbstractBeanFactory::new();
        factory.register_singleton("a".to_string(), Arc::new(1));
        factory.register_singleton("b".to_string(), Arc::new(2));
        factory.register_singleton("c".to_string(), Arc::new(3));
        assert_eq!(factory.singleton_count(), 3);
        assert!(!factory.is_destroyed());

        factory.destroy_singletons();
        assert_eq!(factory.singleton_count(), 0);
        assert!(factory.is_destroyed());
    }

    #[test]
    fn aliases_returns_clone() {
        let factory = AbstractBeanFactory::new();
        factory.register_alias("a".to_string(), "bean".to_string()).unwrap();
        factory.register_alias("b".to_string(), "bean".to_string()).unwrap();

        let aliases = factory.aliases();
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases.get("a"), Some(&"bean".to_string()));
        assert_eq!(aliases.get("b"), Some(&"bean".to_string()));
    }

    #[test]
    fn is_singleton_with_alias() {
        let factory = AbstractBeanFactory::new();
        factory.register_singleton("myBean".to_string(), Arc::new(1));
        factory.register_alias("alias1".to_string(), "myBean".to_string()).unwrap();

        assert!(factory.is_singleton("alias1"));
    }

    #[test]
    fn is_prototype_with_alias() {
        let factory = AbstractBeanFactory::new();
        factory.register_bean_definition("myBean".to_string(), Arc::new(1));
        factory.register_alias("alias1".to_string(), "myBean".to_string()).unwrap();

        assert!(factory.is_prototype("alias1"));
    }

    #[test]
    fn multiple_type_mappings() {
        let factory = AbstractBeanFactory::new();
        factory.register_type_mapping(std::any::TypeId::of::<String>(), "s1".to_string());
        factory.register_type_mapping(std::any::TypeId::of::<i32>(), "i1".to_string());
        factory.register_type_mapping(std::any::TypeId::of::<String>(), "s2".to_string());

        let string_names = factory.get_bean_names_for_type(std::any::TypeId::of::<String>());
        assert_eq!(string_names.len(), 2);
        let i32_names = factory.get_bean_names_for_type(std::any::TypeId::of::<i32>());
        assert_eq!(i32_names.len(), 1);
    }
}
