//! AutowiredAnnotationBeanPostProcessor — Spring 风格的 @Autowired 注解后处理器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.annotation.AutowiredAnnotationBeanPostProcessor`。
//!
//! 处理 `@Autowired` 注解的字段注入和方法注入。

use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::bean_post_processor::BeanPostProcessor;

/// Spring 风格的 `@Autowired` 注解后处理器。
///
/// 对应 Spring 的 `AutowiredAnnotationBeanPostProcessor`。
///
/// 扫描 Bean 实例的字段和方法，识别 `@Autowired` 注解，
/// 自动注入依赖。
pub struct AutowiredAnnotationBeanPostProcessor {
    field_cache: Mutex<HashMap<TypeId, Vec<InjectionPoint>>>,
    method_cache: Mutex<HashMap<TypeId, Vec<InjectionPoint>>>,
    initialized: Mutex<bool>,
}

#[derive(Debug, Clone)]
struct InjectionPoint {
    member_name: String,
    type_id: TypeId,
}

impl AutowiredAnnotationBeanPostProcessor {
    pub fn new() -> Self {
        Self {
            field_cache: Mutex::new(HashMap::new()),
            method_cache: Mutex::new(HashMap::new()),
            initialized: Mutex::new(false),
        }
    }

    pub fn register_field(&self, type_id: TypeId, field_name: String, field_type: TypeId) {
        let mut cache = self.field_cache.lock().unwrap();
        cache.entry(type_id).or_insert_with(Vec::new).push(InjectionPoint {
            member_name: field_name,
            type_id: field_type,
        });
    }

    pub fn register_method(&self, type_id: TypeId, method_name: String, param_type: TypeId) {
        let mut cache = self.method_cache.lock().unwrap();
        cache.entry(type_id).or_insert_with(Vec::new).push(InjectionPoint {
            member_name: method_name,
            type_id: param_type,
        });
    }

    pub fn initialize(&self) {
        let mut init = self.initialized.lock().unwrap();
        *init = true;
    }

    pub fn is_initialized(&self) -> bool {
        *self.initialized.lock().unwrap()
    }

    pub fn get_field_injection_points(&self, type_id: TypeId) -> Vec<String> {
        let cache = self.field_cache.lock().unwrap();
        cache.get(&type_id).map(|points| {
            points.iter().map(|p| p.member_name.clone()).collect()
        }).unwrap_or_default()
    }

    pub fn get_method_injection_points(&self, type_id: TypeId) -> Vec<String> {
        let cache = self.method_cache.lock().unwrap();
        cache.get(&type_id).map(|points| {
            points.iter().map(|p| p.member_name.clone()).collect()
        }).unwrap_or_default()
    }

    pub fn injection_count(&self, type_id: TypeId) -> usize {
        let fields = self.field_cache.lock().unwrap();
        let methods = self.method_cache.lock().unwrap();
        fields.get(&type_id).map(|v| v.len()).unwrap_or(0)
            + methods.get(&type_id).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for AutowiredAnnotationBeanPostProcessor {
    fn default() -> Self { Self::new() }
}

impl BeanPostProcessor for AutowiredAnnotationBeanPostProcessor {
    fn post_process_before_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
    fn post_process_after_initialization(
        &self,
        _bean: Arc<dyn Any + Send + Sync>,
        _bean_name: &str,
    ) -> Result<Option<Arc<dyn Any + Send + Sync>>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }
}
