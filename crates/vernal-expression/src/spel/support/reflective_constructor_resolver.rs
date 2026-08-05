//! 反射构造器解析器（对标 Spring `ReflectiveConstructorResolver`）。
//!
//! 通过闭包注册表模拟 Java 反射构造器查找。`resolve()` 按类型名查找
//! 已注册的构造器闭包，支持精确匹配、近似匹配和类型转换匹配。
//!
//! 对应 Java 类：`org.springframework.expression.spel.support.ReflectiveConstructorResolver`。

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::access_exception::AccessException;
use crate::constructor_executor::ConstructorExecutor;
use crate::constructor_resolver::ConstructorResolver;
use crate::evaluation_context::EvaluationContext;
use crate::typed_value::{TypeDescriptor, TypedValue};

/// 构造器闭包类型。
type ConstructorFn = Arc<
    dyn Fn(&dyn EvaluationContext, &[TypedValue]) -> Result<TypedValue, AccessException>
        + Send
        + Sync,
>;

/// 反射构造器执行器（对标 Spring `ReflectiveConstructorExecutor`）。
///
/// 持有 `Arc<dyn Fn(...)>` 闭包，在 `execute` 时调用用户注册的构造器逻辑。
pub struct ReflectiveConstructorExecutor {
    /// 构造器闭包。
    constructor: ConstructorFn,
}

impl ReflectiveConstructorExecutor {
    /// 创建反射构造器执行器。
    ///
    /// # 参数
    ///
    /// - `constructor` — 构造器闭包
    pub fn new<F>(constructor: F) -> Self
    where
        F: Fn(&dyn EvaluationContext, &[TypedValue]) -> Result<TypedValue, AccessException>
            + Send
            + Sync
            + 'static,
    {
        Self {
            constructor: Arc::new(constructor),
        }
    }
}

impl ConstructorExecutor for ReflectiveConstructorExecutor {
    /// 执行构造器。
    ///
    /// 对标 Spring `ReflectiveConstructorExecutor.execute()`。
    fn execute(
        &self,
        context: &dyn EvaluationContext,
        arguments: &[TypedValue],
    ) -> Result<TypedValue, AccessException> {
        (self.constructor)(context, arguments)
    }
}

/// 反射构造器解析器（对标 Spring `ReflectiveConstructorResolver`）。
///
/// 通过闭包注册表模拟 Java 反射构造器查找。支持按类型名注册构造器闭包，
/// `resolve()` 时查找匹配的构造器。
///
/// # 与 Spring 的关系
///
/// Spring `ReflectiveConstructorResolver` 通过 Java 反射获取类的所有构造器，
/// 按参数数量排序，依次尝试精确匹配、近似匹配和类型转换匹配。
///
/// Rust 中通过闭包注册表实现：每个类型名可以注册多个构造器（不同参数数量），
/// `resolve()` 时按参数数量查找最匹配的构造器。
///
/// # 匹配策略
///
/// 对标 Spring 的三种匹配：
/// 1. **精确匹配**：参数类型完全一致
/// 2. **近似匹配**：参数类型是子类型
/// 3. **转换匹配**：通过 TypeConverter 可以转换
///
/// Rust 实现中简化为：按参数数量匹配，找到第一个参数数量匹配的构造器。
pub struct ReflectiveConstructorResolver {
    /// 按类型名索引的构造器闭包注册表。
    ///
    /// 每个类型名对应多个构造器（不同参数数量），按参数数量升序排列。
    constructors: RwLock<HashMap<String, Vec<(usize, ConstructorFn)>>>,
}

impl ReflectiveConstructorResolver {
    /// 创建空的反射构造器解析器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            constructors: RwLock::new(HashMap::new()),
        }
    }

    /// 注册构造器闭包。
    ///
    /// # 参数
    ///
    /// - `type_name` — 类型名（如 "String"、"java.util.ArrayList"）
    /// - `param_count` — 参数数量
    /// - `constructor` — 构造器闭包
    pub fn register_constructor<F>(&self, type_name: &str, param_count: usize, constructor: F)
    where
        F: Fn(&dyn EvaluationContext, &[TypedValue]) -> Result<TypedValue, AccessException>
            + Send
            + Sync
            + 'static,
    {
        let mut constructors = self.constructors.write().unwrap();
        let entry = constructors
            .entry(type_name.to_string())
            .or_insert_with(Vec::new);
        entry.push((param_count, Arc::new(constructor)));
        // 按参数数量升序排列（对标 Spring Arrays.sort(ctors, Comparator.comparingInt(Constructor::getParameterCount))）
        entry.sort_by_key(|(count, _)| *count);
    }
}

