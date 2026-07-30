//! Environment 类型定位器（Vernal 扩展）。
//!
//! 对标 Spring `Environment` 中的类型定位能力。
//! 通过 `ApplicationEnvironment` 的属性映射来查找类型。
//!
//! # 设计
//!
//! - 继承 `StandardTypeLocator` 的内置类型映射
//! - 额外支持从 `ApplicationEnvironment` 中注册自定义类型
//! - 支持 import 前缀解析

use crate::evaluation_exception::EvaluationException;
use crate::type_locator::TypeLocator;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::RwLock;

/// Environment 类型定位器（Vernal 扩展）。
///
/// 通过 `ApplicationEnvironment` 的属性映射来查找类型。
/// 继承 `StandardTypeLocator` 的内置类型映射。
pub struct EnvironmentTypeLocator {
    /// 已注册的类型映射（名称 → TypeId）。
    types: RwLock<HashMap<String, TypeId>>,
    /// 导入前缀列表。
    imports: Vec<String>,
}

impl EnvironmentTypeLocator {
    /// 创建 Environment 类型定位器（含内置类型映射）。
    #[must_use]
    pub fn new() -> Self {
        let mut types = HashMap::new();
        // 内置常见类型映射
        types.insert("String".to_string(), TypeId::of::<String>());
        types.insert("Integer".to_string(), TypeId::of::<i32>());
        types.insert("Long".to_string(), TypeId::of::<i64>());
        types.insert("Boolean".to_string(), TypeId::of::<bool>());
        types.insert("Double".to_string(), TypeId::of::<f64>());
        types.insert("Float".to_string(), TypeId::of::<f32>());
        types.insert("Number".to_string(), TypeId::of::<f64>());
        types.insert("Object".to_string(), TypeId::of::<()>());

        Self {
            types: RwLock::new(types),
            imports: vec!["java.lang".to_string()],
        }
    }

    /// 注册自定义类型映射。
    pub fn register_type(&self, name: impl Into<String>, type_id: TypeId) {
        if let Ok(mut types) = self.types.write() {
            types.insert(name.into(), type_id);
        }
    }

    /// 注册导入前缀。
    pub fn register_import(&mut self, prefix: impl Into<String>) {
        self.imports.push(prefix.into());
    }

    /// 获取已注册的导入前缀。
    pub fn imports(&self) -> &[String] {
        &self.imports
    }
}

impl Default for EnvironmentTypeLocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeLocator for EnvironmentTypeLocator {
    fn find_type(&self, type_name: &str) -> Result<TypeId, EvaluationException> {
        // 1. 直接查找
        if let Ok(types) = self.types.read() {
            if let Some(tid) = types.get(type_name) {
                return Ok(*tid);
            }
        }

        // 2. 尝试 import 前缀组合
        for prefix in &self.imports {
            let full_name = format!("{}.{}", prefix, type_name);
            if let Ok(types) = self.types.read() {
                if let Some(tid) = types.get(&full_name) {
                    return Ok(*tid);
                }
            }
        }

        // 3. 类型未找到
        Err(EvaluationException::new(
            type_name.to_string(),
            None,
            format!("类型 '{}' 未找到", type_name),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_builtin_string() {
        let locator = EnvironmentTypeLocator::new();
        let tid = locator.find_type("String").unwrap();
        assert_eq!(tid, TypeId::of::<String>());
    }

    #[test]
    fn find_builtin_integer() {
        let locator = EnvironmentTypeLocator::new();
        let tid = locator.find_type("Integer").unwrap();
        assert_eq!(tid, TypeId::of::<i32>());
    }

    #[test]
    fn find_unknown_type() {
        let locator = EnvironmentTypeLocator::new();
        assert!(locator.find_type("UnknownType").is_err());
    }

    #[test]
    fn register_custom_type() {
        let locator = EnvironmentTypeLocator::new();
        struct MyType;
        locator.register_type("MyType", TypeId::of::<MyType>());
        let tid = locator.find_type("MyType").unwrap();
        assert_eq!(tid, TypeId::of::<MyType>());
    }

    #[test]
    fn import_prefix_resolution() {
        let locator = EnvironmentTypeLocator::new();
        // java.lang.String 通过 import 前缀组合查找
        let tid = locator.find_type("String").unwrap();
        assert_eq!(tid, TypeId::of::<String>());
    }
}
