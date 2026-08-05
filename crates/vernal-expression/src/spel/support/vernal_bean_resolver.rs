//! Vernal Bean 解析器（对标 Spring `BeanResolver` 集成）。
//!
//! `VernalBeanResolver` 通过可注入的闭包实现 Bean 解析，
//! 允许在运行时从 IoC 容器（如 `vernal-beans::ComponentRegistry`）解析 Bean。
//!
//! 对标 Spring 中 `BeanFactoryResolver` 的角色：将 `BeanResolver` 接口
//! 桥接到实际的 IoC 容器。
//!
//! 对应 Java 类：Spring 没有直接等价类，概念上对标 `BeanFactoryResolver`。

use std::sync::{Arc, RwLock};

use crate::access_exception::AccessException;
use crate::bean_resolver::BeanResolver;
use crate::evaluation_context::EvaluationContext;
use crate::typed_value::TypedValue;

/// Bean 解析闭包类型。
///
/// 接收 Bean 名称，返回解析到的 `TypedValue`。
type BeanResolverFn = Arc<dyn Fn(&str) -> Result<TypedValue, AccessException> + Send + Sync>;

/// Vernal Bean 解析器（对标 Spring `BeanFactoryResolver`）。
///
/// 通过可注入的闭包实现 Bean 解析。用户可以在构造时提供一个
/// 闭包，该闭包内部可以调用 `vernal-beans::ComponentRegistry`
/// 或任何其他 IoC 容器来解析 Bean。
///
/// # 与 Spring 的关系
///
/// Spring `BeanFactoryResolver` 持有 `BeanFactory` 引用，
/// `resolve()` 时调用 `beanFactory.getBean(name)`。
///
/// Rust 中通过闭包实现：用户构造 `VernalBeanResolver` 时提供
/// 一个解析闭包，该闭包内部可以调用任何 IoC 容器。
///
/// # 使用场景
///
/// 在 SpEL 表达式中使用 `@beanName` 语法引用 IoC 容器中的 Bean。
///
/// # 示例
///
/// ```rust
/// use vernal_expression::spel::support::vernal_bean_resolver::VernalBeanResolver;
/// use vernal_expression::BeanResolver;
/// use vernal_expression::EvaluationContext;
/// use vernal_expression::{TypedValue, ExpressionValue, TypeDescriptor};
/// use vernal_expression::spel::support::standard_evaluation_context::StandardEvaluationContext;
///
/// // 创建解析器，注册一个简单的 Bean 解析闭包
/// let resolver = VernalBeanResolver::new(|name| match name {
///     "myBean" => Ok(TypedValue::new(
///         ExpressionValue::String("bean-value".into()),
///         TypeDescriptor::STRING,
///     )),
///     _ => Err(vernal_expression::AccessException::new(
///         format!("Bean '{}' not found", name),
///     )),
/// });
///
/// let ctx = StandardEvaluationContext::new(TypedValue::null());
/// let result = resolver.resolve(&ctx, "myBean").unwrap();
/// assert_eq!(*result.value(), ExpressionValue::String("bean-value".into()));
/// ```
pub struct VernalBeanResolver {
    /// Bean 解析闭包。
    resolver_fn: RwLock<Option<BeanResolverFn>>,
}

impl VernalBeanResolver {
    /// 创建新的 Vernal Bean 解析器。
    ///
    /// # 参数
    ///
    /// - `resolver` — Bean 解析闭包，接收 Bean 名称，返回 `TypedValue`
    pub fn new<F>(resolver: F) -> Self
    where
        F: Fn(&str) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
    {
        Self {
            resolver_fn: RwLock::new(Some(Arc::new(resolver))),
        }
    }

    /// 创建空的 Vernal Bean 解析器（未配置解析闭包）。
    ///
    /// 所有 Bean 解析将返回错误。
    pub fn empty() -> Self {
        Self {
            resolver_fn: RwLock::new(None),
        }
    }

