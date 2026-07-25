//! 组件名称切点对象。

use std::sync::Arc;

use crate::{Operation, Pointcut};

/// 匹配指定逻辑组件全部操作的切点。
///
/// 名称采用精确比较，不执行正则表达式，也不读取动态请求数据。这样组件切点既
/// 适用于 Rust 服务类型，也适用于 Web Handler、消息消费者和任务处理器的稳定
/// 逻辑名称，并保持计划构建结果确定。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentPointcut {
    component: Arc<str>,
}

impl ComponentPointcut {
    /// 使用稳定逻辑组件名创建切点。
    #[must_use]
    pub fn new(component: impl Into<Arc<str>>) -> Self {
        Self {
            component: component.into(),
        }
    }

    /// 返回目标逻辑组件名。
    #[must_use]
    pub fn component(&self) -> &str {
        &self.component
    }
}

impl Pointcut for ComponentPointcut {
    /// 只比较操作的组件部分。
    fn matches(&self, operation: &Operation) -> bool {
        operation.component() == self.component()
    }
}
