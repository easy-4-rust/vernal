//! 闭包驱动的自定义组件条件对象。

use std::{fmt, sync::Arc};

use vernal_core::BoxError;

use crate::{ApplicationEnvironment, ComponentCondition};

type ConditionPredicate =
    dyn Fn(&ApplicationEnvironment) -> Result<bool, BoxError> + Send + Sync + 'static;

/// 把一个线程安全闭包适配成 [`ComponentCondition`]。
///
/// 该对象用于框架扩展和业务侧自定义条件，例如根据多个属性做联合判断。闭包只会
/// 在应用 `build` 阶段调用一次；运行期请求不会重复求值。诊断只记录 `name`，
/// 不会序列化闭包捕获的数据。
#[derive(Clone)]
pub struct PredicateCondition {
    name: &'static str,
    predicate: Arc<ConditionPredicate>,
}

impl PredicateCondition {
    /// 使用静态诊断名和判断闭包创建自定义条件。
    #[must_use]
    pub fn new<F>(name: &'static str, predicate: F) -> Self
    where
        F: Fn(&ApplicationEnvironment) -> Result<bool, BoxError> + Send + Sync + 'static,
    {
        Self {
            name,
            predicate: Arc::new(predicate),
        }
    }
}

impl ComponentCondition for PredicateCondition {
    fn name(&self) -> &'static str {
        self.name
    }

    fn matches(&self, environment: &ApplicationEnvironment) -> Result<bool, BoxError> {
        (self.predicate)(environment)
    }
}

impl fmt::Debug for PredicateCondition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PredicateCondition")
            .field("name", &self.name)
            .field("predicate", &"<opaque>")
            .finish_non_exhaustive()
    }
}
