//! 求值状态（对标 Spring `ExpressionState`）。

use std::collections::HashMap;

use crate::{EvaluationContext, SpelEvaluationException, SpelMessage, TypedValue};

/// 默认最大操作数限制（对标 Spring `SpelParserConfiguration.DEFAULT_MAX_OPERATIONS`）。
const DEFAULT_MAX_OPERATIONS: u32 = 10_000;

/// 求值状态（对标 Spring `ExpressionState`）。
///
/// 维护每次表达式求值的活动上下文对象栈和操作计数。
/// 操作计数超过 `max_operations` 时抛出 `SpelEvaluationException`。
pub struct ExpressionState<'a> {
    /// 求值上下文。
    context: &'a dyn EvaluationContext,
    /// 活动上下文对象栈（push/pop）。
    active_context: Vec<TypedValue>,
    /// 局部变量。
    variables: HashMap<String, TypedValue>,
    /// 操作计数。
    operation_count: u32,
    /// 最大操作数限制。
    max_operations: u32,
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
            max_operations: DEFAULT_MAX_OPERATIONS,
        }
    }

    /// 创建状态并指定最大操作数限制。
    #[must_use]
    pub fn with_max_operations(context: &'a dyn EvaluationContext, max_operations: u32) -> Self {
        Self {
            context,
            active_context: vec![context.root_object().clone()],
            variables: HashMap::new(),
            operation_count: 0,
            max_operations,
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
    ///
    /// 对标 Spring `ExpressionState.trackOperation()`。
    /// 超过 `max_operations` 时返回 `Err(SpelEvaluationException)`。
    pub fn track_operation(&mut self) -> Result<(), SpelEvaluationException> {
        self.operation_count = self.operation_count.saturating_add(1);
        if self.operation_count > self.max_operations {
            return Err(SpelEvaluationException::new(
                SpelMessage::MaxOperationsExceeded,
                &[&self.max_operations.to_string()],
            ));
        }
        Ok(())
    }

    /// 获取当前操作计数。
    #[must_use]
    pub fn operation_count(&self) -> u32 {
        self.operation_count
    }

    /// 获取最大操作数限制。
    #[must_use]
    pub fn max_operations(&self) -> u32 {
        self.max_operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::{ExpressionValue, TypeDescriptor};

    fn make_ctx() -> StandardEvaluationContext {
        StandardEvaluationContext::new(TypedValue::null())
    }

    #[test]
    fn new_state_has_root_as_active_context() {
        let ctx = make_ctx();
        let state = ExpressionState::new(&ctx);
        assert!(state.active_context_object().is_null());
    }

    #[test]
    fn push_and_pop_active_context() {
        let ctx = make_ctx();
        let mut state = ExpressionState::new(&ctx);

        let v1 = TypedValue::new(ExpressionValue::Int(1), TypeDescriptor::INT);
        let v2 = TypedValue::new(ExpressionValue::Int(2), TypeDescriptor::INT);

        state.push_active_context_object(v1.clone());
        assert_eq!(*state.active_context_object().value(), ExpressionValue::Int(1));

        state.push_active_context_object(v2.clone());
        assert_eq!(*state.active_context_object().value(), ExpressionValue::Int(2));

        let popped = state.pop_active_context_object();
        assert_eq!(*popped.value(), ExpressionValue::Int(2));
        assert_eq!(*state.active_context_object().value(), ExpressionValue::Int(1));

        let popped = state.pop_active_context_object();
        assert_eq!(*popped.value(), ExpressionValue::Int(1));
        assert!(state.active_context_object().is_null());
    }

    #[test]
    fn set_and_lookup_variable() {
        let ctx = make_ctx();
        let mut state = ExpressionState::new(&ctx);

        let val = TypedValue::new(ExpressionValue::String("hello".into()), TypeDescriptor::STRING);
        state.set_variable("greeting", val.clone());

        let found = state.lookup_variable("greeting").unwrap();
        assert_eq!(*found.value(), ExpressionValue::String("hello".into()));
    }

    #[test]
    fn lookup_variable_not_found() {
        let ctx = make_ctx();
        let state = ExpressionState::new(&ctx);
        assert!(state.lookup_variable("nonexistent").is_none());
    }

    #[test]
    fn lookup_variable_context_returns_none() {
        // StandardEvaluationContext::lookup_variable returns None
        // because RwLock<HashMap> cannot return &TypedValue (lifetime issue).
        // Variables are stored but not retrievable via the trait method.
        // ExpressionState has its own variables HashMap that works correctly.
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let state = ExpressionState::new(&ctx);
        assert!(state.lookup_variable("ctx_var").is_none());
    }

    #[test]
    fn local_variable_works_independently() {
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let mut state = ExpressionState::new(&ctx);

        state.set_variable(
            "x",
            TypedValue::new(ExpressionValue::Int(99), TypeDescriptor::INT),
        );

        let found = state.lookup_variable("x").unwrap();
        assert_eq!(*found.value(), ExpressionValue::Int(99));
    }

    #[test]
    fn evaluation_context_returns_context() {
        let ctx = make_ctx();
        let state = ExpressionState::new(&ctx);
        let _ = state.evaluation_context();
    }

    #[test]
    fn track_operation_increments_count() {
        let ctx = make_ctx();
        let mut state = ExpressionState::new(&ctx);
        assert_eq!(state.operation_count(), 0);

        assert!(state.track_operation().is_ok());
        assert_eq!(state.operation_count(), 1);

        assert!(state.track_operation().is_ok());
        assert_eq!(state.operation_count(), 2);

        assert!(state.track_operation().is_ok());
        assert_eq!(state.operation_count(), 3);
    }

    #[test]
    fn track_operation_enforces_limit() {
        let ctx = make_ctx();
        let mut state = ExpressionState::with_max_operations(&ctx, 3);
        assert_eq!(state.max_operations(), 3);

        assert!(state.track_operation().is_ok());
        assert!(state.track_operation().is_ok());
        assert!(state.track_operation().is_ok());
        // 第4次操作应该超过限制
        assert!(state.track_operation().is_err());
    }

    #[test]
    fn with_max_operations_sets_limit() {
        let ctx = make_ctx();
        let state = ExpressionState::with_max_operations(&ctx, 100);
        assert_eq!(state.max_operations(), 100);
        assert_eq!(state.operation_count(), 0);
    }

    #[test]
    fn multiple_push_pop_cycles() {
        let ctx = make_ctx();
        let mut state = ExpressionState::new(&ctx);

        for i in 0..10 {
            state.push_active_context_object(TypedValue::new(
                ExpressionValue::Int(i),
                TypeDescriptor::INT,
            ));
        }

        for i in (0..10).rev() {
            let popped = state.pop_active_context_object();
            assert_eq!(*popped.value(), ExpressionValue::Int(i));
        }

        assert!(state.active_context_object().is_null());
    }
}
