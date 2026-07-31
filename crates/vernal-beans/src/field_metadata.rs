//! FieldMetadata — 编译期字段元数据收集。
//!
//! 使用 inventory 或 linkme 在编译期收集结构体字段的元数据，
//! 用于 autowireBean 的字段注入逻辑。
//!
//! ## 设计原理
//!
//! Spring 通过反射获取 Bean 的字段信息，然后按类型注入。
//! Rust 没有运行时反射，因此使用：
//! 1. 编译期宏生成字段元数据（通过 `inventory` 收集）
//! 2. 运行时按 TypeId 匹配并注入
//!
//! ## 使用方式
//!
//! ```rust,ignore
//! use vernal_beans::prelude::*;
//!
//! #[derive(ComponentMetadata)]
//! struct MyService {
//!     #[inject]
//!     database: Arc<DatabasePool>,
//!     #[inject(optional)]
//!     cache: Option<Arc<CacheService>>,
//! }
//! ```

use std::any::TypeId;
use std::sync::Mutex;

/// 字段注入描述符。
///
/// 描述一个需要注入的字段：
/// - 字段名称
/// - 字段类型（TypeId）
/// - 是否可选
/// - 限定符名称
#[derive(Clone, Debug)]
pub struct FieldDescriptor {
    /// 字段名称。
    pub name: &'static str,
    /// 字段类型（TypeId）。
    pub type_id: TypeId,
    /// 类型名称。
    pub type_name: &'static str,
    /// 是否可选（Option<T>）。
    pub optional: bool,
    /// 限定符名称。
    pub qualifier: Option<&'static str>,
}

impl FieldDescriptor {
    /// 创建新的字段描述符。
    pub fn new(name: &'static str, type_id: TypeId, type_name: &'static str) -> Self {
        Self {
            name,
            type_id,
            type_name,
            optional: false,
            qualifier: None,
        }
    }

    /// 设置为可选。
    pub fn with_optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// 设置限定符。
    pub fn with_qualifier(mut self, qualifier: &'static str) -> Self {
        self.qualifier = Some(qualifier);
        self
    }
}

/// 类型元数据：包含类型名称和需要注入的字段列表。
#[derive(Clone, Debug)]
pub struct TypeMetadata {
    /// 类型 ID。
    pub type_id: TypeId,
    /// 类型名称。
    pub type_name: &'static str,
    /// 需要注入的字段。
    pub fields: Vec<FieldDescriptor>,
}

/// 全局类型元数据存储（线程安全）。
///
/// 使用 Mutex<Vec> 实现动态注册，支持运行时添加类型元数据。
static TYPE_METADATA: Mutex<Vec<TypeMetadata>> = Mutex::new(Vec::new());

/// 注册类型元数据（运行时调用）。
///
/// 对应 Spring 的 `@Autowired` 注解处理：编译期收集字段信息，运行时注册。
pub fn register_type_metadata(metadata: TypeMetadata) {
    TYPE_METADATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(metadata);
}

/// 获取指定类型的元数据。
pub fn get_metadata_for_type(type_id: TypeId) -> Option<TypeMetadata> {
    TYPE_METADATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .find(|m| m.type_id == type_id)
        .cloned()
}

/// 获取所有已注册的类型元数据。
pub fn get_all_metadata() -> Vec<TypeMetadata> {
    TYPE_METADATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// 按类型 ID 查找所有需要注入该类型的字段。
///
/// 返回 `(SourceTypeId, FieldDescriptor)` 列表，表示哪些类型的哪些字段需要注入。
pub fn find_fields_needing_type(target_type_id: TypeId) -> Vec<(TypeId, &'static str)> {
    let mut result = Vec::new();
    let metadata = TYPE_METADATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for m in metadata.iter() {
        for field in &m.fields {
            if field.type_id == target_type_id {
                result.push((m.type_id, field.name));
            }
        }
    }
    result
}

/// 检查指定类型是否需要注入。
pub fn type_needs_injection(type_id: TypeId) -> bool {
    TYPE_METADATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .any(|m| m.type_id == type_id && !m.fields.is_empty())
}

/// 清空所有已注册的类型元数据（用于测试）。
pub fn clear_metadata() {
    TYPE_METADATA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}