    /// 设置 Bean 解析闭包。
    ///
    /// 允许在运行时动态配置解析逻辑。
    pub fn set_resolver<F>(&self, resolver: F)
    where
        F: Fn(&str) -> Result<TypedValue, AccessException> + Send + Sync + 'static,
    {
        let mut guard = self.resolver_fn.write().unwrap();
        *guard = Some(Arc::new(resolver));
    }
}

impl BeanResolver for VernalBeanResolver {
    /// 解析 Bean。
    ///
    /// 对标 Spring `BeanFactoryResolver.resolve()`。
    ///
    /// # 参数
    ///
    /// - `context` — 求值上下文（当前未使用，保留用于未来扩展）
    /// - `bean_name` — Bean 名称
    ///
    /// # 返回
    ///
    /// 解析到的 `TypedValue`，或 `AccessException`。
    fn resolve(
        &self,
        _context: &dyn EvaluationContext,
        bean_name: &str,
    ) -> Result<TypedValue, AccessException> {
        let guard = self.resolver_fn.read().unwrap();
        if let Some(resolver_fn) = guard.as_ref() {
            resolver_fn(bean_name)
        } else {
            Err(AccessException::new(&format!(
                "No bean resolver configured: cannot resolve bean '{}'",
                bean_name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::{ExpressionValue, TypeDescriptor};

    #[test]
    fn new_with_resolver() {
        let resolver = VernalBeanResolver::new(|name| {
            Ok(TypedValue::new(
                ExpressionValue::String(format!("bean:{}", name)),
                TypeDescriptor::STRING,
            ))
        });
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = resolver.resolve(&ctx, "myBean").unwrap();
        assert_eq!(
            *result.value(),
            ExpressionValue::String("bean:myBean".into())
        );
    }

    #[test]
    fn empty_resolver_returns_error() {
        let resolver = VernalBeanResolver::empty();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = resolver.resolve(&ctx, "myBean");
        assert!(result.is_err());
    }

    #[test]
    fn resolver_returns_error_for_unknown_bean() {
        let resolver = VernalBeanResolver::new(|name| {
            Err(AccessException::new(format!("Bean '{}' not found", name)))
        });
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = resolver.resolve(&ctx, "unknown");
        assert!(result.is_err());
    }

    #[test]
    fn set_resolver_dynamically() {
        let resolver = VernalBeanResolver::empty();
        let ctx = StandardEvaluationContext::new(TypedValue::null());

        // 初始状态：无解析闭包
        assert!(resolver.resolve(&ctx, "bean").is_err());

        // 动态设置解析闭包
        resolver.set_resolver(|name| {
            Ok(TypedValue::new(
                ExpressionValue::String(format!("resolved:{}", name)),
                TypeDescriptor::STRING,
            ))
        });

        let result = resolver.resolve(&ctx, "bean").unwrap();
        assert_eq!(
            *result.value(),
            ExpressionValue::String("resolved:bean".into())
        );
    }

    #[test]
    fn resolve_multiple_beans() {
        let resolver = VernalBeanResolver::new(|name| match name {
            "bean1" => Ok(TypedValue::new(
                ExpressionValue::Int(1),
                TypeDescriptor::INT,
            )),
            "bean2" => Ok(TypedValue::new(
                ExpressionValue::Int(2),
                TypeDescriptor::INT,
            )),
            _ => Err(AccessException::new(format!("Bean '{}' not found", name))),
        });

        let ctx = StandardEvaluationContext::new(TypedValue::null());

        let r1 = resolver.resolve(&ctx, "bean1").unwrap();
        assert_eq!(*r1.value(), ExpressionValue::Int(1));

        let r2 = resolver.resolve(&ctx, "bean2").unwrap();
        assert_eq!(*r2.value(), ExpressionValue::Int(2));

        assert!(resolver.resolve(&ctx, "bean3").is_err());
    }
}
