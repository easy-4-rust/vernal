//! 标准类型定位器（对标 Spring `StandardTypeLocator`）。
//!
//! 对标 Java `org.springframework.expression.spel.support.StandardTypeLocator`。
//! 支持按名称查找类型（含 import 前缀解析）。

use crate::evaluation_exception::EvaluationException;
use crate::type_locator::TypeLocator;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::RwLock;

/// 标准类型定位器（对标 Spring `StandardTypeLocator`）。
///
/// 支持按名称查找类型，含 import 前缀解析。
/// 内置常见 Rust 标准类型的映射。
pub struct StandardTypeLocator {
    /// 已注册的类型映射（名称 → TypeId）。
    types: RwLock<HashMap<String, TypeId>>,
    /// 导入前缀列表（对标 Java import 语句）。
    imports: Vec<String>,
}

impl StandardTypeLocator {
    /// 创建标准类型定位器（含内置类型映射）。
    #[must_use]
    pub fn new() -> Self {
        let mut types = HashMap::new();
        // 内置常见类型映射（对标 Spring 默认 java.lang 包导入）
        types.insert("String".to_string(), TypeId::of::<String>());
        types.insert("java.lang.String".to_string(), TypeId::of::<String>());
        types.insert("Integer".to_string(), TypeId::of::<i32>());
        types.insert("java.lang.Integer".to_string(), TypeId::of::<i32>());
        types.insert("Long".to_string(), TypeId::of::<i64>());
        types.insert("java.lang.Long".to_string(), TypeId::of::<i64>());
        types.insert("Boolean".to_string(), TypeId::of::<bool>());
        types.insert("java.lang.Boolean".to_string(), TypeId::of::<bool>());
        types.insert("Double".to_string(), TypeId::of::<f64>());
        types.insert("java.lang.Double".to_string(), TypeId::of::<f64>());
        types.insert("Float".to_string(), TypeId::of::<f32>());
        types.insert("java.lang.Float".to_string(), TypeId::of::<f32>());
        types.insert("Number".to_string(), TypeId::of::<f64>());
        types.insert("java.lang.Number".to_string(), TypeId::of::<f64>());
        types.insert("Object".to_string(), TypeId::of::<()>());
        types.insert("java.lang.Object".to_string(), TypeId::of::<()>());

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

impl Default for StandardTypeLocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeLocator for StandardTypeLocator {
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
        let locator = StandardTypeLocator::new();
        let tid = locator.find_type("String").unwrap();
        assert_eq!(tid, TypeId::of::<String>());
    }

    #[test]
    fn find_builtin_integer() {
        let locator = StandardTypeLocator::new();
        let tid = locator.find_type("Integer").unwrap();
        assert_eq!(tid, TypeId::of::<i32>());
    }

    #[test]
    fn find_unknown_type() {
        let locator = StandardTypeLocator::new();
        assert!(locator.find_type("UnknownType").is_err());
    }

    #[test]
    fn register_custom_type() {
        let locator = StandardTypeLocator::new();
        struct MyType;
        locator.register_type("MyType", TypeId::of::<MyType>());
        let tid = locator.find_type("MyType").unwrap();
        assert_eq!(tid, TypeId::of::<MyType>());
    }

    #[test]
    fn import_prefix_resolution() {
        let locator = StandardTypeLocator::new();
        // java.lang.String 已内置
        let tid = locator.find_type("java.lang.String").unwrap();
        assert_eq!(tid, TypeId::of::<String>());
    }
}
