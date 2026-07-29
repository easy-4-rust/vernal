//! StaticListableBeanFactory — Spring 风格的静态可列举 BeanFactory。
//!
//! 对应 Java 类：`org.springframework.beans.factory.support.StaticListableBeanFactory`。
//!
//! 内置一组预注册的 singleton Bean，不解析依赖、不创建实例，仅用于静态配置场景。
//! 它是 `ListableBeanFactory` 的最简实现，常用于测试与工具集成。

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// 静态可列举 BeanFactory。
///
/// 对应 Spring 的 `StaticListableBeanFactory`。
///
/// 维护一个静态的 Bean 名称 → 实例映射，提供 `register_singleton()` 增量注册与
/// `get_bean()` / `contains_bean()` / `bean_names()` 等查询能力。
/// 所有 Bean 默认视为 singleton。类型信息（`TypeId`、类型名）随注册一并记录。
#[derive(Debug, Default)]
pub struct StaticListableBeanFactory {
    /// Bean 名称 → 类型擦除实例。
    beans: HashMap<String, BeanEntry>,
}

/// 单个静态 Bean 的存储条目。
#[derive(Clone)]
struct BeanEntry {
    /// 实例（类型擦除）。
    value: Arc<dyn Any + Send + Sync>,
    /// 实例的 `TypeId`。
    type_id: TypeId,
    /// Rust 类型名。
    type_name: &'static str,
}

impl std::fmt::Debug for BeanEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BeanEntry")
            .field("type_id", &self.type_id)
            .field("type_name", &self.type_name)
            .finish()
    }
}

impl StaticListableBeanFactory {
    /// 创建空的工厂。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个 singleton Bean。
    ///
    /// 若同名 Bean 已存在则覆盖（对应 Spring 的覆盖语义）。
    ///
    /// # 泛型
    ///
    /// - `T` — Bean 的具体类型，必须 `Send + Sync + 'static`
    pub fn register_singleton<T>(&mut self, name: impl Into<String>, bean: T)
    where
        T: Any + Send + Sync + 'static,
    {
        let name = name.into();
        let entry = BeanEntry {
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
            value: Arc::new(bean),
        };
        self.beans.insert(name, entry);
    }

    /// 注册一个已类型擦除的 singleton Bean。
    ///
    /// 需要显式提供 `TypeId` 与类型名；用于无法静态确定类型的场景。
    pub fn register_singleton_dyn(
        &mut self,
        name: impl Into<String>,
        value: Arc<dyn Any + Send + Sync>,
        type_id: TypeId,
        type_name: &'static str,
    ) {
        let name = name.into();
        let entry = BeanEntry {
            value,
            type_id,
            type_name,
        };
        self.beans.insert(name, entry);
    }

    /// 按名称获取 Bean 实例。
    ///
    /// 对应 Spring 的 `Object getBean(String name)`。
    pub fn get_bean(
        &self,
        name: &str,
    ) -> Result<Arc<dyn Any + Send + Sync>, Box<dyn std::error::Error + Send + Sync>> {
        match self.beans.get(name) {
            Some(entry) => Ok(entry.value.clone()),
            None => Err(format!("No bean named '{}' available", name).into()),
        }
    }

    /// 按名称获取强类型 Bean 实例。
    ///
    /// 类型不匹配时返回错误。
    pub fn get_typed_bean<T>(
        &self,
        name: &str,
    ) -> Result<Arc<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: Any + Send + Sync + 'static,
    {
        let raw = self.get_bean(name)?;
        // 类型校验：TypeId 必须一致。
        let expected = TypeId::of::<T>();
        match self.beans.get(name) {
            Some(entry) if entry.type_id == expected => {}
            _ => {
                return Err(format!("Bean '{}' is not of the requested type", name).into());
            }
        }
        Arc::downcast::<T>(raw).map_err(|_| -> Box<dyn std::error::Error + Send + Sync> {
            format!(
                "Bean '{}' could not be downcast to the requested type",
                name
            )
            .into()
        })
    }

    /// 检查是否包含指定名称的 Bean。
    ///
    /// 对应 Spring 的 `boolean containsBean(String name)`。
    pub fn contains_bean(&self, name: &str) -> bool {
        self.beans.contains_key(name)
    }

    /// 获取所有已注册 Bean 的名称。
    ///
    /// 对应 Spring 的 `String[] getBeanDefinitionNames()`。
    pub fn bean_names(&self) -> Vec<String> {
        self.beans.keys().cloned().collect()
    }

    /// 已注册 Bean 数量。
    pub fn bean_count(&self) -> usize {
        self.beans.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.beans.is_empty()
    }

    /// 按类型获取所有匹配的 Bean 名称。
    ///
    /// 对应 Spring 的 `String[] getBeanNamesForType(Class<?> type)`。
    pub fn bean_names_for_type(&self, type_id: TypeId) -> Vec<String> {
        self.beans
            .iter()
            .filter(|(_, entry)| entry.type_id == type_id)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// 获取指定 Bean 的类型名。
    pub fn get_type(&self, name: &str) -> Option<&'static str> {
        self.beans.get(name).map(|entry| entry.type_name)
    }

    /// 获取指定 Bean 的 `TypeId`。
    pub fn get_type_id(&self, name: &str) -> Option<TypeId> {
        self.beans.get(name).map(|entry| entry.type_id)
    }

    /// 移除指定 Bean。
    pub fn remove_bean(&mut self, name: &str) -> bool {
        self.beans.remove(name).is_some()
    }

    /// 清空所有 Bean。
    pub fn clear(&mut self) {
        self.beans.clear();
    }
}
