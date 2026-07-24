//! 被拦截操作描述对象。

use std::{fmt, sync::Arc};

/// 描述一次可被切面匹配的组件操作。
///
/// `component` 通常是组件类型或逻辑服务名，`method` 是方法或端点名称。两段式
/// 表达既适合普通组件方法，也能映射 HTTP 路由、消息消费和任务执行。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Operation {
    component: Arc<str>,
    method: Arc<str>,
}

impl Operation {
    /// 创建操作描述。
    #[must_use]
    pub fn new(component: impl Into<Arc<str>>, method: impl Into<Arc<str>>) -> Self {
        Self {
            component: component.into(),
            method: method.into(),
        }
    }

    /// 返回组件逻辑名称。
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }

    /// 返回方法或端点名称。
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.component, self.method)
    }
}
