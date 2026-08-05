//! 求值状态（对标 Spring `ExpressionState`）。

use std::collections::HashMap;

use crate::{EvaluationContext, SpelEvaluationException, SpelMessage, TypedValue};

/// 默认最大操作数限制（对标 Spring `SpelParserConfiguration.DEFAULT_MAX_OPERATIONS`）。
const DEFAULT_MAX_OPERATIONS: u32 = 10_000;

/// 求值状态（对标 Spring `ExpressionState`）。
///
/// 维护每次表达式求值的活动上下文对象栈、scope 根对象栈和操作计数。
/// 操作计数超过 `max_operations` 时抛出 `SpelEvaluationException`。
pub struct ExpressionState<'a> {
    /// 求值上下文。
    context: &'a dyn EvaluationContext,
    /// 活动上下文对象栈（push/pop）。
    active_context: Vec<TypedValue>,
    /// scope 根对象栈（enter_scope/exit_scope）。
    /// 用于 Selection/Projection/Indexer 建立新的 #this 作用域。
    scope_root_objects: Vec<TypedValue>,
    /// 局部变量。
    variables: HashMap<String, TypedValue>,
    /// 操作计数。
    operation_count: u32,
    /// 最大操作数限制。
    max_operations: u32,
    /// 是否自动增长集合（对标 Spring SpelParserConfiguration.autoGrowCollections）。
    auto_grow_collections: bool,
    /// 集合自动增长最大尺寸（对标 Spring SpelParserConfiguration.maximumAutoGrowSize）。
    maximum_auto_grow_size: usize,
}

impl<'a> ExpressionState<'a> {
    /// 创建状态。
    #[must_use]
    pub fn new(context: &'a dyn EvaluationContext) -> Self {
        Self {
            context,
            active_context: vec![context.root_object().clone()],
            scope_root_objects: Vec::new(),
            variables: HashMap::new(),
            operation_count: 0,
            max_operations: DEFAULT_MAX_OPERATIONS,
            auto_grow_collections: false,
            maximum_auto_grow_size: 256,
        }
    }

    /// 创建状态并指定最大操作数限制。
    #[must_use]
    pub fn with_max_operations(context: &'a dyn EvaluationContext, max_operations: u32) -> Self {
        Self {
            context,
            active_context: vec![context.root_object().clone()],
            scope_root_objects: Vec::new(),
            variables: HashMap::new(),
            operation_count: 0,
            max_operations,
            auto_grow_collections: false,
            maximum_auto_grow_size: 256,
        }
    }

    /// 设置是否自动增长集合。
    pub fn set_auto_grow_collections(&mut self, auto_grow: bool) {
        self.auto_grow_collections = auto_grow;
    }

    /// 设置集合自动增长最大尺寸。
    pub fn set_maximum_auto_grow_size(&mut self, max_size: usize) {
        self.maximum_auto_grow_size = max_size;
    }

    /// 是否自动增长集合。
    #[must_use]
    pub fn auto_grow_collections(&self) -> bool {
        self.auto_grow_collections
    }

    /// 集合自动增长最大尺寸。
    #[must_use]
    pub fn maximum_auto_grow_size(&self) -> usize {
        self.maximum_auto_grow_size
    }

