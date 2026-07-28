//! 反射方法解析器（对标 Spring `ReflectiveMethodResolver` + `ReflectiveMethodExecutor`）。
//!
//! 存储 `Arc<dyn Fn>` 闭包，resolve 时通过 `Arc::clone` 创建新执行器。
//! 零 unsafe，`#![forbid(unsafe_code)]` 友好。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::method_executor::MethodExecutor;
use crate::method_resolver::MethodResolver;
use crate::typed_value::{TypeDescriptor, TypedValue};

/// Arc 包装的方法执行器（对标 Spring `ReflectiveMethodExecutor`）。
///
/// 持有 `Arc<dyn Fn(...)>` 闭包，在 `execute` 时调用用户注册的方法逻辑。
/// `Clone` 通过 `Arc::clone` 实现，零 unsafe。
pub struct ArcReflectiveMethodExecutor {
    inner: Arc<dyn Fn(&dyn EvaluationContext, &TypedValue, &[TypedValue]) -> Result<TypedValue, AccessException> + Send + Sync>,
}

impl ArcReflectiveMethodExecutor {
    /// 创建 Arc 包装的方法执行器。
    pub fn new<F>(executor: F) -> Self
    where
        F: Fn(&dyn EvaluationContext, &TypedValue, &[TypedValue]) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
    {
        Self { inner: Arc::new(executor) }
    }
}

impl MethodExecutor for ArcReflectiveMethodExecutor {
    fn execute(&self, context: &dyn EvaluationContext, target: &TypedValue, arguments: &[TypedValue]) -> Result<TypedValue, AccessException> {
        (self.inner)(context, target, arguments)
    }
}

impl Clone for ArcReflectiveMethodExecutor {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

/// 方法闭包类型（内部存储用）。
type MethodFn = Arc<dyn Fn(&dyn EvaluationContext, &TypedValue, &[TypedValue]) -> Result<TypedValue, AccessException> + Send + Sync>;

/// 反射方法解析器（对标 Spring `ReflectiveMethodResolver`）。
///
/// 存储按名称索引的闭包列表。`resolve()` 按名称查找，
/// 用 `Arc::clone` 创建新 `ArcReflectiveMethodExecutor` 返回。
pub struct ReflectiveMethodResolver {
    methods: RwLock<HashMap<String, Vec<MethodFn>>>,
}

impl ReflectiveMethodResolver {
    /// 创建空的反射方法解析器。
    #[must_use]
    pub fn new() -> Self {
        Self { methods: RwLock::new(HashMap::new()) }
    }

    /// 注册一个方法（用闭包）。
    pub fn register_fn<F>(&self, name: impl Into<String>, executor: F)
    where
        F: Fn(&dyn EvaluationContext, &TypedValue, &[TypedValue]) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
    {
        if let Ok(mut methods) = self.methods.write() {
            methods.entry(name.into()).or_default().push(Arc::new(executor));
        }
    }

    /// 注册一个方法执行器。
    pub fn register(&self, name: impl Into<String>, executor: ArcReflectiveMethodExecutor) {
        if let Ok(mut methods) = self.methods.write() {
            methods.entry(name.into()).or_default().push(executor.inner);
        }
    }

    /// 检查是否包含指定名称的方法。
    #[must_use]
    pub fn has_method(&self, name: &str) -> bool {
        self.methods.read().map(|m| m.contains_key(name)).unwrap_or(false)
    }

    /// 获取已注册的方法名称列表。
    #[must_use]
    pub fn method_names(&self) -> Vec<String> {
        self.methods.read().map(|m| m.keys().cloned().collect()).unwrap_or_default()
    }
}

impl Default for ReflectiveMethodResolver {
    fn default() -> Self { Self::new() }
}

impl MethodResolver for ReflectiveMethodResolver {
    fn resolve(
        &self,
        _context: &dyn EvaluationContext,
        _target: &TypedValue,
        name: &str,
        _argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn MethodExecutor>>, AccessException> {
        let methods = self.methods.read()
            .map_err(|e| AccessException::new(format!("读取方法表失败: {e}")))?;

        if let Some(closures) = methods.get(name) {
            if let Some(closure) = closures.first() {
                let executor = ArcReflectiveMethodExecutor { inner: Arc::clone(closure) };
                return Ok(Some(Box::new(executor)));
            }
        }
        Ok(None)
    }
}

/// 保留旧类型（兼容性）。
pub struct ReflectiveMethodExecutor;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_value::ExpressionValue;
    use crate::type_descriptor::{PrimitiveKind, TypeDescriptor};

    #[test]
    fn resolver_register_and_resolve() {
        let resolver = ReflectiveMethodResolver::new();
        assert!(!resolver.has_method("add"));
        resolver.register_fn("add", |_, _, _| {
            Ok(TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::Primitive(PrimitiveKind::Int)))
        });
        assert!(resolver.has_method("add"));
        assert_eq!(resolver.method_names(), vec!["add"]);
    }

    #[test]
    fn resolver_returns_none_for_unknown() {
        let resolver = ReflectiveMethodResolver::new();
        let ctx = crate::spel::support::standard_evaluation_context::StandardEvaluationContext::new_default();
        let result = resolver.resolve(&ctx, &TypedValue::null(), "unknown", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn executor_executes_closure() {
        let executor = ArcReflectiveMethodExecutor::new(|_, _, _| {
            Ok(TypedValue::new(ExpressionValue::Int(100), TypeDescriptor::Primitive(PrimitiveKind::Int)))
        });
        let ctx = crate::spel::support::standard_evaluation_context::StandardEvaluationContext::new_default();
        let result = executor.execute(&ctx, &TypedValue::null(), &[]);
        assert_eq!(result.unwrap().value(), &ExpressionValue::Int(100));
    }

    #[test]
    fn resolver_resolve_returns_executable() {
        let resolver = ReflectiveMethodResolver::new();
        resolver.register_fn("double", |_, _, args| {
            if let Some(ExpressionValue::Int(i)) = args.first().map(|a| a.value()) {
                Ok(TypedValue::new(ExpressionValue::Int(i * 2), TypeDescriptor::Primitive(PrimitiveKind::Int)))
            } else {
                Ok(TypedValue::null())
            }
        });

        let ctx = crate::spel::support::standard_evaluation_context::StandardEvaluationContext::new_default();
        let resolved = resolver.resolve(&ctx, &TypedValue::null(), "double", &[]).unwrap().unwrap();
        let result = resolved.execute(&ctx, &TypedValue::null(), &[TypedValue::new(
            ExpressionValue::Int(21), TypeDescriptor::Primitive(PrimitiveKind::Int),
        )]).unwrap();
        assert_eq!(result.value(), &ExpressionValue::Int(42));
    }

    #[test]
    fn multiple_methods_registered() {
        let resolver = ReflectiveMethodResolver::new();
        resolver.register_fn("add", |_, _, _| Ok(TypedValue::null()));
        resolver.register_fn("multiply", |_, _, _| Ok(TypedValue::null()));
        assert!(resolver.has_method("add"));
        assert!(resolver.has_method("multiply"));
    }
}
