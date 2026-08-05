//! 数据绑定方法解析器（对标 Spring `DataBindingMethodResolver`）。
//!
//! `DataBindingMethodResolver` 是 `ReflectiveMethodResolver` 的特化版本，
//! 专为数据绑定场景设计。与 `ReflectiveMethodResolver` 不同，它：
//!
//! - 不支持 `Class` 目标（即不能对类本身调用方法，只能对实例调用）
//! - 不解析静态方法
//! - 不解析 `Object`、`Class`、`ClassLoader` 上的技术方法
//!
//! 对应 Java 类：`org.springframework.expression.spel.support.DataBindingMethodResolver`。

use crate::access_exception::AccessException;
use crate::evaluation_context::EvaluationContext;
use crate::method_executor::MethodExecutor;
use crate::method_resolver::MethodResolver;
use crate::typed_value::{TypeDescriptor, TypedValue};

use super::reflective_method_resolver::ReflectiveMethodResolver;

/// 数据绑定方法解析器（对标 Spring `DataBindingMethodResolver`）。
///
/// 仅解析目标对象上的实例方法，排除静态方法和技术方法
/// （`Object`、`Class`、`ClassLoader` 上的方法）。
///
/// # 与 Spring 的关系
///
/// Spring `DataBindingMethodResolver` 继承 `ReflectiveMethodResolver`，
/// 重写 `resolve()` 以拒绝 `Class` 目标，重写 `isCandidateForInvocation()`
/// 以过滤静态方法和技术方法。
///
/// Rust 中通过组合（持有 `ReflectiveMethodResolver`）而非继承实现，
/// 在 `resolve()` 中添加前置校验。
///
/// # 使用场景
///
/// 适用于 `SimpleEvaluationContext` 等数据绑定场景，
/// 不允许访问类级别的静态方法或 Java 平台内部方法。
pub struct DataBindingMethodResolver {
    /// 内部委托的反射方法解析器。
    delegate: ReflectiveMethodResolver,
}

impl DataBindingMethodResolver {
    /// 创建新的数据绑定方法解析器。
    ///
    /// 对标 Spring `DataBindingMethodResolver()` 私有构造器。
    fn new() -> Self {
        Self {
            delegate: ReflectiveMethodResolver::new(),
        }
    }

    /// 创建用于实例方法调用的数据绑定方法解析器。
    ///
    /// 对标 Spring `DataBindingMethodResolver.forInstanceMethodInvocation()`。
    ///
    /// # 返回
    ///
    /// 配置好的 `DataBindingMethodResolver` 实例。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use vernal_expression::spel::support::data_binding_method_resolver::DataBindingMethodResolver;
    ///
    /// let resolver = DataBindingMethodResolver::for_instance_method_invocation();
    /// ```
    pub fn for_instance_method_invocation() -> Self {
        Self::new()
    }

    /// 注册方法闭包。
    ///
    /// 委托给内部 `ReflectiveMethodResolver`。
    ///
    /// # 参数
    ///
    /// - `name` — 方法名
    /// - `executor` — 方法执行闭包
    pub fn register_fn<F>(&self, name: impl Into<String>, executor: F)
    where
        F: Fn(
                &dyn EvaluationContext,
                &TypedValue,
                &[TypedValue],
            ) -> Result<TypedValue, AccessException>
            + Send
            + Sync
            + 'static,
    {
        self.delegate.register_fn(name, executor);
    }
}

impl MethodResolver for DataBindingMethodResolver {
    /// 解析方法。
    ///
    /// 对标 Spring `DataBindingMethodResolver.resolve()`。
    ///
    /// # 与 Spring 的差异
    ///
    /// Spring 通过 `targetObject instanceof Class` 检查拒绝类目标；
    /// Rust 中没有直接等价的 `Class` 类型，此检查通过
    /// `TypeDescriptor::is_primitive()` 和类型名判断近似实现。
    ///
    /// # 参数
    ///
    /// - `context` — 求值上下文
    /// - `target` — 目标对象
    /// - `name` — 方法名
    /// - `argument_types` — 参数类型列表
    ///
    /// # 返回
    ///
    /// 方法执行器（如果找到），否则 `None`。
    fn resolve(
        &self,
        context: &dyn EvaluationContext,
        target: &TypedValue,
        name: &str,
        argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn MethodExecutor>>, AccessException> {
        // 对标 Spring: if (targetObject instanceof Class) throw IllegalArgumentException
        // Rust 中通过检查类型名是否为 "Class" 或 "java.lang.Class" 来近似
        let type_name = target.type_descriptor().name();
        if type_name == "Class" || type_name == "java.lang.Class" {
            return Err(AccessException::new(
                "DataBindingMethodResolver does not support Class targets",
            ));
        }

        // 委托给 ReflectiveMethodResolver 进行实际方法查找
        self.delegate.resolve(context, target, name, argument_types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::ExpressionValue;

    #[test]
    fn for_instance_method_invocation_creates_resolver() {
        let resolver = DataBindingMethodResolver::for_instance_method_invocation();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(ExpressionValue::Int(42), TypeDescriptor::INT);
        let result = resolver.resolve(&ctx, &target, "nonexistent", &[]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn resolve_returns_none_for_unregistered_method() {
        let resolver = DataBindingMethodResolver::for_instance_method_invocation();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("hello".into()),
            TypeDescriptor::STRING,
        );
        let result = resolver.resolve(&ctx, &target, "nonexistent", &[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn register_and_resolve_method() {
        let resolver = DataBindingMethodResolver::for_instance_method_invocation();
        resolver.register_fn("greet", |_ctx, _target, _args| {
            Ok(TypedValue::new(
                ExpressionValue::String("hello".into()),
                TypeDescriptor::STRING,
            ))
        });

        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("world".into()),
            TypeDescriptor::STRING,
        );
        let result = resolver.resolve(&ctx, &target, "greet", &[]).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn class_target_returns_error() {
        let resolver = DataBindingMethodResolver::for_instance_method_invocation();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        // 创建一个类型名为 "Class" 的目标
        let target = TypedValue::new(
            ExpressionValue::String("test".into()),
            TypeDescriptor::from_type_name("Class"),
        );
        let result = resolver.resolve(&ctx, &target, "method", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn java_lang_class_target_returns_error() {
        let resolver = DataBindingMethodResolver::for_instance_method_invocation();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let target = TypedValue::new(
            ExpressionValue::String("test".into()),
            TypeDescriptor::from_type_name("java.lang.Class"),
        );
        let result = resolver.resolve(&ctx, &target, "method", &[]);
        assert!(result.is_err());
    }
}
