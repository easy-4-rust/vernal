//! 求值状态（对标 Spring `ExpressionState`）。

use std::collections::HashMap;

use crate::evaluation_context::EvaluationContext;
use crate::evaluation_exception::EvaluationException;
use crate::typed_value::TypedValue;

/// 求值状态（对标 Spring `ExpressionState`）。
///
/// 维护每次表达式求值的活动上下文对象栈和操作计数。
pub struct ExpressionState<'a> {
    /// 求值上下文。
    context: &'a dyn EvaluationContext,
    /// 活动上下文对象栈（push/pop）。
    active_context: Vec<TypedValue>,
    /// 局部变量。
    variables: HashMap<String, TypedValue>,
    /// 操作计数。
    operation_count: u32,
}

impl<'a> ExpressionState<'a> {
    /// 创建状态。
    #[must_use]
    pub fn new(context: &'a dyn EvaluationContext) -> Self {
        Self {
            context,
            active_context: vec![context.root_object().clone()],
            variables: HashMap::new(),
            operation_count: 0,
        }
    }

    /// 设置局部变量。
    pub fn set_variable(&mut self, name: &str, value: TypedValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// 查找变量。
    #[must_use]
    pub fn lookup_variable(&self, name: &str) -> Option<&TypedValue> {
        if let Some(v) = self.variables.get(name) {
            return Some(v);
        }
        self.context.lookup_variable(name)
    }

    /// 获取当前活动上下文对象（栈顶）。
    #[must_use]
    pub fn active_context_object(&self) -> &TypedValue {
        self.active_context.last().expect("active context stack empty")
    }

    /// 压入活动上下文。
    pub fn push_active_context_object(&mut self, value: TypedValue) {
        self.active_context.push(value);
    }

    /// 弹出活动上下文。
    pub fn pop_active_context_object(&mut self) -> TypedValue {
        self.active_context.pop().expect("active context stack underflow")
    }

    /// 获取求值上下文。
    #[must_use]
    pub fn evaluation_context(&self) -> &'a dyn EvaluationContext {
        self.context
    }

    /// 跟踪一次操作。
    pub fn track_operation(&mut self) {
        self.operation_count = self.operation_count.saturating_add(1);
    }
}
