//! ResourceEditorRegistrar — Spring 风格的资源编辑器注册器。
//!
//! 对应 Java 类：`org.springframework.beans.support.ResourceEditorRegistrar`。
//!
//! 在 Spring 中，`ResourceEditorRegistrar` 实现了 `PropertyEditorRegistrar` 接口，
//! 负责注册与资源相关的属性编辑器（Resource, InputStream, File 等）。
//! 它通常在应用上下文刷新时被调用。
//!
//! ## 设计说明
//!
//! 在 vernal 中，`ResourceEditorRegistrar` 维护一个已注册的编辑器类型列表。

use std::collections::HashSet;
use std::sync::Mutex;

/// 资源编辑器注册器。
///
/// 对应 Spring 的 `ResourceEditorRegistrar`。
///
/// 注册与资源相关的属性编辑器。
#[derive(Debug, Default)]
pub struct ResourceEditorRegistrar {
    /// 已注册的编辑器类型名。
    registered_editors: Mutex<HashSet<String>>,
}

impl ResourceEditorRegistrar {
    /// 创建资源编辑器注册器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册所有默认的资源编辑器。
    ///
    /// 对应 Spring 的 `registerCustomEditors(PropertyEditorRegistry)`。
    pub fn register_custom_editors(&self) -> usize {
        let editors = [
            "FileEditor",
            "InputStreamEditor",
            "ReaderEditor",
            "URLEditor",
            "URLEditor",
            "PathEditor",
        ];

        let mut registered = self.registered_editors.lock().unwrap();
        for editor in &editors {
            registered.insert(editor.to_string());
        }
        registered.len()
    }

    /// 注册单个编辑器类型。
    pub fn register_editor(&self, editor_type: impl Into<String>) {
        self.registered_editors
            .lock()
            .unwrap()
            .insert(editor_type.into());
    }

    /// 检查编辑器是否已注册。
    pub fn is_registered(&self, editor_type: &str) -> bool {
        self.registered_editors
            .lock()
            .unwrap()
            .contains(editor_type)
    }

    /// 获取已注册的编辑器数量。
    pub fn registered_count(&self) -> usize {
        self.registered_editors.lock().unwrap().len()
    }

    /// 获取所有已注册的编辑器类型。
    pub fn registered_editors(&self) -> Vec<String> {
        self.registered_editors
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_default_editors() {
        let registrar = ResourceEditorRegistrar::new();
        let count = registrar.register_custom_editors();
        assert!(count > 0);
        assert!(registrar.is_registered("FileEditor"));
        assert!(registrar.is_registered("URLEditor"));
    }

    #[test]
    fn register_custom_editor() {
        let registrar = ResourceEditorRegistrar::new();
        registrar.register_editor("CustomEditor");
        assert!(registrar.is_registered("CustomEditor"));
        assert!(!registrar.is_registered("UnknownEditor"));
    }

    #[test]
    fn registered_count() {
        let registrar = ResourceEditorRegistrar::new();
        assert_eq!(registrar.registered_count(), 0);
        registrar.register_custom_editors();
        assert!(registrar.registered_count() > 0);
    }

    #[test]
    fn registered_editors_list() {
        let registrar = ResourceEditorRegistrar::new();
        registrar.register_editor("Editor1");
        registrar.register_editor("Editor2");
        let editors = registrar.registered_editors();
        assert!(editors.contains(&"Editor1".to_string()));
        assert!(editors.contains(&"Editor2".to_string()));
    }
}