    /// 设置局部变量。
    pub fn set_variable(&mut self, name: &str, value: TypedValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// 查找变量。
    #[must_use]
    pub fn lookup_variable(&self, name: &str) -> Option<TypedValue> {
        if let Some(v) = self.variables.get(name) {
            return Some(v.clone());
        }
        self.context.lookup_variable(name)
    }

    /// 获取当前活动上下文对象（栈顶）。
    #[must_use]
    pub fn active_context_object(&self) -> &TypedValue {
        self.active_context
            .last()
            .expect("active context stack empty")
    }

    /// 压入活动上下文。
    pub fn push_active_context_object(&mut self, value: TypedValue) {
        self.active_context.push(value);
    }

    /// 弹出活动上下文。
    pub fn pop_active_context_object(&mut self) -> TypedValue {
        self.active_context
            .pop()
            .expect("active context stack underflow")
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

    // ── Scope 管理（对标 Spring ExpressionState enterScope/exitScope） ──

    /// 进入新 scope（对标 Spring `ExpressionState.enterScope()`）。
    ///
    /// 将当前活动上下文对象压入 scope 根对象栈，
    /// Selection/Projection/Indexer 使用此方法建立新的 #this 作用域。
    pub fn enter_scope(&mut self) {
        let current = self.active_context_object().clone();
        self.scope_root_objects.push(current);
    }

    /// 退出 scope（对标 Spring `ExpressionState.exitScope()`）。
    ///
    /// 弹出 scope 根对象栈顶。
    pub fn exit_scope(&mut self) {
        self.scope_root_objects.pop();
    }

    /// 获取 scope 根上下文对象（对标 Spring `ExpressionState.getScopeRootContextObject()`）。
    ///
    /// 返回 scope 根对象栈顶，如果栈为空则返回根对象。
    #[must_use]
    pub fn scope_root_context_object(&self) -> &TypedValue {
        self.scope_root_objects
            .last()
            .unwrap_or_else(|| self.context.root_object())
    }

    // ── 类型转换（对标 Spring ExpressionState.convertValue） ──

    /// 转换值到目标类型（对标 Spring `ExpressionState.convertValue()`）。
    ///
    /// 委托给上下文的 TypeConverter。
    pub fn convert_value(
        &self,
        value: &TypedValue,
        target_type: &crate::TypeDescriptor,
    ) -> Result<TypedValue, SpelEvaluationException> {
        let converter = self.context.type_converter().ok_or_else(|| {
            SpelEvaluationException::new(
                SpelMessage::TypeConversionError,
                &["No TypeConverter configured"],
            )
        })?;
        converter.convert_value(value, target_type).map_err(|e| {
            SpelEvaluationException::new(
                SpelMessage::TypeConversionError,
                &[
                    &value.type_descriptor().name(),
                    &target_type.name(),
                    &e.to_string(),
                ],
            )
        })
    }

    // ── 类型查找（对标 Spring ExpressionState.findType） ──

    /// 查找类型（对标 Spring `ExpressionState.findType()`）。
    pub fn find_type(&self, type_name: &str) -> Result<std::any::TypeId, SpelEvaluationException> {
        let locator = self
            .context
            .type_locator()
            .ok_or_else(|| SpelEvaluationException::new(SpelMessage::TypeNotFound, &[type_name]))?;
        locator.find_type(type_name).map_err(|e| {
            SpelEvaluationException::new(SpelMessage::TypeNotFound, &[type_name, &e.to_string()])
        })
    }

    // ── 访问器（对标 Spring ExpressionState 的各种 getter） ──

    /// 获取类型比较器。
    #[must_use]
    pub fn type_comparator(&self) -> Option<&dyn crate::TypeComparator> {
        self.context.type_comparator()
    }

    /// 获取类型转换器。
    #[must_use]
    pub fn type_converter(&self) -> Option<&dyn crate::TypeConverter> {
        self.context.type_converter()
    }

    /// 获取属性访问器列表。
    #[must_use]
    pub fn property_accessors(&self) -> Vec<&dyn crate::PropertyAccessor> {
        self.context.property_accessors()
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
        assert_eq!(
            *state.active_context_object().value(),
            ExpressionValue::Int(1)
        );

        state.push_active_context_object(v2.clone());
        assert_eq!(
            *state.active_context_object().value(),
            ExpressionValue::Int(2)
        );

        let popped = state.pop_active_context_object();
        assert_eq!(*popped.value(), ExpressionValue::Int(2));
        assert_eq!(
            *state.active_context_object().value(),
            ExpressionValue::Int(1)
        );

        let popped = state.pop_active_context_object();
        assert_eq!(*popped.value(), ExpressionValue::Int(1));
        assert!(state.active_context_object().is_null());
    }

    #[test]
    fn set_and_lookup_variable() {
        let ctx = make_ctx();
        let mut state = ExpressionState::new(&ctx);

        let val = TypedValue::new(
            ExpressionValue::String("hello".into()),
            TypeDescriptor::STRING,
        );
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
    fn lookup_variable_falls_through_to_context() {
        // StandardEvaluationContext::lookup_variable now returns owned values
        let mut ctx = StandardEvaluationContext::new(TypedValue::null());
        ctx.set_variable(
            "ctx_var",
            TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT),
        );
        let state = ExpressionState::new(&ctx);
        let found = state.lookup_variable("ctx_var").unwrap();
        assert_eq!(*found.value(), ExpressionValue::Int(42));
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
