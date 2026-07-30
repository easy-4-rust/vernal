//! AbstractBeanFactory — Spring 风格的抽象 Bean 工厂基类。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.AbstractBeanFactory`。
//!
//! 提供 BeanFactory 的基础实现，包括 Bean 定义缓存、别名管理、
//! 单例缓存、Scope 注册等核心功能。

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
}

impl AbstractBeanFactory {
    pub fn new() -> Self {
        Self {
            bean_definitions: Mutex::new(HashMap::new()),
            singletons: Mutex::new(HashMap::new()),
            aliases: Mutex::new(HashMap::new()),
            scopes: Mutex::new(HashMap::new()),
            parent: Mutex::new(None),
            configuration_frozen: Mutex::new(false),
        }
    }

    /// 注册 Bean 定义
    pub fn register_bean_definition(&self, name: String, definition: Arc<dyn Any + Send + Sync>) {
        self.bean_definitions.lock().unwrap().insert(name, definition);
    }

    /// 获取 Bean 定义数量
    pub fn bean_definition_count(&self) -> usize {
        self.bean_definitions.lock().unwrap().len()
    }

    /// 检查 Bean 定义是否存在
    pub fn contains_bean_definition(&self, name: &str) -> bool {
        self.bean_definitions.lock().unwrap().contains_key(name)
    }

    /// 获取所有 Bean 定义名称
    pub fn bean_definition_names(&self) -> Vec<String> {
        self.bean_definitions.lock().unwrap().keys().cloned().collect()
    }

    /// 注册单例
    pub fn register_singleton(&self, name: String, singleton: Arc<dyn Any + Send + Sync>) {
        self.singletons.lock().unwrap().insert(name, singleton);
    }

    /// 获取单例
    pub fn get_singleton(&self, name: &str) -> Option<Arc<dyn Any + Send + Sync>> {
        self.singletons.lock().unwrap().get(name).map(Arc::clone)
    }

    /// 检查单例是否存在
    pub fn contains_singleton(&self, name: &str) -> bool {
        self.singletons.lock().unwrap().contains_key(name)
    }

    /// 获取单例数量
    pub fn singleton_count(&self) -> usize {
        self.singletons.lock().unwrap().len()
    }

    /// 注册别名
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

    /// 解析别名
    pub fn resolve_alias(&self, alias: &str) -> String {
        self.aliases.lock().unwrap().get(alias).cloned().unwrap_or_else(|| alias.to_string())
    }

    /// 获取别名数量
    pub fn alias_count(&self) -> usize {
        self.aliases.lock().unwrap().len()
    }

    /// 注册 Scope
    pub fn register_scope(&self, name: String, scope: ScopeKey) {
        self.scopes.lock().unwrap().insert(name, scope);
    }

    /// 获取已注册的 Scope 数量
    pub fn registered_scope_count(&self) -> usize {
        self.scopes.lock().unwrap().len()
    }

    /// 检查 Scope 是否已注册
    pub fn contains_scope(&self, name: &str) -> bool {
        self.scopes.lock().unwrap().contains_key(name)
    }

    /// 设置父 BeanFactory
    pub fn set_parent(&self, parent: Option<String>) {
        *self.parent.lock().unwrap() = parent;
    }

    /// 获取父 BeanFactory 名称
    pub fn parent(&self) -> Option<String> {
        self.parent.lock().unwrap().clone()
    }

    /// 冻结配置
    pub fn freeze_configuration(&self) {
        *self.configuration_frozen.lock().unwrap() = true;
    }

    /// 检查配置是否已冻结
    pub fn is_configuration_frozen(&self) -> bool {
        *self.configuration_frozen.lock().unwrap()
    }

    /// 销毁所有单例
    pub fn destroy_singletons(&self) {
        self.singletons.lock().unwrap().clear();
    }
}

impl Default for AbstractBeanFactory {
    fn default() -> Self { Self::new() }
}
