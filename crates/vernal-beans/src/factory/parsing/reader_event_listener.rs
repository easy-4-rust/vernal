//! ReaderEventListener — 对应 Spring beans.factory.parsing.ReaderEventListener。
//!
//! Bean 定义读取器事件监听器接口。在 XML/Properties 等配置文件的
//! 读取过程中，当遇到组件定义、别名、导入等元素时触发回调。
//! 用于日志记录、统计和扩展钩子。
//!
//! 对应 Java 接口：`org.springframework.beans.factory.parsing.ReaderEventListener`。

use super::alias_definition::AliasDefinition;
use super::component_definition::ComponentDefinitionRef;
use super::import_definition::ImportDefinition;

/// Bean 定义读取器事件监听器。
///
/// 对应 Spring 的 `ReaderEventListener`。
///
/// 在 Bean 定义读取过程中触发事件回调。框架组件或用户扩展可以
/// 实现此接口来监听解析过程中的关键事件。
///
/// ## 事件类型
///
/// - `component_registered` — 组件定义被注册
/// - `alias_registered` — 别名被注册
/// - `import_registered` — 导入被处理
///
/// ## 示例
///
/// ```rust,ignore
/// use vernal_beans::factory::parsing::reader_event_listener::ReaderEventListener;
///
/// struct LoggingEventListener;
///
/// impl ReaderEventListener for LoggingEventListener {
///     fn component_registered(&self, component: &ComponentDefinitionRef) {
///         // 日志记录组件注册
///     }
/// }
/// ```
pub trait ReaderEventListener: Send + Sync {
    /// 当组件定义被注册时触发。
    ///
    /// 对应 Spring 的 `componentRegistered(ComponentDefinition)`。
    fn component_registered(&self, _component: &ComponentDefinitionRef) {}

    /// 当别名被注册时触发。
    ///
    /// 对应 Spring 的 `aliasRegistered(AliasDefinition)`。
    fn alias_registered(&self, _alias: &AliasDefinition) {}

    /// 当导入被处理时触发。
    ///
    /// 对应 Spring 的 `importRegistered(ImportDefinition)`。
    fn import_registered(&self, _import: &ImportDefinition) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct TestListener {
        events: Mutex<Vec<String>>,
    }

    impl TestListener {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }

        fn event_count(&self) -> usize {
            self.events.lock().unwrap().len()
        }
    }

    impl ReaderEventListener for TestListener {
        fn component_registered(&self, component: &ComponentDefinitionRef) {
            self.events
                .lock()
                .unwrap()
                .push(format!("component:{}", component.bean_name()));
        }

        fn alias_registered(&self, alias: &AliasDefinition) {
            self.events
                .lock()
                .unwrap()
                .push(format!("alias:{}", alias.alias()));
        }

        fn import_registered(&self, import: &ImportDefinition) {
            self.events
                .lock()
                .unwrap()
                .push(format!("import:{}", import.imported_resource()));
        }
    }

    #[test]
    fn test_component_registered_event() {
        let listener = TestListener::new();
        let comp = ComponentDefinitionRef::new("myBean", Some("com.example.MyBean".to_string()));
        listener.component_registered(&comp);
        assert_eq!(listener.event_count(), 1);
    }

    #[test]
    fn test_alias_registered_event() {
        let listener = TestListener::new();
        let alias = AliasDefinition::new("myBean", "myAlias");
        listener.alias_registered(&alias);
        assert_eq!(listener.event_count(), 1);
    }

    #[test]
    fn test_import_registered_event() {
        let listener = TestListener::new();
        let import = ImportDefinition::new("classpath:other.xml");
        listener.import_registered(&import);
        assert_eq!(listener.event_count(), 1);
    }
}
