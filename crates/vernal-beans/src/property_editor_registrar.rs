//! PropertyEditorRegistrar — 对应 Spring `org.springframework.beans.PropertyEditorRegistrar`。
//!
//! 属性编辑器注册器接口。

use crate::property_editor_registry::PropertyEditorRegistry;

/// 属性编辑器注册器接口。
///
/// 对应 Java 接口：`org.springframework.beans.PropertyEditorRegistrar`。
///
/// 用于注册自定义属性编辑器的策略接口。
pub trait PropertyEditorRegistrar: Send + Sync {
    /// 注册自定义属性编辑器。
    ///
    /// 对应 Java 方法：`void registerCustomEditors(PropertyEditorRegistry registry)`
    fn register_custom_editors(&self, registry: &mut dyn PropertyEditorRegistry);
}

/// PropertyEditorRegistrar 的闭包实现。
pub struct ClosurePropertyEditorRegistrar {
    callback: Box<dyn Fn(&mut dyn PropertyEditorRegistry) + Send + Sync>,
}

impl ClosurePropertyEditorRegistrar {
    /// 创建一个新的 ClosurePropertyEditorRegistrar。
    pub fn new(callback: impl Fn(&mut dyn PropertyEditorRegistry) + Send + Sync + 'static) -> Self {
        Self {
            callback: Box::new(callback),
        }
    }
}

impl PropertyEditorRegistrar for ClosurePropertyEditorRegistrar {
    fn register_custom_editors(&self, registry: &mut dyn PropertyEditorRegistry) {
        (self.callback)(registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_editor_registry::SimplePropertyEditorRegistry;

    #[test]
    fn test_closure_registrar() {
        let registrar = ClosurePropertyEditorRegistrar::new(|_registry| {
            // 注册编辑器的逻辑
        });
        let mut registry = SimplePropertyEditorRegistry::new();
        registrar.register_custom_editors(&mut registry);
    }
}