impl ConstructorResolver for ReflectiveConstructorResolver {
    /// 解析构造器。
    ///
    /// 对标 Spring `ReflectiveConstructorResolver.resolve()`。
    ///
    /// # 匹配策略
    ///
    /// 1. 按类型名查找已注册的构造器列表
    /// 2. 按参数数量匹配（对标 Spring 的参数数量检查）
    /// 3. 返回第一个匹配的构造器执行器
    ///
    /// # 参数
    ///
    /// - `context` — 求值上下文
    /// - `type_name` — 类型名
    /// - `argument_types` — 参数类型列表
    ///
    /// # 返回
    ///
    /// 构造器执行器（如果找到），否则 `None`。
    fn resolve(
        &self,
        _context: &dyn EvaluationContext,
        type_name: &str,
        argument_types: &[TypeDescriptor],
    ) -> Result<Option<Box<dyn ConstructorExecutor>>, AccessException> {
        let constructors = self.constructors.read().unwrap();

        if let Some(ctor_list) = constructors.get(type_name) {
            let arg_count = argument_types.len();

            // 按参数数量查找匹配的构造器
            // 对标 Spring: 按参数数量排序后依次检查
            for (param_count, constructor) in ctor_list {
                if *param_count == arg_count {
                    // 精确参数数量匹配
                    return Ok(Some(Box::new(ReflectiveConstructorExecutor::new({
                        let constructor = Arc::clone(constructor);
                        move |ctx, args| (constructor)(ctx, args)
                    }))));
                }
            }
        }

        // 未找到匹配的构造器
        // 对标 Spring: return null
        Ok(None)
    }
}

impl Default for ReflectiveConstructorResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spel::support::standard_evaluation_context::StandardEvaluationContext;
    use crate::typed_value::ExpressionValue;

    #[test]
    fn new_creates_empty_resolver() {
        let resolver = ReflectiveConstructorResolver::new();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = resolver.resolve(&ctx, "String", &[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn register_and_resolve_constructor() {
        let resolver = ReflectiveConstructorResolver::new();
        resolver.register_constructor("String", 0, |_ctx, _args| {
            Ok(TypedValue::new(
                ExpressionValue::String(String::new()),
                TypeDescriptor::STRING,
            ))
        });

        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = resolver.resolve(&ctx, "String", &[]).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn resolve_returns_none_for_unregistered_type() {
        let resolver = ReflectiveConstructorResolver::new();
        resolver.register_constructor("String", 0, |_ctx, _args| Ok(TypedValue::null()));

        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = resolver.resolve(&ctx, "Integer", &[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn resolve_returns_none_for_wrong_arg_count() {
        let resolver = ReflectiveConstructorResolver::new();
        resolver.register_constructor("String", 0, |_ctx, _args| Ok(TypedValue::null()));

        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let arg_types = vec![TypeDescriptor::INT];
        let result = resolver.resolve(&ctx, "String", &arg_types).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn resolve_matches_by_arg_count() {
        let resolver = ReflectiveConstructorResolver::new();
        resolver.register_constructor("String", 0, |_ctx, _args| {
            Ok(TypedValue::new(
                ExpressionValue::String("zero-arg".into()),
                TypeDescriptor::STRING,
            ))
        });
        resolver.register_constructor("String", 1, |_ctx, _args| {
            Ok(TypedValue::new(
                ExpressionValue::String("one-arg".into()),
                TypeDescriptor::STRING,
            ))
        });

        let ctx = StandardEvaluationContext::new(TypedValue::null());

        // 零参数
        let result = resolver.resolve(&ctx, "String", &[]).unwrap();
        assert!(result.is_some());

        // 一参数
        let arg_types = vec![TypeDescriptor::STRING];
        let result = resolver.resolve(&ctx, "String", &arg_types).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn constructor_executor_execute() {
        let resolver = ReflectiveConstructorResolver::new();
        resolver.register_constructor("String", 1, |_ctx, args| {
            if let Some(arg) = args.first() {
                Ok(arg.clone())
            } else {
                Ok(TypedValue::null())
            }
        });

        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let arg_types = vec![TypeDescriptor::STRING];
        let executor = resolver
            .resolve(&ctx, "String", &arg_types)
            .unwrap()
            .unwrap();

        let arg = TypedValue::new(
            ExpressionValue::String("hello".into()),
            TypeDescriptor::STRING,
        );
        let result = executor.execute(&ctx, &[arg]).unwrap();
        assert_eq!(*result.value(), ExpressionValue::String("hello".into()));
    }

    #[test]
    fn default_trait() {
        let resolver = ReflectiveConstructorResolver::default();
        let ctx = StandardEvaluationContext::new(TypedValue::null());
        let result = resolver.resolve(&ctx, "String", &[]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn constructors_sorted_by_param_count() {
        let resolver = ReflectiveConstructorResolver::new();
        // 注册顺序：2参数、0参数、1参数
        resolver.register_constructor("Foo", 2, |_ctx, _args| Ok(TypedValue::null()));
        resolver.register_constructor("Foo", 0, |_ctx, _args| Ok(TypedValue::null()));
        resolver.register_constructor("Foo", 1, |_ctx, _args| Ok(TypedValue::null()));

        let constructors = resolver.constructors.read().unwrap();
        let foo_ctors = constructors.get("Foo").unwrap();
        // 应该按参数数量升序排列
        assert_eq!(foo_ctors[0].0, 0);
        assert_eq!(foo_ctors[1].0, 1);
        assert_eq!(foo_ctors[2].0, 2);
    }
}
