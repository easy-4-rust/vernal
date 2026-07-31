//! 方法名称切点对象。

use std::sync::Arc;

use crate::{Operation, Pointcut};

/// 匹配所有组件中指定方法或端点名称的切点。
///
/// 该对象适合对同名操作施加统一能力，例如全部 `create` 命令或全部 `GET`
/// 端点。若还需要限制组件，应通过 [`crate::PointcutExt::and`] 与
/// [`crate::ComponentPointcut`] 组合，而不是把多个职责塞入一个切点类型。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MethodPointcut {
    method: Arc<str>,
}

impl MethodPointcut {
    /// 使用稳定方法或端点名称创建切点。
    #[must_use]
    pub fn new(method: impl Into<Arc<str>>) -> Self {
        Self {
            method: method.into(),
        }
    }

    /// 返回目标方法或端点名称。
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }
}

impl Pointcut for MethodPointcut {
    /// 只比较操作的方法部分。
    fn matches(&self, operation: &Operation) -> bool {
        operation.method() == self.method()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Operation;

    #[test]
    fn method_pointcut_matches() {
        let pc = MethodPointcut::new("method");
        let op = Operation::new("Service", "method");
        assert!(pc.matches(&op));
    }

    #[test]
    fn method_pointcut_no_match() {
        let pc = MethodPointcut::new("method");
        let op = Operation::new("Service", "other");
        assert!(!pc.matches(&op));
    }
}
