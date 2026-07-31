//! EmptyReaderEventListener — 对应 Spring EmptyReaderEventListener。
//!
//! 空的读取器事件监听器。所有事件回调均为 no-op。当不需要监听
//! 解析事件时作为默认实现使用。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.EmptyReaderEventListener`。

use super::alias_definition::AliasDefinition;
use super::component_definition::ComponentDefinitionRef;
use super::import_definition::ImportDefinition;
use super::reader_event_listener::ReaderEventListener;

/// 空的读取器事件监听器。
///
/// 对应 Spring 的 `EmptyReaderEventListener`。
///
/// 所有事件回调均为 no-op，适合作为不需要监听解析事件时的
/// 默认实现。
///
/// ## 示例
///
/// ```rust
/// use vernal_beans::factory::parsing::empty_reader_event_listener::EmptyReaderEventListener;
/// use vernal_beans::factory::parsing::reader_event_listener::ReaderEventListener;
/// use vernal_beans::factory::parsing::component_definition::ComponentDefinitionRef;
///
/// let listener = EmptyReaderEventListener::new();
/// let comp = ComponentDefinitionRef::new("test", None);
/// // 不会 panic，也不会有任何副作用
/// listener.component_registered(&comp);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyReaderEventListener;

impl EmptyReaderEventListener {
    /// 创建一个新的空事件监听器。
    pub fn new() -> Self {
        Self
    }
}

impl ReaderEventListener for EmptyReaderEventListener {
    fn component_registered(&self, _component: &ComponentDefinitionRef) {
        // no-op
    }

    fn alias_registered(&self, _alias: &AliasDefinition) {
        // no-op
    }

    fn import_registered(&self, _import: &ImportDefinition) {
        // no-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_op_component_registered() {
        let listener = EmptyReaderEventListener::new();
        let comp = ComponentDefinitionRef::new("bean1", Some("MyClass".to_string()));
        // 不应 panic
        listener.component_registered(&comp);
    }

    #[test]
    fn test_no_op_alias_registered() {
        let listener = EmptyReaderEventListener::new();
        let alias = AliasDefinition::new("bean1", "alias1");
        listener.alias_registered(&alias);
    }

    #[test]
    fn test_no_op_import_registered() {
        let listener = EmptyReaderEventListener::new();
        let import = ImportDefinition::new("config.xml");
        listener.import_registered(&import);
    }
}
