//! CustomEditorConfigurer — Spring 风格的自定义属性编辑器配置器。
//!
//! 对应 Java 类：`org.springframework.beans.factory.config.CustomEditorConfigurer`。
//!
//! 用于向容器注册自定义的 `PropertyEditor`（按目标 `TypeId` 索引），
//! 通常作为 `BeanFactoryPostProcessor` 的辅助配置 Bean。

use std::any::TypeId;
use std::collections::HashMap;

use crate::property_editor::PropertyEditor;

/// 自定义属性编辑器配置器。
///
/// 对应 Spring 的 `CustomEditorConfigurer`。
///
/// 维护一张 `TypeId → PropertyEditor` 映射，供容器在属性绑定时查询。
/// 由于 `PropertyEditor` 是 trait 对象，本类型不可 `Clone`/`Debug` 自动派生，
/// 因此手动实现了 `Debug`。
pub struct CustomEditorConfigurer {
    /// 自定义编辑器映射：目标类型 id → 编辑器实例。
    custom_editors: HashMap<TypeId, Box<dyn PropertyEditor>>,
}

impl CustomEditorConfigurer {
    /// 创建空的配置器。
    pub fn new() -> Self {
        Self {
            custom_editors: HashMap::new(),
        }
    }

    /// 注册自定义编辑器。
    ///
    /// 对应 Spring 的
    /// `CustomEditorConfigurer.setCustomEditors(Map<Class<?>, Class<? extends PropertyEditor>>)`。
    ///
    /// 以 `TypeId` 作为键。`target_type` 通常与编辑器的 `target_type()` 一致，
    /// 但允许注册时显式覆盖。
    pub fn register_custom_editor(&mut self, target_type: TypeId, editor: Box<dyn PropertyEditor>) {
        self.custom_editors.insert(target_type, editor);
    }

    /// 获取指定类型的自定义编辑器引用。
    pub fn get_custom_editor(&self, target_type: TypeId) -> Option<&dyn PropertyEditor> {
        self.custom_editors.get(&target_type).map(|e| e.as_ref())
    }

    /// 是否注册了指定类型的自定义编辑器。
    pub fn has_custom_editor(&self, target_type: TypeId) -> bool {
        self.custom_editors.contains_key(&target_type)
    }

    /// 已注册编辑器数量。
    pub fn len(&self) -> usize {
        self.custom_editors.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.custom_editors.is_empty()
    }

    /// 获取所有已注册编辑器的目标类型 id。
    pub fn registered_types(&self) -> Vec<TypeId> {
        self.custom_editors.keys().copied().collect()
    }

    /// 移除指定类型的编辑器。
    pub fn remove_custom_editor(&mut self, target_type: TypeId) -> bool {
        self.custom_editors.remove(&target_type).is_some()
    }

    /// 清空所有编辑器。
    pub fn clear(&mut self) {
        self.custom_editors.clear();
    }
}

impl Default for CustomEditorConfigurer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CustomEditorConfigurer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomEditorConfigurer")
            .field("editor_count", &self.custom_editors.len())
            .field(
                "registered_types",
                &self.custom_editors.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}
