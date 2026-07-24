//! Web 路由元数据对象。

use std::sync::Arc;

use vernal_aop::Operation;

/// Adapter 解析完成后的稳定路由描述。
///
/// `path_template` 应使用 `/orders/{id}` 一类低基数模板，而不是包含用户输入的
/// 原始 URI；它可直接映射成 AOP Operation 与指标标签。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RouteMetadata {
    handler: Arc<str>,
    operation: Arc<str>,
    path_template: Arc<str>,
}

impl RouteMetadata {
    /// 创建路由元数据。
    #[must_use]
    pub fn new(
        handler: impl Into<Arc<str>>,
        operation: impl Into<Arc<str>>,
        path_template: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            handler: handler.into(),
            operation: operation.into(),
            path_template: path_template.into(),
        }
    }

    /// 返回 Handler 逻辑名称。
    #[must_use]
    pub fn handler(&self) -> &str {
        &self.handler
    }

    /// 返回操作名称。
    #[must_use]
    pub fn operation_name(&self) -> &str {
        &self.operation
    }

    /// 返回低基数路由模板。
    #[must_use]
    pub fn path_template(&self) -> &str {
        &self.path_template
    }

    /// 转换成 AOP 操作描述。
    #[must_use]
    pub fn aop_operation(&self) -> Operation {
        Operation::new(Arc::clone(&self.handler), Arc::clone(&self.operation))
    }
}
