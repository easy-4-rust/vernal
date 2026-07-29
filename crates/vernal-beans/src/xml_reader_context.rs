//! xml_reader_context — XML 读取器的上下文（ReaderContext）。
//!
//! 对应 Java 类：`org.springframework.beans.factory.parsing.ReaderContext`。
//!
//! 在 XML Bean 定义读取过程中携带环境信息：资源、注册表引用、
//! 命名空间处理器解析器等，并向读取流程提供错误上报入口。

use std::fmt;
use std::sync::Arc;

use crate::bean_definition_registry::BeanDefinitionRegistry;
use crate::namespace_handler_resolver::NamespaceHandlerResolver;
use crate::resource::Resource;

/// XML 读取上下文。
///
/// 对应 Spring 的 `ReaderContext`。
///
/// 把读取过程中所需的协作对象集中起来，并在读取流程中传递。
pub struct ReaderContext {
    /// 当前资源（用于错误消息中的描述）。
    resource: Arc<dyn Resource>,
    /// Bean 定义注册表。
    registry: Arc<dyn BeanDefinitionRegistry>,
    /// 命名空间处理器解析器。
    namespace_handler_resolver: Option<Arc<dyn NamespaceHandlerResolver>>,
}

impl ReaderContext {
    /// 创建新的读取上下文。
    pub fn new(
        resource: Arc<dyn Resource>,
        registry: Arc<dyn BeanDefinitionRegistry>,
        namespace_handler_resolver: Option<Arc<dyn NamespaceHandlerResolver>>,
    ) -> Self {
        Self {
            resource,
            registry,
            namespace_handler_resolver,
        }
    }

    /// 返回资源描述字符串。
    pub fn resource_description(&self) -> String {
        self.resource.description()
    }

    /// 返回资源引用。
    pub fn resource(&self) -> &dyn Resource {
        self.resource.as_ref()
    }

    /// 返回注册表引用。
    pub fn registry(&self) -> &dyn BeanDefinitionRegistry {
        self.registry.as_ref()
    }

    /// 返回底层注册表句柄的克隆（便于外部持有）。
    pub fn registry_handle(&self) -> Arc<dyn BeanDefinitionRegistry> {
        Arc::clone(&self.registry)
    }

    /// 返回命名空间处理器解析器（可能未配置）。
    pub fn namespace_handler_resolver(&self) -> Option<&dyn NamespaceHandlerResolver> {
        self.namespace_handler_resolver.as_deref()
    }

    /// 报告致命错误（封装为错误结果）。
    pub fn fatal(&self, message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
        format!(
            "Fatal error loading bean definitions from '{}': {}",
            self.resource.description(),
            message.into()
        )
        .into()
    }

    /// 报告错误（封装为错误结果）。
    pub fn error(&self, message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
        format!(
            "Error loading bean definitions from '{}': {}",
            self.resource.description(),
            message.into()
        )
        .into()
    }

    /// 报告警告（输出到标准错误，无返回值）。
    pub fn warning(&self, message: impl AsRef<str>) {
        eprintln!(
            "[vernal-beans] Warning loading bean definitions from '{}': {}",
            self.resource.description(),
            message.as_ref()
        );
    }
}

impl fmt::Debug for ReaderContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReaderContext")
            .field("resource", &self.resource.description())
            .field(
                "has_namespace_handler_resolver",
                &self.namespace_handler_resolver.is_some(),
            )
            .finish_non_exhaustive()
    }
}
